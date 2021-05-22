// Copyright (c) 2020 Huawei Technologies Co.,Ltd. All rights reserved.
//
// lib-shim-v2 is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

#![crate_type = "dylib"]
pub mod client;
pub mod protocols;

use crate::Status::{
    CreatedStatus, DeletedStatus, PauseStatus, PausingStatus, RunningStatus, StoppedStatus,
    UnknownStatus,
};
use client::client::State as client_state;
use client::client::Status as client_status;
use client::client::{del_conn, get_conn, new_conn};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};

fn to_string(x: *const c_char) -> String {
    unsafe {
        if x.is_null() {
            "".to_string()
        } else {
            CStr::from_ptr(x).to_str().unwrap_or_default().to_string()
        }
    }
}

#[no_mangle]
pub extern "C" fn shim_v2_new(container_id: *const c_char, addr: *const c_char) -> c_int {
    let (r_container_id, r_addr) = (to_string(container_id), to_string(addr));
    println!("lib-shim-v2::new::{}:: [{}]", r_container_id, r_addr);
    if let Err(e) = new_conn(&r_container_id, &r_addr) {
        println!("lib-shim-v2::new::{}:: failed, {}.", r_container_id, e);
        return -1;
    }

    println!("lib-shim-v2::new::{}:: done.", r_container_id);
    0
}

#[no_mangle]
pub extern "C" fn shim_v2_close(container_id: *const c_char) -> c_int {
    let r_container_id = to_string(container_id);
    println!("lib-shim-v2::close::{}::", r_container_id);
    del_conn(&r_container_id);
    0
}

