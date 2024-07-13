use anyhow::{anyhow, Result};
use nix::{
    errno::Errno,
    fcntl::OFlag,
    poll::{poll, PollFd, PollFlags, PollTimeout},
    unistd::{read, write},
};
use std::os::fd::{AsFd, AsRawFd, OwnedFd, RawFd};

use crate::unix;

// Package rwcancel implements cancelable read/write operations on
// a file descriptor.
pub struct RwCancel {
    // TODO: change to OwnedFd later if needed or Arc<OwnedFd>, or a BorrowFd
    fd: OwnedFd,
    closing_reader: OwnedFd,
    closing_writer: OwnedFd,
}

pub fn retry_after_error(err: Errno) -> bool {
    return err == Errno::EAGAIN || err == Errno::EINTR;
}

impl RwCancel {
    pub fn new(fd: OwnedFd) -> Result<Self> {
        let _ = unix::set_non_blocking(fd.as_raw_fd(), true)?;
        let (closing_reader, closing_writer) =
            nix::unistd::pipe2(OFlag::O_CLOEXEC.union(OFlag::O_NONBLOCK))?;
        // TODO: convert fd to OwnedFd may change to Arc OwnedFd later for ownership
        Ok(Self {
            fd,
            closing_reader,
            closing_writer,
        })
    }

    pub fn ready_read(&self) -> bool {
        let close_fd = self.closing_reader.as_fd();
        let mut fds = [
            PollFd::new(self.fd.as_fd(), PollFlags::POLLIN),
            PollFd::new(close_fd, PollFlags::POLLIN),
        ];

        let has_err = loop {
            let poll_res = poll(&mut fds, PollTimeout::NONE);
            if poll_res.is_ok() {
                break false;
            }
            if let Err(err) = poll_res {
                if !retry_after_error(err) {
                    break true;
                }
            }
        };

        let is_cancel = fds[1].revents().map(|ev| !ev.is_empty()).unwrap_or(true);
        let ready = fds[0].revents().map(|ev| !ev.is_empty()).unwrap_or(true);
        return !has_err && !is_cancel && ready;
    }

    pub fn ready_write(&self) -> bool {
        let close_fd = self.closing_reader.as_fd();
        let mut fds = [
            PollFd::new(self.fd.as_fd(), PollFlags::POLLOUT),
            PollFd::new(close_fd, PollFlags::POLLOUT),
        ];

        let has_err = loop {
            let poll_res = poll(&mut fds, PollTimeout::NONE);
            if poll_res.is_ok() {
                break false;
            }
            if let Err(err) = poll_res {
                if !retry_after_error(err) {
                    break true;
                }
            }
        };

        let is_cancel = fds[1].revents().map(|ev| !ev.is_empty()).unwrap_or(true);
        let ready = fds[0].revents().map(|ev| !ev.is_empty()).unwrap_or(true);
        return !has_err && !is_cancel && ready;
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match read(self.fd.as_raw_fd(), buf) {
                Ok(n) => return Ok(n),
                Err(err) => {
                    if !retry_after_error(err) {
                        return Err(std::io::Error::from(err).into());
                    }
                }
            }

            if !self.ready_read() {
                return Err(anyhow!("file already closed"));
            }
        }
    }
    pub fn write(&self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match write(self.fd.as_fd(), buf) {
                Ok(n) => return Ok(n),
                Err(err) => {
                    if !retry_after_error(err) {
                        return Err(std::io::Error::from(err).into());
                    }
                }
            }

            if !self.ready_write() {
                return Err(anyhow!("file already closed"));
            }
        }
    }

    pub fn cancel(&self) -> Result<()> {
        let _ = write(self.closing_writer.as_fd(), &mut [0u8])?;
        Ok(())
    }
}
