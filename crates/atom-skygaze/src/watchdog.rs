use std::fs::File;
use std::os::fd::AsRawFd;

const WDIOC_SETOPTIONS: ::std::os::raw::c_ulong = 0x40045704;
const WDIOC_KEEPALIVE: ::std::os::raw::c_ulong = 0x40045705;
const WDIOC_SETTIMEOUT: ::std::os::raw::c_ulong = 0xc0045706;
const WDIOS_ENABLECARD: ::std::os::raw::c_int = 0x0002;

pub struct WatchdogManager(File);

impl WatchdogManager {
    pub fn init(count: ::std::os::raw::c_int) -> std::io::Result<Self> {
        let file = File::open("/dev/watchdog")?;
        let fd = file.as_raw_fd();
        unsafe {
            let option_ptr: *const ::std::os::raw::c_int = &WDIOS_ENABLECARD;
            if libc::ioctl(fd, WDIOC_SETOPTIONS, option_ptr) != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "WDIOC_SETOPTIONS fail",
                ));
            }

            let count_ptr: *const ::std::os::raw::c_int = &count;
            if libc::ioctl(fd, WDIOC_SETTIMEOUT, count_ptr) != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "WDIOC_SETTIMEOUT fail",
                ));
            }

            if libc::ioctl(fd, WDIOC_KEEPALIVE, 0) != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "WDIOC_KEEPALIVE fail",
                ));
            }
        }
        Ok(Self(file))
    }

    pub fn make_instance(&self) -> Watchdog {
        Watchdog {
            fd: self.0.as_raw_fd(),
        }
    }
}

pub struct Watchdog {
    fd: ::std::os::raw::c_int,
}

impl Watchdog {
    pub fn feed(&self) -> std::io::Result<()> {
        unsafe {
            if libc::ioctl(self.fd, WDIOC_KEEPALIVE, 0) != 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "WDIOC_KEEPALIVE fail",
                ));
            }
        }
        Ok(())
    }
}
