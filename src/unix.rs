use anyhow::Result;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::os::fd::RawFd;

pub fn set_non_blocking(fd: i32, non_blocking: bool) -> Result<RawFd> {
    let flag = fcntl(fd, FcntlArg::F_GETFL)?;
    let mut flag = OFlag::from_bits(flag).unwrap();
    flag.set(OFlag::O_NONBLOCK, non_blocking);
    // set the descriptor file flag
    let arg = FcntlArg::F_SETFL(flag);
    let _ = fcntl(fd, arg)?;
    Ok(fd)
}
