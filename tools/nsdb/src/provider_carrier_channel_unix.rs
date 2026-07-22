use crate::provider_carrier_channel::{encode_provider_carrier_frames, fnv1a64};
use std::{
    fs::{self, File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    os::{
        raw::c_int,
        unix::{io::AsRawFd, process::CommandExt},
    },
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const F_GETFD: c_int = 1;
const F_SETFD: c_int = 2;
const FD_CLOEXEC: c_int = 1;
static NEXT_CARRIER_ID: AtomicU64 = AtomicU64::new(0);

extern "C" {
    fn fcntl(fd: c_int, command: c_int, ...) -> c_int;
}

pub(crate) struct InheritedFdCarrier {
    file: File,
    packet_len: usize,
    packet_hash: u64,
}

impl InheritedFdCarrier {
    pub(crate) fn new(frames: &[&[u8]]) -> Result<Self, String> {
        let packet = encode_provider_carrier_frames(frames)?;
        let mut file = create_unlinked_carrier_file()?;
        file.write_all(&packet)
            .map_err(|error| format!("failed to write inherited-fd carrier: {error}"))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|error| format!("failed to rewind inherited-fd carrier: {error}"))?;
        Ok(Self {
            file,
            packet_len: packet.len(),
            packet_hash: fnv1a64(&packet),
        })
    }

    pub(crate) fn frame_argument(&self, frame_index: usize) -> String {
        format!(
            "fd:{}:{frame_index}:{}:{}",
            self.file.as_raw_fd(),
            self.packet_len,
            self.packet_hash
        )
    }

    pub(crate) fn configure_command(&self, command: &mut Command) {
        let fd = self.file.as_raw_fd();
        unsafe {
            command.pre_exec(move || {
                let flags = fcntl(fd, F_GETFD);
                if flags < 0 || fcntl(fd, F_SETFD, flags & !FD_CLOEXEC) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
}

fn create_unlinked_carrier_file() -> Result<File, String> {
    for _ in 0..16 {
        let id = NEXT_CARRIER_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nuis-provider-carrier-{}-{id}.bin",
            std::process::id()
        ));
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => {
                fs::remove_file(&path)
                    .map_err(|error| format!("failed to unlink inherited-fd carrier: {error}"))?;
                return Ok(file);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("failed to create inherited-fd carrier: {error}"));
            }
        }
    }
    Err("failed to allocate a unique inherited-fd carrier".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_binds_fd_frame_length_and_hash() {
        let carrier = InheritedFdCarrier::new(&[b"nuis"]).expect("carrier");
        let argument = carrier.frame_argument(0);
        let fields = argument.split(':').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5);
        assert_eq!(fields[0], "fd");
        assert_eq!(fields[2], "0");
        assert_eq!(fields[3], "36");
        assert!(fields[4].parse::<u64>().is_ok());
        let flags = unsafe { fcntl(carrier.file.as_raw_fd(), F_GETFD) };
        assert_ne!(flags & FD_CLOEXEC, 0);
    }
}
