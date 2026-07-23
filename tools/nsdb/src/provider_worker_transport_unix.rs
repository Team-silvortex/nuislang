use crate::provider_worker_request::{
    decode_provider_worker_reply, decode_provider_worker_request, encode_provider_worker_request,
    render_role_manifest, validate_frame_token, MAX_PROVIDER_WORKER_DESCRIPTORS,
};
use std::{
    mem::{size_of, zeroed},
    os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
    os::unix::net::UnixDatagram,
    os::unix::process::CommandExt,
    process::{Child, Command},
    time::Duration,
};

const MAX_FRAME_BYTES: usize = 64 * 1024;
const WORKER_IO_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct UnixWorkerProcessTransport {
    child: Child,
    socket: UnixDatagram,
    lease_id: String,
    next_sequence: usize,
    worker_pid: u32,
    closed: bool,
}

pub(crate) struct UnixWorkerProcessReply {
    pub(crate) sequence: usize,
    pub(crate) request_id: String,
    pub(crate) worker_pid: u32,
    pub(crate) descriptor_count: usize,
    pub(crate) first_byte_sum: u32,
    pub(crate) dispatch_status: i64,
    pub(crate) payload_hash: String,
    pub(crate) descriptor_roles: Vec<String>,
    pub(crate) payload: Vec<u8>,
}

pub(crate) struct UnixWorkerRequest {
    pub(crate) lease_id: String,
    pub(crate) sequence: usize,
    pub(crate) request_id: String,
    pub(crate) payload: Vec<u8>,
    pub(crate) payload_hash: String,
    pub(crate) descriptor_roles: Vec<String>,
    pub(crate) descriptors: Vec<OwnedFd>,
}

pub(crate) struct UnixWorkerDescriptor<'a> {
    pub(crate) role: &'a str,
    pub(crate) descriptor: BorrowedFd<'a>,
}

pub(crate) fn worker_socket_pair() -> Result<(UnixDatagram, UnixDatagram), String> {
    UnixDatagram::pair()
        .map_err(|error| format!("failed to create provider worker socket: {error}"))
}

