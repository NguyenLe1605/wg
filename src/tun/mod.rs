use anyhow::Result;
use parking_lot::{Mutex, Once};
use std::fs::File;

use crate::rwcancel::RwCancel;

const VIRTIO_NET_HDR_LEN: usize = 8 * 2 + 16 * 4;
const CLONE_DEVICE_PATH: &str = "dev/net/tun";
const IF_REQ_SIZE: usize = libc::IFNAMSIZ + 64;

#[repr(i32)]
pub enum Event {
    Up = 1 << 0,
    Down = 1 << 1,
    MTUUpdate = 1 << 2,
}

pub struct Tun {
    // The file represent the tun interface
    tun_file: File,
    // index of the interface
    index: i32,
    // TODO: add channels later
    // async error handling
    // errors -> channel of Error

    // device related events
    // events Receievr<Event>

    // TODO: sock for what
    net_link_sock: i32,
    net_link_cancel: RwCancel,

    hack_listener_closed: Mutex<()>,
    // TODO: shutdown channel, add later
    // status_listener_shutdown

    // TODO: what are they batching?
    batch_size: i32,
    // TODO: what is vnet_hdr
    vnet_hdr: bool,
    // TODO: what is this, is it to enable gso optmization???
    udp_gso: bool,

    // guards calling init_name_cache, which sets following fields
    name_once: Once,
    name_cache: String,
    name_err: anyhow::Error,

    // if vnet_hdr every read() is prefixed by virtio_net_hdr
    read_buff: Mutex<[u8; VIRTIO_NET_HDR_LEN + 65535]>,

    // write_op_mu guards to_write, tcp_gro_table
    write_op_mu: Mutex<()>,
    to_write: Vec<isize>,
    // udp and tcp gro table
}

impl Tun {
    pub fn create_tun(if_name: &str, mtu: isize) -> Result<Self> {
        unimplemented!()
    }

    pub fn create_tun_from_file(file: File, mtu: isize) -> Result<Self> {
        unimplemented!()
    }

    // File returns the file descriptor of the device.
    // pub fn file(&mut self) ->  Option<File>

    /// name returns the current name of the TUN device.
    pub fn name(&self) -> Result<String> {
        unimplemented!()
    }
}
