use crate::provider_carrier_channel::fnv1a64;
#[cfg(test)]
use std::os::unix::fs::FileExt;
use std::{
    ffi::c_void,
    fs::{self, File, OpenOptions},
    io::Write,
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
const PROT_READ: c_int = 1;
const MAP_PRIVATE: c_int = 2;
const INHERITED_FD_MAGIC: &[u8; 8] = b"NUISPFD1";
const INHERITED_FD_HEADER_LEN: usize = 16;
const INHERITED_FD_FRAME_RECORD_LEN: usize = 40;
static NEXT_CARRIER_ID: AtomicU64 = AtomicU64::new(0);

extern "C" {
    fn fcntl(fd: c_int, command: c_int, ...) -> c_int;
    fn getpagesize() -> c_int;
    fn mmap(
        address: *mut c_void,
        length: usize,
        protection: c_int,
        flags: c_int,
        fd: c_int,
        offset: i64,
    ) -> *mut c_void;
    fn munmap(address: *mut c_void, length: usize) -> c_int;
}

pub(crate) struct MappedInheritedFdFrame {
    mapping: *mut c_void,
    packet_len: usize,
    payload_offset: usize,
    payload_len: usize,
}

impl MappedInheritedFdFrame {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.mapping.cast::<u8>().add(self.payload_offset),
                self.payload_len,
            )
        }
    }

    fn packet_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.mapping.cast::<u8>(), self.packet_len) }
    }
}

impl Drop for MappedInheritedFdFrame {
    fn drop(&mut self) {
        unsafe {
            munmap(self.mapping, self.packet_len);
        }
    }
}

pub(crate) struct InheritedFdCarrier {
    file: File,
    packet_len: usize,
    packet_hash: u64,
    frame_layouts: Vec<InheritedFdFrameLayout>,
}

#[derive(Clone)]
struct InheritedFdFrameLayout {
    offset: usize,
    byte_len: usize,
    hash_offset: usize,
}

impl InheritedFdCarrier {
    pub(crate) fn new(frames: &[&[u8]]) -> Result<Self, String> {
        let encoded = encode_inherited_fd_frames(frames)?;
        let mut file = create_unlinked_carrier_file()?;
        file.write_all(&encoded.bytes)
            .map_err(|error| format!("failed to write inherited-fd carrier: {error}"))?;
        Ok(Self {
            file,
            packet_len: encoded.bytes.len(),
            packet_hash: fnv1a64(&encoded.bytes),
            frame_layouts: encoded.layouts,
        })
    }

    pub(crate) fn new_writable_single_frame(byte_len: usize) -> Result<Self, String> {
        let page_size = system_page_size()?;
        let payload_offset = align_up(
            INHERITED_FD_HEADER_LEN + INHERITED_FD_FRAME_RECORD_LEN,
            page_size,
        )?;
        let mapped_len = align_up(byte_len.max(1), page_size)?;
        let packet_len = payload_offset
            .checked_add(mapped_len)
            .ok_or_else(|| "inherited-fd carrier length overflow".to_owned())?;
        let mut header = [0u8; INHERITED_FD_HEADER_LEN + INHERITED_FD_FRAME_RECORD_LEN];
        header[..8].copy_from_slice(INHERITED_FD_MAGIC);
        header[8..12].copy_from_slice(&1u32.to_le_bytes());
        header[12..16].copy_from_slice(
            &u32::try_from(page_size)
                .map_err(|_| "inherited-fd page size overflow".to_owned())?
                .to_le_bytes(),
        );
        header[24..32].copy_from_slice(&(payload_offset as u64).to_le_bytes());
        header[32..40].copy_from_slice(&(byte_len as u64).to_le_bytes());
        header[40..48].copy_from_slice(&(mapped_len as u64).to_le_bytes());
        let mut file = create_unlinked_carrier_file()?;
        file.write_all(&header)
            .map_err(|error| format!("failed to write inherited-fd output header: {error}"))?;
        file.set_len(packet_len as u64)
            .map_err(|error| format!("failed to size inherited-fd output carrier: {error}"))?;
        Ok(Self {
            file,
            packet_len,
            packet_hash: 0,
            frame_layouts: vec![InheritedFdFrameLayout {
                offset: payload_offset,
                byte_len,
                hash_offset: 48,
            }],
        })
    }

    pub(crate) fn output_descriptor(&self) -> Result<String, String> {
        let layout = self
            .frame_layouts
            .first()
            .filter(|_| self.frame_layouts.len() == 1)
            .ok_or_else(|| "provider output carrier requires one frame".to_owned())?;
        Ok(format!(
            "fd:{}:{}:{}:{}",
            self.file.as_raw_fd(),
            layout.offset,
            layout.byte_len,
            layout.hash_offset
        ))
    }

    pub(crate) fn verify_written_output(
        mut self,
        expected_hash: u64,
    ) -> Result<(MappedInheritedFdFrame, Self), String> {
        let layout = self
            .frame_layouts
            .first()
            .filter(|_| self.frame_layouts.len() == 1)
            .ok_or_else(|| "provider output carrier requires one frame".to_owned())?;
        let mapping = unsafe {
            mmap(
                std::ptr::null_mut(),
                self.packet_len,
                PROT_READ,
                MAP_PRIVATE,
                self.file.as_raw_fd(),
                0,
            )
        };
        if mapping as isize == -1 {
            return Err(format!(
                "failed to map provider output carrier: {}",
                std::io::Error::last_os_error()
            ));
        }
        let frame = MappedInheritedFdFrame {
            mapping,
            packet_len: self.packet_len,
            payload_offset: layout.offset,
            payload_len: layout.byte_len,
        };
        let stored_hash = frame.packet_bytes()[layout.hash_offset..layout.hash_offset + 8]
            .try_into()
            .expect("validated inherited frame hash slot");
        if fnv1a64(frame.as_bytes()) != expected_hash
            || u64::from_le_bytes(stored_hash) != expected_hash
        {
            return Err("provider output carrier hash mismatch".to_owned());
        }
        self.packet_hash = fnv1a64(frame.packet_bytes());
        Ok((frame, self))
    }