impl UnixWorkerProcessTransport {
    pub(crate) fn spawn(command: &mut Command, lease_id: &str) -> Result<Self, String> {
        validate_frame_token(lease_id, "lease id")?;
        let (parent_socket, child_socket) = worker_socket_pair()?;
        parent_socket
            .set_read_timeout(Some(WORKER_IO_TIMEOUT))
            .map_err(|error| format!("failed to set provider worker read timeout: {error}"))?;
        parent_socket
            .set_write_timeout(Some(WORKER_IO_TIMEOUT))
            .map_err(|error| format!("failed to set provider worker write timeout: {error}"))?;
        let child_fd = child_socket.as_raw_fd();
        command.env("NUIS_PROVIDER_WORKER_SOCKET_FD", child_fd.to_string());
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(child_fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(child_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("failed to spawn Unix provider worker: {error}"))?;
        drop(child_socket);
        let handshake_result = (|| {
            let sent = parent_socket.send(b"NUISPWUH0").map_err(|error| {
                format!("failed to start Unix provider worker handshake: {error}")
            })?;
            if sent != b"NUISPWUH0".len() {
                return Err("Unix provider worker handshake request was truncated".to_owned());
            }
            let handshake = receive_text(&parent_socket)?;
            let fields = handshake.split('\t').collect::<Vec<_>>();
            if fields.len() != 2 || fields[0] != "NUISPWUH1" {
                return Err("Unix provider worker handshake is invalid".to_owned());
            }
            let worker_pid = fields[1]
                .parse::<u32>()
                .map_err(|error| format!("Unix provider worker pid is invalid: {error}"))?;
            if worker_pid != child.id() {
                return Err("Unix provider worker handshake pid mismatch".to_owned());
            }
            Ok(worker_pid)
        })();
        let worker_pid = match handshake_result {
            Ok(worker_pid) => worker_pid,
            Err(error) => {
                let exited = child
                    .try_wait()
                    .ok()
                    .flatten()
                    .map(|status| format!("; child exited with status {status}"))
                    .unwrap_or_default();
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{error}{exited}"));
            }
        };
        Ok(Self {
            child,
            socket: parent_socket,
            lease_id: lease_id.to_owned(),
            next_sequence: 0,
            worker_pid,
            closed: false,
        })
    }

    pub(crate) fn request(
        &mut self,
        request_id: &str,
        payload: &[u8],
        descriptors: &[UnixWorkerDescriptor<'_>],
    ) -> Result<UnixWorkerProcessReply, String> {
        if self.closed {
            return Err("Unix provider worker is closed".to_owned());
        }
        let sequence = self.next_sequence;
        if let Err(error) = send_worker_request(
            &self.socket,
            &self.lease_id,
            sequence,
            request_id,
            payload,
            descriptors,
        ) {
            let exited = self
                .child
                .try_wait()
                .ok()
                .flatten()
                .map(|status| format!("; child exited with status {status}"))
                .unwrap_or_default();
            return Err(format!("{error}{exited}"));
        }
        let descriptor_roles = descriptors
            .iter()
            .map(|descriptor| descriptor.role)
            .collect::<Vec<_>>();
        let reply = receive_process_reply(
            &self.socket,
            &self.lease_id,
            sequence,
            request_id,
            self.worker_pid,
            payload,
            &descriptor_roles,
        )?;
        if reply.descriptor_count != descriptors.len() {
            return Err("Unix provider worker receipt descriptor count mismatch".to_owned());
        }
        if request_id != "__close__" {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| format!("failed to inspect Unix provider worker: {error}"))?
            {
                return Err(format!(
                    "Unix provider worker exited with status {status} after replying to `{request_id}`"
                ));
            }
        }
        self.next_sequence += 1;
        Ok(reply)
    }

    pub(crate) fn close(mut self) -> Result<u32, String> {
        let reply = self.request("__close__", &[], &[])?;
        let status = self
            .child
            .wait()
            .map_err(|error| format!("failed to wait for Unix provider worker: {error}"))?;
        if !status.success() {
            return Err(format!("Unix provider worker exited with status {status}"));
        }
        self.closed = true;
        Ok(reply.worker_pid)
    }
}

impl Drop for UnixWorkerProcessTransport {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

pub(crate) fn send_worker_request(
    socket: &UnixDatagram,
    lease_id: &str,
    sequence: usize,
    request_id: &str,
    payload: &[u8],
    descriptors: &[UnixWorkerDescriptor<'_>],
) -> Result<(), String> {
    let descriptor_roles = descriptors
        .iter()
        .map(|descriptor| descriptor.role)
        .collect::<Vec<_>>();
    let frame =
        encode_provider_worker_request(lease_id, sequence, request_id, payload, &descriptor_roles)?;
    let mut iov = libc::iovec {
        iov_base: frame.as_ptr().cast_mut().cast(),
        iov_len: frame.len(),
    };
    let mut control = descriptor_control_buffer(descriptors.len());
    let mut message = unsafe { zeroed::<libc::msghdr>() };
    message.msg_iov = &mut iov;
    message.msg_iovlen = 1;
    if !descriptors.is_empty() {
        message.msg_control = control.as_mut_ptr().cast();
        message.msg_controllen = control
            .len()
            .try_into()
            .map_err(|_| "provider worker control length overflow")?;
        let header = unsafe { libc::CMSG_FIRSTHDR(&message) };
        if header.is_null() {
            return Err("provider worker descriptor header is unavailable".to_owned());
        }
        unsafe {
            (*header).cmsg_level = libc::SOL_SOCKET;
            (*header).cmsg_type = libc::SCM_RIGHTS;
            (*header).cmsg_len = libc::CMSG_LEN(
                (descriptors.len() * size_of::<RawFd>())
                    .try_into()
                    .map_err(|_| "provider worker descriptor length overflow")?,
            ) as _;
            let raw = libc::CMSG_DATA(header).cast::<RawFd>();
            for (index, descriptor) in descriptors.iter().enumerate() {
                raw.add(index).write(descriptor.descriptor.as_raw_fd());
            }
        }
    }
    let sent = unsafe { libc::sendmsg(socket.as_raw_fd(), &message, 0) };
    if sent < 0 {
        return Err(format!(
            "failed to send provider worker request: {}",
            std::io::Error::last_os_error()
        ));
    }
    if sent as usize != frame.len() {
        return Err("provider worker request was partially sent".to_owned());
    }
    Ok(())
}

pub(crate) fn receive_worker_request(
    socket: &UnixDatagram,
    expected_lease_id: &str,
    expected_sequence: usize,
    expected_request_id: &str,
) -> Result<UnixWorkerRequest, String> {
    let request = receive_unchecked(socket)?;
    validate_received_request(
        request,
        expected_lease_id,
        expected_sequence,
        expected_request_id,
    )
}

fn validate_received_request(
    request: UnixWorkerRequest,
    expected_lease_id: &str,
    expected_sequence: usize,
    expected_request_id: &str,
) -> Result<UnixWorkerRequest, String> {
    if request.lease_id != expected_lease_id
        || request.sequence != expected_sequence
        || request.request_id != expected_request_id
    {
        return Err("provider worker descriptor frame identity mismatch".to_owned());
    }
    Ok(request)
}

fn receive_unchecked(socket: &UnixDatagram) -> Result<UnixWorkerRequest, String> {
    let mut frame = vec![0u8; MAX_FRAME_BYTES];
    let mut iov = libc::iovec {
        iov_base: frame.as_mut_ptr().cast(),
        iov_len: frame.len(),
    };
    let mut control = descriptor_control_buffer(MAX_PROVIDER_WORKER_DESCRIPTORS);
    let mut message = unsafe { zeroed::<libc::msghdr>() };
    message.msg_iov = &mut iov;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = control
        .len()
        .try_into()
        .map_err(|_| "provider worker control length overflow")?;
    let received = unsafe { libc::recvmsg(socket.as_raw_fd(), &mut message, 0) };
    if received < 0 {
        return Err(format!(
            "failed to receive provider worker request: {}",
            std::io::Error::last_os_error()
        ));
    }
    if message.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0 {
        return Err("provider worker request or descriptors were truncated".to_owned());
    }
    let mut descriptors = receive_descriptors(&message)?;
    let envelope = decode_provider_worker_request(&frame[..received as usize], descriptors.len())?;
    for descriptor in &descriptors {
        set_close_on_exec(descriptor.as_raw_fd())?;
    }
    Ok(UnixWorkerRequest {
        lease_id: envelope.lease_id,
        sequence: envelope.sequence,
        request_id: envelope.request_id,
        payload: envelope.payload,
        payload_hash: envelope.payload_hash,
        descriptor_roles: envelope.descriptor_roles,
        descriptors: std::mem::take(&mut descriptors),
    })
}

fn receive_descriptors(message: &libc::msghdr) -> Result<Vec<OwnedFd>, String> {
    let mut descriptors = Vec::new();
    let mut header = unsafe { libc::CMSG_FIRSTHDR(message) };
    while !header.is_null() {
        unsafe {
            if (*header).cmsg_level != libc::SOL_SOCKET || (*header).cmsg_type != libc::SCM_RIGHTS {
                return Err("provider worker returned unsupported ancillary data".to_owned());
            }
            let base_len = libc::CMSG_LEN(0) as usize;
            if (*header).cmsg_len < base_len as _ {
                return Err("provider worker descriptor header is truncated".to_owned());
            }
            let byte_len = (*header).cmsg_len as usize - base_len;
            if byte_len % size_of::<RawFd>() != 0 {
                return Err("provider worker descriptor payload is misaligned".to_owned());
            }
            let count = byte_len / size_of::<RawFd>();
            if descriptors.len() + count > MAX_PROVIDER_WORKER_DESCRIPTORS {
                return Err("provider worker returned too many descriptors".to_owned());
            }
            let raw = libc::CMSG_DATA(header).cast::<RawFd>();
            for index in 0..count {
                descriptors.push(OwnedFd::from_raw_fd(raw.add(index).read()));
            }
            header = libc::CMSG_NXTHDR(message, header);
        }
    }
    Ok(descriptors)
}

fn set_close_on_exec(fd: RawFd) -> Result<(), String> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(format!(
            "failed to protect provider worker descriptor: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn receive_process_reply(
    socket: &UnixDatagram,
    expected_lease_id: &str,
    expected_sequence: usize,
    expected_request_id: &str,
    expected_worker_pid: u32,
    expected_payload: &[u8],
    expected_descriptor_roles: &[&str],
) -> Result<UnixWorkerProcessReply, String> {
    let packet = receive_packet(socket)?;
    let envelope = decode_provider_worker_reply(&packet)?;
    let expected_payload_hash = crate::provider_sample_artifact::fnv1a64_hex(expected_payload);
    if envelope.lease_id != expected_lease_id
        || envelope.sequence != expected_sequence
        || envelope.request_id != expected_request_id
        || envelope.worker_pid != expected_worker_pid
        || envelope.request_payload_hash != expected_payload_hash
        || render_role_manifest(
            &envelope
                .descriptor_roles
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        ) != render_role_manifest(expected_descriptor_roles)
    {
        return Err("Unix provider worker receipt identity mismatch".to_owned());
    }
    Ok(UnixWorkerProcessReply {
        sequence: expected_sequence,
        request_id: expected_request_id.to_owned(),
        worker_pid: expected_worker_pid,
        descriptor_count: envelope.descriptor_count,
        first_byte_sum: envelope.first_byte_sum,
        dispatch_status: envelope.dispatch_status,
        payload_hash: envelope.payload_hash,
        descriptor_roles: envelope.descriptor_roles,
        payload: envelope.payload,
    })
}

fn receive_packet(socket: &UnixDatagram) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0u8; MAX_FRAME_BYTES];
    let received = socket
        .recv(&mut bytes)
        .map_err(|error| format!("failed to receive Unix provider worker packet: {error}"))?;
    bytes.truncate(received);
    Ok(bytes)
}

fn receive_text(socket: &UnixDatagram) -> Result<String, String> {
    let bytes = receive_packet(socket)?;
    std::str::from_utf8(&bytes)
        .map(str::to_owned)
        .map_err(|_| "Unix provider worker receipt is not UTF-8".to_owned())
}

fn descriptor_control_buffer(count: usize) -> Vec<u8> {
    let byte_len = count * size_of::<RawFd>();
    vec![0; unsafe { libc::CMSG_SPACE(byte_len.try_into().unwrap_or(u32::MAX)) } as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self, File},
        io::Read,
        os::fd::AsFd,
        path::PathBuf,
        time::SystemTime,
    };

