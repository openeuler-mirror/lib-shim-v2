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
    container_id: String,
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

fn virtio_vsock(address: &str) -> Result<SockAddr> {
    let (cid, port) = {
        let vec: Vec<String> = address.split(":").map(String::from).collect();
        if vec.len() != 2 {
            let err_msg = format!("vsock address {address} is invalid");
            return Err(other!(err_msg));
        }
        let cid = vec[0].parse::<u32>().map_err(other_error!(e, "failed to parse cid: "))?;
        let port = vec[1].parse::<u32>().map_err(other_error!(e, "failed to parse port: "))?;
        (cid, port)
    };
    let sockaddr = SockAddr::Vsock(VsockAddr::new(cid, port));
    Ok(sockaddr)
}

fn connect_to_vsock(address: &str) -> Result<RawFd> {
    let fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(other_error!(e, "failed to create socket fd: "))?;
    let sockaddr = virtio_vsock(address)?;
    connect(fd, &sockaddr).map_err(other_error!(e, "failed to connect vsock: "))?;
    Ok(fd)
}

fn connect_to_unix_socket(abs: bool, address: &str) -> Result<RawFd> {
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
    let fd = if addr.starts_with("ttrpc+vsock://") {
        let address = addr.strip_prefix("ttrpc+vsock://").unwrap();
        connect_to_vsock(address)?
    } else if addr.starts_with("ttrpc+unix://") {
        let address = addr.strip_prefix("ttrpc+unix://").unwrap();
        let path = Path::new(&MAIN_SEPARATOR.to_string()).join(address);
        connect_to_unix_socket(!addr.starts_with("ttrpc+unix://"), &path.to_string_lossy())?
    } else {
        let address = if addr.starts_with("unix://") {
            addr.strip_prefix("unix://").unwrap()
        } else {
            addr
        };
        let path = Path::new(&MAIN_SEPARATOR.to_string()).join(address);
        connect_to_unix_socket(!addr.starts_with("unix://"), &path.to_string_lossy())?
    };

    TTRPC_CLIENTS.lock().unwrap().insert(
        container_id.clone(),
        Store {
            conn: Client::new(fd),
            container_id: container_id.clone(),
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
        bundle: &String,
        terminal: bool,
        stdin: &String,
        stdout: &String,
        stderr: &String,
    ) -> Result<i32> {
        ValidateTool {}.str_empty(bundle)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::CreateTaskRequest::new();
        req.set_id(self.container_id.clone());
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

    pub fn start(&self, exec_id: &String) -> Result<i32> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::StartRequest::new();
        req.set_id(self.container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .start(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call start failed"))?;

        Ok(resp.pid as i32)
    }

    #[allow(unused)]
    pub fn kill(
        &self,
        signal: u32,
        all: bool,
    ) -> Result<()> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::KillRequest::new();
        req.set_id(self.container_id.clone());
        req.set_signal(signal);
        req.set_all(all);

        client
            .kill(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call kill failed"))?;

        Ok(())
    }

    pub fn delete(&self, exec_id: &String) -> Result<DeleteResponse> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::DeleteRequest::new();
        req.set_id(self.container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .delete(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call delete failed"))?;

        Ok(DeleteResponse {
            exit_status: resp.exit_status,
            pid: resp.pid,
        })
    }

    pub fn shutdown(&self) -> Result<()> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ShutdownRequest::new();
        req.set_id(self.container_id.clone());

        client
            .shutdown(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call shutdown failed"))?;

        Ok(())
    }

    pub fn exec(
        &self,
        exec_id: &String,
        terminal: bool,
        stdin: &String,
        stdout: &String,
        stderr: &String,
        spec: &[u8],
    ) -> Result<()> {
        ValidateTool {}
            .str_empty(exec_id)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ExecProcessRequest::new();
        req.set_id(self.container_id.clone());
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
        exec_id: &String,
        height: u32,
        width: u32,
    ) -> Result<()> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ResizePtyRequest::new();
        req.set_id(self.container_id.clone());
        req.set_exec_id(exec_id.clone());
        req.set_height(height);
        req.set_width(width);

        client
            .resize_pty(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call resize_pty failed"))?;

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::PauseRequest::new();
        req.set_id(self.container_id.clone());

        client
            .pause(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call pause failed"))?;

        Ok(())
    }

    pub fn resume(&self) -> Result<()> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::ResumeRequest::new();
        req.set_id(self.container_id.clone());

        client
            .resume(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call resume failed"))?;

        Ok(())
    }

    pub fn state(&self) -> Result<State> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::StateRequest::new();
        req.set_id(self.container_id.clone());

        let resp = client
            .state(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call state failed"))?;

        Ok(State {
            id: self.container_id.clone(),
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

    pub fn pids(&self) -> Result<i32> {
        let c = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::PidsRequest::new();
        req.id = self.container_id.clone();

        let resp = c
            .pids(&req, self.timeout)
            .map_err(shim_error!(e, "call pids failed"))?;
        let process = &resp.get_processes()[0];

        Ok(process.pid as i32)
    }

    pub fn wait(&self, exec_id: &String) -> Result<i32> {

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::WaitRequest::new();
        req.set_id(self.container_id.clone());
        req.set_exec_id(exec_id.clone());

        let resp = client
            .wait(&req, self.timeout)
            .map_err(shim_error!(e, "ttrpc call wait failed"))?;

        Ok(resp.exit_status as i32)
    }
}