    pub(crate) fn frame_argument(&self, frame_index: usize) -> String {
        format!(
            "fd:{}:{frame_index}:{}:{}",
            self.file.as_raw_fd(),
            self.packet_len,
            self.packet_hash
        )
    }

    pub(crate) fn try_clone(&self) -> Result<Self, String> {
        Ok(Self {
            file: self
                .file
                .try_clone()
                .map_err(|error| format!("failed to clone inherited-fd carrier: {error}"))?,
            packet_len: self.packet_len,
            packet_hash: self.packet_hash,
            frame_layouts: self.frame_layouts.clone(),
        })
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

struct EncodedInheritedFdFrames {
    bytes: Vec<u8>,
    layouts: Vec<InheritedFdFrameLayout>,
}

fn encode_inherited_fd_frames(frames: &[&[u8]]) -> Result<EncodedInheritedFdFrames, String> {
    let page_size = system_page_size()?;
    let frame_count = u32::try_from(frames.len())
        .map_err(|_| "inherited-fd carrier has too many frames".to_owned())?;
    let records_len = frames
        .len()
        .checked_mul(INHERITED_FD_FRAME_RECORD_LEN)
        .ok_or_else(|| "inherited-fd carrier record length overflow".to_owned())?;
    let header_len = INHERITED_FD_HEADER_LEN
        .checked_add(records_len)
        .ok_or_else(|| "inherited-fd carrier header length overflow".to_owned())?;
    let mut next_offset = align_up(header_len, page_size)?;
    let mut raw_layouts = Vec::with_capacity(frames.len());
    for payload in frames {
        let mapped_len = align_up(payload.len().max(1), page_size)?;
        raw_layouts.push((next_offset, payload.len(), mapped_len, fnv1a64(payload)));
        next_offset = next_offset
            .checked_add(mapped_len)
            .ok_or_else(|| "inherited-fd carrier length overflow".to_owned())?;
    }
    let mut out = vec![0; next_offset];
    out[..8].copy_from_slice(INHERITED_FD_MAGIC);
    out[8..12].copy_from_slice(&frame_count.to_le_bytes());
    out[12..16].copy_from_slice(
        &u32::try_from(page_size)
            .map_err(|_| "inherited-fd page size overflow".to_owned())?
            .to_le_bytes(),
    );
    let mut layouts = Vec::with_capacity(frames.len());
    for (index, (payload, layout)) in frames.iter().zip(&raw_layouts).enumerate() {
        let (offset, byte_len, mapped_len, payload_hash) = *layout;
        let cursor = INHERITED_FD_HEADER_LEN + index * INHERITED_FD_FRAME_RECORD_LEN;
        out[cursor..cursor + 4].copy_from_slice(
            &u32::try_from(index)
                .map_err(|_| "inherited-fd frame index overflow".to_owned())?
                .to_le_bytes(),
        );
        out[cursor + 8..cursor + 16].copy_from_slice(&(offset as u64).to_le_bytes());
        out[cursor + 16..cursor + 24].copy_from_slice(&(byte_len as u64).to_le_bytes());
        out[cursor + 24..cursor + 32].copy_from_slice(&(mapped_len as u64).to_le_bytes());
        out[cursor + 32..cursor + 40].copy_from_slice(&payload_hash.to_le_bytes());
        out[offset..offset + byte_len].copy_from_slice(payload);
        layouts.push(InheritedFdFrameLayout {
            offset,
            byte_len,
            hash_offset: cursor + 32,
        });
    }
    Ok(EncodedInheritedFdFrames {
        bytes: out,
        layouts,
    })
}

fn system_page_size() -> Result<usize, String> {
    usize::try_from(unsafe { getpagesize() })
        .ok()
        .filter(|size| size.is_power_of_two())
        .ok_or_else(|| "inherited-fd carrier page size is invalid".to_owned())
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "inherited-fd carrier alignment overflow".to_owned())
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
        let packet_len = fields[3].parse::<usize>().expect("packet length");
        let page_size = unsafe { getpagesize() } as usize;
        assert_eq!(packet_len % page_size, 0);
        assert!(fields[4].parse::<u64>().is_ok());
        let mut magic = [0; 8];
        std::os::unix::fs::FileExt::read_at(&carrier.file, &mut magic, 0).expect("magic");
        assert_eq!(&magic, INHERITED_FD_MAGIC);
        let flags = unsafe { fcntl(carrier.file.as_raw_fd(), F_GETFD) };
        assert_ne!(flags & FD_CLOEXEC, 0);
    }

    #[test]
    fn verifies_written_output_as_a_borrowed_mapping() {
        let carrier = InheritedFdCarrier::new_writable_single_frame(4).expect("carrier");
        let layout = carrier.frame_layouts[0].clone();
        let payload = [1, 2, 3, 4];
        let hash = fnv1a64(&payload);
        carrier
            .file
            .write_all_at(&payload, layout.offset as u64)
            .expect("payload");
        carrier
            .file
            .write_all_at(&hash.to_le_bytes(), layout.hash_offset as u64)
            .expect("hash");
        let (mapped, sealed) = carrier
            .verify_written_output(hash)
            .expect("verified output");
        assert_eq!(mapped.as_bytes(), payload);
        assert_eq!(
            sealed.frame_argument(0).split(':').count(),
            5,
            "sealed output remains transferable"
        );
    }
}