    #[test]
    fn rights_frame_binds_identity_count_and_cloexec() {
        let (sender, receiver) = worker_socket_pair().expect("pair");
        let file = File::open("Cargo.toml").expect("file");
        let descriptors = [UnixWorkerDescriptor {
            role: "input.primary",
            descriptor: file.as_fd(),
        }];
        send_worker_request(
            &sender,
            "lease:test",
            2,
            "request.test",
            &[0, b'\n', 0xff],
            &descriptors,
        )
        .expect("send");
        let request =
            receive_worker_request(&receiver, "lease:test", 2, "request.test").expect("receive");
        assert_eq!(request.descriptors.len(), 1);
        assert_eq!(request.payload, [0, b'\n', 0xff]);
        assert_eq!(request.descriptor_roles, ["input.primary"]);
        assert_ne!(
            unsafe { libc::fcntl(request.descriptors[0].as_raw_fd(), libc::F_GETFD) }
                & libc::FD_CLOEXEC,
            0
        );
        let mut text = String::new();
        File::from(request.descriptors.into_iter().next().unwrap())
            .read_to_string(&mut text)
            .expect("read");
        assert!(text.contains("[workspace]") || text.contains("[package]"));
    }

    #[test]
    fn identity_mismatch_closes_received_descriptors() {
        let (sender, receiver) = worker_socket_pair().expect("pair");
        let file = File::open("Cargo.toml").expect("file");
        let descriptors = [UnixWorkerDescriptor {
            role: "input.primary",
            descriptor: file.as_fd(),
        }];
        send_worker_request(
            &sender,
            "lease:test",
            0,
            "request.test",
            b"identity",
            &descriptors,
        )
        .expect("send");
        let request = receive_unchecked(&receiver).expect("receive");
        let received_fd = request.descriptors[0].as_raw_fd();
        assert!(validate_received_request(request, "lease:other", 0, "request.test").is_err());
        assert_eq!(unsafe { libc::fcntl(received_fd, libc::F_GETFD) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EBADF)
        );
    }

