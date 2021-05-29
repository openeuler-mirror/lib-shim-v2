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

use super::error::{Error, Result};
use crate::protocols;
use lazy_static::lazy_static;
use nix::sys::socket::*;
use protocols::task::Status as shim_v2_status;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::path::MAIN_SEPARATOR;
use std::sync::Mutex;
use ttrpc::client::Client;

#[derive(Clone)]
pub struct Store {
    conn: Client,
    timeout: i64,
}

#[derive(Debug)]
pub struct DeleteResponse {
    pub exit_status: u32,
    pub pid: u32,
}

#[derive(Debug)]
pub enum Status {
    UnknownStatus,
    CreatedStatus,
    RunningStatus,
    StoppedStatus,
    DeletedStatus,
    PauseStatus,
    PausingStatus,
}

#[derive(Debug)]
pub struct State {
    pub id: ::std::string::String,
    pub pid: u32,
    pub status: Status,
    pub stdin: ::std::string::String,
    pub stdout: ::std::string::String,
    pub stderr: ::std::string::String,
    pub terminal: bool,
    pub exit_status: u32,
}

lazy_static! {
    static ref TTRPC_CLIENTS: Mutex<HashMap<String, Store>> = Mutex::new(HashMap::new());
}

fn unix_sock(r#abstract: bool, socket_path: &str) -> Result<SockAddr> {
    let sockaddr_u = if r#abstract {
        let sockaddr_h = socket_path.to_owned() + &"\x00".to_string();
        UnixAddr::new_abstract(sockaddr_h.as_bytes())
    } else {
        UnixAddr::new(socket_path)
    }
    .map_err(other_error!(e, "failed to create socket: "))?;

    let sockaddr = SockAddr::Unix(sockaddr_u);
    Ok(sockaddr)
}

