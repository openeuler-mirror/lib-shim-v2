#![crate_type = "dylib"]
pub mod client;
pub mod protocols;

use client::client::{new_conn, get_conn};
use std::os::raw::{c_char, c_int};
use std::ffi::CStr;

fn to_string(x: *const c_char) -> String {
    unsafe { if x.is_null() { "".to_string() } else { CStr::from_ptr(x).to_str().unwrap_or_default().to_string() } }
}

#[no_mangle]
pub extern "C" fn shim_v2_new(container_id: *const c_char, addr: *const c_char) -> c_int {
    let (r_container_id, r_addr) = (
        to_string(container_id),
        to_string(addr)
    );
    println!("lib-shim-v2::new::{}:: [{}]", r_container_id, r_addr);
    if let Err(e) = new_conn(&r_container_id, &r_addr) {
        println!("lib-shim-v2::new::{}:: failed, {}.", r_container_id, e);
        return -1;
    }

    println!("lib-shim-v2::new::{}:: done.", r_container_id);
    0
}

#[no_mangle]
pub extern "C" fn shim_v2_create(container_id: *const c_char, bundle: *const c_char, terminal: bool,
        stdin: *const c_char, stdout: *const c_char, stderr: *const c_char, pid: &mut c_int) -> c_int {
    let (r_container_id, r_bundle, r_stdin, r_stdout, r_stderr) = (
        to_string(container_id),
        to_string(bundle),
        to_string(stdin),
        to_string(stdout),
        to_string(stderr)
    );
    println!("lib-shim-v2::creating::{}:: [{} {} {} {} {}]",
            r_container_id, r_bundle, terminal, r_stdin, r_stdout, r_stderr);
    get_conn(&r_container_id).and_then(|client| {
        client.create(&r_container_id, &r_bundle, terminal, &r_stdin, &r_stdout, &r_stderr).map(|process_pid| {
            *pid = process_pid;
            println!("lib-shim-v2::create::{}:: done.", r_container_id);
            0
        })
    }).unwrap_or_else(|e| {
        println!("lib-shim-v2::create::{}:: failed, {}.", r_container_id, e);
        -1
    })
}

#[no_mangle]
pub extern "C" fn shim_v2_start(container_id: *const c_char, exec_id: *const c_char, pid: &mut c_int) -> c_int {
    let (r_container_id, r_exec_id) = (
        to_string(container_id),
        to_string(exec_id)
    );
    println!("lib-shim-v2::start::{}:: [{}]", r_container_id, r_exec_id);
    get_conn(&r_container_id).and_then(|client| {
        client.start(&r_container_id, &r_exec_id).map(|process_pid| {
            *pid = process_pid;
            println!("lib-shim-v2::start::{}:: done.", r_container_id);
            0
        })
    }).unwrap_or_else(|e| {
        println!("lib-shim-v2::start::{}:: failed, {}.", r_container_id, e);
        -1
    })
}