use anyhow::Result;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use std::{
    env,
    fs::File,
    os::fd::{FromRawFd, RawFd},
};
use wg::{
    device::{
        self,
        logger::{self, LogLevel},
    },
    tun::Tun,
    unix,
};

const EXIT_SETUP_SUCCESS: i32 = 0;
const EXIT_SETUP_FAILED: i32 = 1;

const ENV_WG_TUN_FD: &str = "WG_TUN_FD";
const ENV_WG_UAPI_FD: &str = "WG_UAPI_FD";
const ENV_WG_PROCESS_FOREGROUND: &str = "WG_PROCESS_FOREGROUND";

const VERSION: &str = "0.0.20230223";

fn print_usage(prog_name: &str) {
    println!("Usage: {} [-f/--foreground] INTERFACE-NAME", prog_name);
}

fn open_tun_device(if_name: &str) -> Result<Tun> {
    let tun_fd_str = if let Ok(s) = env::var("ENV_WG_TUN_FD") {
        s
    } else {
        return Tun::create_tun(if_name, device::DEFAULT_MTU);
    };

    // construct tun device from supplied fd
    let fd = tun_fd_str.parse::<i32>()?;
    let fd = unix::set_non_blocking(fd, true)?;
    let tun_file = unsafe { File::from_raw_fd(fd) };
    return Tun::create_tun_from_file(tun_file, device::DEFAULT_MTU);
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 2 && args[1] == "--version" {
        println!(
            "wg {}\n\nUserspace WireGuard daemon for {}-{}.",
            VERSION,
            env::consts::OS,
            env::consts::ARCH
        );
        return Ok(());
    }

    let (foreground, interface_name) = match args[1].as_str() {
        "-f" | "--foreground" => {
            if args.len() != 3 {
                print_usage(&args[0]);
                return Ok(());
            }
            (true, args[2].clone())
        }
        _ => {
            if args.len() != 2 {
                print_usage(&args[0]);
                return Ok(());
            }
            let foreground = env::var(ENV_WG_PROCESS_FOREGROUND)
                .map(|s| s == "1")
                .unwrap_or(false);
            (foreground, args[1].clone())
        }
    };

    // get log level, default to info
    let log_level = if let Ok(level) = env::var("LOG_LEVEL") {
        match level.as_str() {
            "verbose" | "debug" => LogLevel::Verbose,
            "error" => LogLevel::Error,
            "silent" => LogLevel::Silent,
            _ => LogLevel::Error,
        }
    } else {
        LogLevel::Error
    };

    logger::init_logger(log_level, &format!("({}) ", interface_name));
    logger::verbose!("Starting wireguard");

    // open TUN device or from fd
    let tun_dev = open_tun_device(&interface_name).unwrap_or_else(|err| {
        logger::error!("Failed to create TUN device: {}", err);
        std::process::exit(EXIT_SETUP_FAILED);
    });
    let interface_name = tun_dev.name().unwrap_or(interface_name);

    // open UAPI file (or use supplied fd)
    Ok(())
}