fn connect_to_socket(abs: bool, address: &str) -> Result<RawFd> {
    let fd = socket(
        AddressFamily::Unix,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(other_error!(e, "failed to create socket fd: "))?;

    let sockaddr = unix_sock(abs, address)?;
    connect(fd, &sockaddr).map_err(other_error!(e, "failed to connect socket: "))?;
    Ok(fd)
}

pub fn new_conn(container_id: &String, addr: &String) -> Result<()> {
    let address = if addr.starts_with("unix://") {
        addr.strip_prefix("unix://").unwrap()
    } else {
        addr
    };

    let path = Path::new(&MAIN_SEPARATOR.to_string()).join(address);
    let fd = connect_to_socket(true, &path.to_string_lossy())?;
    TTRPC_CLIENTS.lock().unwrap().insert(
        container_id.clone(),
        Store {
            conn: Client::new(fd),
            timeout: 0,
        },
    );

    Ok(())
}

pub fn get_conn(container_id: &String) -> Result<Store> {
    if TTRPC_CLIENTS.lock().unwrap().contains_key(container_id) {
        Ok(TTRPC_CLIENTS
            .lock()
            .unwrap()
            .get(container_id)
            .unwrap()
            .clone())
    } else {
        Err(other!("connection has not been established..."))
    }
}

pub fn del_conn(container_id: &String) {
    TTRPC_CLIENTS.lock().unwrap().remove(container_id);
}

struct ValidateTool {}

impl ValidateTool {
    fn str_empty(self, x: &String) -> Result<Self> {
        return if x != "" {
            Ok(self)
        } else {
            Err(other!("parameter must not be empty!"))
        };
    }
}

impl Store {
    pub fn create(
        &self,
        container_id: &String,
        bundle: &String,
        terminal: bool,
        stdin: &String,
        stdout: &String,
        stderr: &String,
    ) -> Result<i32> {
        ValidateTool {}.str_empty(container_id)?.str_empty(bundle)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::CreateTaskRequest::new();
        req.set_id(container_id.clone());
        req.set_bundle(bundle.clone());
        req.set_terminal(terminal);
        req.set_stdin(stdin.clone());
        req.set_stdout(stdout.clone());
        req.set_stderr(stderr.clone());

        let resp = client
            .create(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call create failed"))?;

        Ok(resp.pid as i32)
    }

    pub fn start(&self, container_id: &String, exec_id: &String) -> Result<i32> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::StartRequest::new();
        req.set_id(container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .start(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call start failed"))?;

        Ok(resp.pid as i32)
    }

    #[allow(unused)]
    pub fn kill(
        &self,
        container_id: &String,
        exec_id: &String,
        signal: u32,
        all: bool,
    ) -> Result<()> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::KillRequest::new();
        req.set_id(container_id.clone());
        // unused variable: exec_id
        req.set_signal(signal);
        req.set_all(all);

        client
            .kill(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call kill failed"))?;

        Ok(())
    }

    pub fn delete(&self, container_id: &String, exec_id: &String) -> Result<DeleteResponse> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::DeleteRequest::new();
        req.set_id(container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .delete(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call delete failed"))?;

        Ok(DeleteResponse {
            exit_status: resp.exit_status,
            pid: resp.pid,
        })
    }

    pub fn shutdown(&self, container_id: &String) -> Result<()> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ShutdownRequest::new();
        req.set_id(container_id.clone());

        client
            .shutdown(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call shutdown failed"))?;

        Ok(())
    }

    pub fn exec(
        &self,
        container_id: &String,
        exec_id: &String,
        terminal: bool,
        stdin: &String,
        stdout: &String,
        stderr: &String,
        spec: &[u8],
    ) -> Result<()> {
        ValidateTool {}
            .str_empty(container_id)?
            .str_empty(exec_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ExecProcessRequest::new();
        req.set_id(container_id.clone());
        req.set_exec_id(exec_id.clone());
        req.set_terminal(terminal);
        req.set_stdin(stdin.clone());
        req.set_stdout(stdout.clone());
        req.set_stderr(stderr.clone());
        let mut exec_spec: ::protobuf::well_known_types::Any =
            ::protobuf::well_known_types::Any::new();
        exec_spec.set_value(std::vec::Vec::from(spec));
        exec_spec.set_type_url(String::from(
            "types.containerd.io/opencontainers/runtime-spec/1/Process",
        ));
        req.set_spec(exec_spec);

        client
            .exec(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call exec failed"))?;

        Ok(())
    }

    pub fn resize_pty(
        &self,
        container_id: &String,
        exec_id: &String,
        height: u32,
        width: u32,
    ) -> Result<()> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ResizePtyRequest::new();
        req.set_id(container_id.clone());
        req.set_exec_id(exec_id.clone());
        req.set_height(height);
        req.set_width(width);

        client
            .resize_pty(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call resize_pty failed"))?;

        Ok(())
    }

    pub fn pause(&self, container_id: &String) -> Result<()> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::PauseRequest::new();
        req.set_id(container_id.clone());

        client
            .pause(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call pause failed"))?;

        Ok(())
    }

    pub fn resume(&self, container_id: &String) -> Result<()> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ResumeRequest::new();
        req.set_id(container_id.clone());

        client
            .resume(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call resume failed"))?;

        Ok(())
    }

    pub fn state(&self, container_id: &String) -> Result<State> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::StateRequest::new();
        req.set_id(container_id.clone());

        let resp = client
            .state(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call state failed"))?;

        Ok(State {
            id: container_id.clone(),
            pid: resp.pid,
            status: match resp.status {
                shim_v2_status::CREATED => Status::CreatedStatus,
                shim_v2_status::RUNNING => Status::RunningStatus,
                shim_v2_status::STOPPED => Status::StoppedStatus,
                shim_v2_status::PAUSED => Status::PauseStatus,
                shim_v2_status::PAUSING => Status::PausingStatus,
                _ => Status::UnknownStatus,
            },
            stdin: resp.stdin,
            stdout: resp.stdout,
            stderr: resp.stderr,
            terminal: resp.terminal,
            exit_status: resp.exit_status,
        })
    }

    pub fn pids(&self, container_id: &String) -> Result<i32> {
        let c = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::PidsRequest::new();
        req.id = container_id.clone();

        let resp = c
            .pids(&req, self.timeout)
            .map_err(shim_error!(e, "call pids failed"))?;
        let process = &resp.get_processes()[0];

        Ok(process.pid as i32)
    }

    pub fn wait(&self, container_id: &String, exec_id: &String) -> Result<i32> {
        ValidateTool {}.str_empty(container_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::WaitRequest::new();
        req.set_id(container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .wait(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call wait failed"))?;

        Ok(resp.exit_status as i32)
    }
}