#[no_mangle]
pub extern "C" fn shim_v2_create(
    container_id: *const c_char,
    bundle: *const c_char,
    terminal: bool,
    stdin: *const c_char,
    stdout: *const c_char,
    stderr: *const c_char,
    pid: &mut c_int,
) -> c_int {
    let (r_container_id, r_bundle, r_stdin, r_stdout, r_stderr) = (
        to_string(container_id),
        to_string(bundle),
        to_string(stdin),
        to_string(stdout),
        to_string(stderr),
    );
    println!(
        "lib-shim-v2::create::{}:: [{} {} {} {} {}]",
        r_container_id, r_bundle, terminal, r_stdin, r_stdout, r_stderr
    );
    get_conn(&r_container_id)
        .and_then(|client| {
            client
                .create(
                    &r_container_id,
                    &r_bundle,
                    terminal,
                    &r_stdin,
                    &r_stdout,
                    &r_stderr,
                )
                .map(|process_pid| {
                    *pid = process_pid;
                    println!("lib-shim-v2::create::{}:: done.", r_container_id);
                    0
                })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::create::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_start(
    container_id: *const c_char,
    exec_id: *const c_char,
    pid: &mut c_int,
) -> c_int {
    let (r_container_id, r_exec_id) = (to_string(container_id), to_string(exec_id));
    println!("lib-shim-v2::start::{}:: [{}]", r_container_id, r_exec_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client
                .start(&r_container_id, &r_exec_id)
                .map(|process_pid| {
                    *pid = process_pid;
                    println!("lib-shim-v2::start::{}:: done.", r_container_id);
                    0
                })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::start::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_kill(
    container_id: *const c_char,
    exec_id: *const c_char,
    signal: u32,
    all: bool,
) -> c_int {
    let (r_container_id, r_exec_id) = (to_string(container_id), to_string(exec_id));
    println!("lib-shim-v2::kill::{}:: [{}]", r_container_id, r_exec_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client
                .kill(&r_container_id, &r_exec_id, signal, all)
                .map(|_| {
                    println!("lib-shim-v2::kill::{}:: done.", r_container_id);
                    0
                })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::kill::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[repr(C)]
pub struct DeleteResponse {
    exit_status: c_uint,
    pid: c_uint,
}

#[no_mangle]
pub extern "C" fn shim_v2_delete(
    container_id: *const c_char,
    exec_id: *const c_char,
    resp: &mut DeleteResponse,
) -> c_int {
    let (r_container_id, r_exec_id) = (to_string(container_id), to_string(exec_id));
    println!("lib-shim-v2::delete::{}:: [{}]", r_container_id, r_exec_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.delete(&r_container_id, &r_exec_id).map(|response| {
                resp.exit_status = response.exit_status;
                resp.pid = response.pid;
                println!("lib-shim-v2::delete::{}:: done.", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::delete::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_shutdown(container_id: *const c_char) -> c_int {
    let r_container_id = to_string(container_id);
    println!("lib-shim-v2::shutdown::{}::", r_container_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.shutdown(&r_container_id).map(|_| {
                println!("lib-shim-v2::shutdown::{}:: done.", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::shutdown::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_exec(
    container_id: *const c_char,
    exec_id: *const c_char,
    terminal: bool,
    stdin: *const c_char,
    stdout: *const c_char,
    stderr: *const c_char,
    spec: *const c_char,
) -> c_int {
    let (r_container_id, r_exec_id, r_stdin, r_stdout, r_stderr) = (
        to_string(container_id),
        to_string(exec_id),
        to_string(stdin),
        to_string(stdout),
        to_string(stderr),
    );
    let r_spec;
    unsafe {
        r_spec = CStr::from_ptr(spec).to_bytes();
    }
    println!(
        "lib-shim-v2::exec::{}:: [{} {} {} {} {}]",
        r_container_id, r_exec_id, terminal, r_stdin, r_stdout, r_stderr
    );
    get_conn(&r_container_id)
        .and_then(|client| {
            client
                .exec(
                    &r_container_id,
                    &r_exec_id,
                    terminal,
                    &r_stdin,
                    &r_stdout,
                    &r_stderr,
                    r_spec,
                )
                .map(|_| {
                    println!("lib-shim-v2::exec::{}:: done.", r_container_id);
                    0
                })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::exec::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_resize_pty(
    container_id: *const c_char,
    exec_id: *const c_char,
    height: u32,
    width: u32,
) -> c_int {
    let (r_container_id, r_exec_id) = (to_string(container_id), to_string(exec_id));
    println!(
        "lib-shim-v2::resize_pty::{}:: [{}]",
        r_container_id, r_exec_id
    );
    get_conn(&r_container_id)
        .and_then(|client| {
            client
                .resize_pty(&r_container_id, &r_exec_id, height, width)
                .map(|_| {
                    println!("lib-shim-v2::resize_pty::{}:: done.", r_container_id);
                    0
                })
        })
        .unwrap_or_else(|e| {
            println!(
                "lib-shim-v2::resize_pty::{}:: failed, {}.",
                r_container_id, e
            );
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_pause(container_id: *const c_char) -> c_int {
    let r_container_id = to_string(container_id);
    println!("lib-shim-v2::pause::{}::", r_container_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.pause(&r_container_id).map(|_| {
                println!("lib-shim-v2::pause::{}:: done.", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::pause::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_resume(container_id: *const c_char) -> c_int {
    let r_container_id = to_string(container_id);
    println!("lib-shim-v2::resume::{}::", r_container_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.resume(&r_container_id).map(|_| {
                println!("lib-shim-v2::resume::{}:: done.", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::resume::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[repr(C)]
pub enum Status {
    UnknownStatus,
    CreatedStatus,
    RunningStatus,
    StoppedStatus,
    DeletedStatus,
    PauseStatus,
    PausingStatus,
}

impl Status {
    fn new(in_obj: client_status) -> Status {
        match in_obj {
            client_status::UnknownStatus => UnknownStatus,
            client_status::CreatedStatus => CreatedStatus,
            client_status::RunningStatus => RunningStatus,
            client_status::StoppedStatus => StoppedStatus,
            client_status::DeletedStatus => DeletedStatus,
            client_status::PauseStatus => PauseStatus,
            client_status::PausingStatus => PausingStatus,
        }
    }
}

#[repr(C)]
pub struct State {
    id: *const c_char,
    pid: c_uint,
    status: Status,
    stdin: *const c_char,
    stdout: *const c_char,
    stderr: *const c_char,
    terminal: bool,
    exit_status: c_uint,
}

impl State {
    fn copy(&mut self, in_obj: client_state) {
        self.id = CString::new(in_obj.id).unwrap().into_raw();
        self.pid = in_obj.pid;
        self.status = Status::new(in_obj.status);
        self.stdin = CString::new(in_obj.stdin).unwrap().into_raw();
        self.stdout = CString::new(in_obj.stdout).unwrap().into_raw();
        self.stderr = CString::new(in_obj.stderr).unwrap().into_raw();
        self.terminal = in_obj.terminal;
        self.exit_status = in_obj.exit_status;
    }
}

#[no_mangle]
pub extern "C" fn shim_v2_state(container_id: *const c_char, state: &mut State) -> c_int {
    let r_container_id = to_string(container_id);
    println!("lib-shim-v2::state::{}::", r_container_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.state(&r_container_id).map(|container_state| {
                state.copy(container_state);
                println!("lib-shim-v2::state::{}:: done.", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("lib-shim-v2::state::{}:: failed, {}.", r_container_id, e);
            -1
        })
}

#[no_mangle]
pub extern "C" fn shim_v2_pids(container_id: *const c_char, pid: &mut c_int) -> c_int {
    let r_container_id = to_string(container_id);
    println!("in rutst::shim_v2_pids::{}:: start.", r_container_id);
    get_conn(&r_container_id)
        .and_then(|client| {
            client.pids(&r_container_id).map(|process_pid| {
                *pid = process_pid;
                println!("in rust::shim_v2_pids::{}:: done", r_container_id);
                0
            })
        })
        .unwrap_or_else(|e| {
            println!("in rust::shim_v2_pids::{}:: failed, {}", r_container_id, e);
            -1
        })
}