    #[test]
    fn nuis_worker_image_receives_two_distinct_post_spawn_descriptors() {
        let paths = WorkerProbePaths::new();
        let first_image = crate::provider_worker_image::resolve_provider_worker_image(
            "coreml:apple-ane",
            &paths.output_dir,
        )
        .expect("resolve Nuis worker");
        let cached_image = crate::provider_worker_image::resolve_provider_worker_image(
            "coreml:apple-ane",
            &paths.cache_output_dir,
        )
        .expect("restore Nuis worker");
        assert_eq!(cached_image.cache_status, "hit");
        assert_eq!(first_image.cache_key, cached_image.cache_key);
        assert_eq!(
            cached_image.resolver_contract,
            crate::provider_worker_image::PROVIDER_WORKER_IMAGE_RESOLVER_CONTRACT
        );
        fs::write(&paths.first, [17u8]).expect("first");
        fs::write(&paths.second, [29u8]).expect("second");
        let first = File::open(&paths.first).expect("first file");
        let second = File::open(&paths.second).expect("second file");
        let mut command = cached_image.command();
        let mut worker =
            UnixWorkerProcessTransport::spawn(&mut command, "lease:persistent").expect("worker");
        let first_descriptors = [UnixWorkerDescriptor {
            role: "input.primary",
            descriptor: first.as_fd(),
        }];
        let first_payload = b"capsule_token=capsule-token:101\ninputs=1\noutputs=1\n";
        let first_reply = worker
            .request("first", first_payload, &first_descriptors)
            .expect("first request");
        let second_descriptors = [UnixWorkerDescriptor {
            role: "output.result",
            descriptor: second.as_fd(),
        }];
        let second_payload = b"capsule_token=capsule-token:202\ninputs=1\noutputs=1\n";
        let second_reply = worker
            .request("second", second_payload, &second_descriptors)
            .expect("second request");
        assert_eq!((first_reply.sequence, second_reply.sequence), (0, 1));
        assert_eq!(
            (
                first_reply.request_id.as_str(),
                second_reply.request_id.as_str()
            ),
            ("first", "second")
        );
        assert_eq!(first_reply.worker_pid, second_reply.worker_pid);
        assert_eq!(
            (first_reply.first_byte_sum, second_reply.first_byte_sum),
            (17, 29)
        );
        assert_eq!(
            (first_reply.dispatch_status, second_reply.dispatch_status),
            (1, 2)
        );
        assert_eq!(first_reply.descriptor_roles, ["input.primary"]);
        assert_eq!(second_reply.descriptor_roles, ["output.result"]);
        assert_eq!(
            first_reply.payload_hash,
            crate::provider_sample_artifact::fnv1a64_hex(first_payload)
        );
        assert_eq!(first_reply.payload, first_payload);
        assert_eq!(second_reply.payload, second_payload);
        assert_eq!(worker.close().expect("close"), first_reply.worker_pid);
    }

    struct WorkerProbePaths {
        output_dir: PathBuf,
        cache_output_dir: PathBuf,
        first: PathBuf,
        second: PathBuf,
    }

    impl WorkerProbePaths {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let stem = format!("nuis-provider-worker-{}-{nonce}", std::process::id());
            let temp = std::env::temp_dir();
            Self {
                output_dir: temp.join(format!("{stem}.build")),
                cache_output_dir: temp.join(format!("{stem}.cached-build")),
                first: temp.join(format!("{stem}.first")),
                second: temp.join(format!("{stem}.second")),
            }
        }
    }

    impl Drop for WorkerProbePaths {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.output_dir);
            let _ = fs::remove_dir_all(&self.cache_output_dir);
            for path in [&self.first, &self.second] {
                let _ = fs::remove_file(path);
            }
        }
    }
}
