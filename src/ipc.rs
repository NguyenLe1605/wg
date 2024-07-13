use anyhow::{Error, Result};
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Receiver;
use std::{fs::File, os::unix::net::UnixListener};

use crate::rwcancel::RwCancel;

const SOCKET_DIRECTORY: &str = "/var/run/wireguard";

fn sock_path(iface: &str) -> String {
    format!("{}/{}.sock", SOCKET_DIRECTORY, iface)
}

pub struct UAPIListener {
    // unix socket listener
    listener: UnixListener,
    conn_new: Receiver<UnixStream>,
    conn_err: Receiver<Error>,
    inotify_fd: i32,
    inotify_cancel: RwCancel,
}

pub fn uapi_open(name: &str) -> Result<File> {
    unimplemented!()
}
