use anyhow::Result;
use std::fs::File;

pub struct Tun {}

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
