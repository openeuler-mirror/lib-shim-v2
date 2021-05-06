use crate::protocols;
use super::error::{Error, Result};
use std::sync::Mutex;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use nix::sys::socket::*;
use lazy_static::lazy_static;
use ttrpc::client::Client;
use std::path::Path;
use std::path::MAIN_SEPARATOR;

#[derive(Clone)]
pub struct Store {
    conn: Client,
    timeout: i64,
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
    ).map_err(other_error!(e, "failed to create socket fd: "))?;

    let sockaddr = unix_sock(abs, address)?;
    connect(fd, &sockaddr).map_err(other_error!(e, "failed to connect socket: "))?;
    Ok(fd)
}

pub fn new_conn(container_id: &String, addr: &String) -> Result<()> {
    let address;
    if addr.starts_with("unix://") {
        address = addr.strip_prefix("unix://").unwrap();
    } else {
        address = addr;
    }

    let path = Path::new(&MAIN_SEPARATOR.to_string()).join(address);
    let fd = connect_to_socket(true, &path.to_string_lossy())?;
    TTRPC_CLIENTS.lock().unwrap().insert(container_id.clone(), Store {
        conn: Client::new(fd),
        timeout: 0,
    });

    Ok(())
}

pub fn get_conn(container_id: &String) -> Result<Store> {
    if TTRPC_CLIENTS.lock().unwrap().contains_key(container_id) {
        Ok(TTRPC_CLIENTS.lock().unwrap().get(container_id).unwrap().clone())
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
        return if x != "" { Ok(self) } else { Err(other!("parameter must not be empty!")) };
    }
}

impl Store {
    pub fn create(&self, container_id: &String, bundle: &String, terminal: bool,
                    stdin: &String, stdout: &String, stderr: &String) -> Result<i32> {
        ValidateTool {}.str_empty(container_id)?.str_empty(bundle)?;

        let client = protocols::shim_ttrpc::TaskClient::new(self.conn.clone());

        let mut req = protocols::shim::CreateTaskRequest::new();
        req.id = container_id.clone();
        req.bundle = bundle.clone();
        req.terminal = terminal;
        req.stdin = stdin.clone();
        req.stdout = stdout.clone();
        req.stderr = stderr.clone();

        let resp = client.create(&req, self.timeout).map_err(shim_error!(e, "ttrpc call create failed"))?;

        Ok(resp.pid as i32)
    }
}