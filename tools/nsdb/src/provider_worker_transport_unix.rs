use crate::provider_worker_descriptor_capability::{
    ProviderWorkerDescriptorCapability, ProviderWorkerOutputDescriptorCapability,
    PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT,
    PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT,
};
use crate::provider_worker_request::{
    decode_provider_worker_reply, decode_provider_worker_request, encode_provider_worker_request,
    render_role_manifest, validate_frame_token, MAX_PROVIDER_WORKER_DESCRIPTORS,
};
use std::{
    io::Read,
    mem::{size_of, zeroed},
    os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
    os::unix::net::UnixDatagram,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const MAX_FRAME_BYTES: usize = 64 * 1024;
const WORKER_IO_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) struct UnixWorkerProcessTransport {
    child: Child,
    socket: UnixDatagram,
    lease_id: String,
    next_sequence: usize,
    worker_pid: u32,
    descriptor_capability: ProviderWorkerDescriptorCapability,
    output_descriptor_capability: ProviderWorkerOutputDescriptorCapability,
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
    pub(crate) output_descriptors: Vec<OwnedFd>,
    pub(crate) output_descriptor_roles: Vec<String>,
    pub(crate) output_descriptor_byte_lengths: Vec<usize>,
    pub(crate) output_descriptor_hashes: Vec<String>,
    pub(crate) output_descriptor_modes: Vec<String>,
    pub(crate) output_descriptor_payloads: Vec<Vec<u8>>,
    pub(crate) adapter_protocol: Vec<u8>,
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
    pub(crate) fn spawn(
        command: &mut Command,
        lease_id: &str,
        descriptor_capability: ProviderWorkerDescriptorCapability,
        output_descriptor_capability: ProviderWorkerOutputDescriptorCapability,
    ) -> Result<Self, String> {
        validate_frame_token(lease_id, "lease id")?;
        descriptor_capability.validate()?;
        output_descriptor_capability.validate()?;
        let (parent_socket, child_socket) = worker_socket_pair()?;
        parent_socket
            .set_read_timeout(Some(WORKER_IO_TIMEOUT))
            .map_err(|error| format!("failed to set provider worker read timeout: {error}"))?;
        parent_socket
            .set_write_timeout(Some(WORKER_IO_TIMEOUT))
            .map_err(|error| format!("failed to set provider worker write timeout: {error}"))?;
        let child_fd = child_socket.as_raw_fd();
        command.env("NUIS_PROVIDER_WORKER_SOCKET_FD", child_fd.to_string());
        command.stderr(Stdio::piped());
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
            if fields.len() != 8
                || fields[0] != "NUISPWUH3"
                || fields[2] != PROVIDER_WORKER_DESCRIPTOR_CAPABILITY_CONTRACT
                || fields[6] != PROVIDER_WORKER_OUTPUT_DESCRIPTOR_CAPABILITY_CONTRACT
            {
                return Err("Unix provider worker handshake is invalid".to_owned());
            }
            let worker_pid = fields[1]
                .parse::<u32>()
                .map_err(|error| format!("Unix provider worker pid is invalid: {error}"))?;
            if worker_pid != child.id() {
                return Err("Unix provider worker handshake pid mismatch".to_owned());
            }
            let semantic_limit = fields[3].parse::<usize>().map_err(|error| {
                format!("Unix provider worker semantic descriptor limit is invalid: {error}")
            })?;
            let control_limit = fields[4].parse::<usize>().map_err(|error| {
                format!("Unix provider worker control descriptor limit is invalid: {error}")
            })?;
            let total_limit = fields[5].parse::<usize>().map_err(|error| {
                format!("Unix provider worker total descriptor limit is invalid: {error}")
            })?;
            if semantic_limit != descriptor_capability.max_semantic_descriptors
                || control_limit != descriptor_capability.max_control_descriptors
                || total_limit != descriptor_capability.total_limit()
            {
                return Err("Unix provider worker descriptor capability mismatch".to_owned());
            }
            let output_limit = fields[7].parse::<usize>().map_err(|error| {
                format!("Unix provider worker output descriptor limit is invalid: {error}")
            })?;
            if output_limit != output_descriptor_capability.max_output_descriptors {
                return Err("Unix provider worker output descriptor capability mismatch".to_owned());
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
            descriptor_capability,
            output_descriptor_capability,
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
        let descriptor_roles = descriptors
            .iter()
            .map(|descriptor| descriptor.role)
            .collect::<Vec<_>>();
        self.descriptor_capability
            .validate_roles(&descriptor_roles)?;
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
        wait_for_worker_reply(&self.socket, &mut self.child, request_id)?;
        let reply = receive_process_reply(
            &self.socket,
            &self.lease_id,
            sequence,
            request_id,
            self.worker_pid,
            payload,
            &descriptor_roles,
            self.output_descriptor_capability,
        )
        .map_err(|error| {
            let exited = self
                .child
                .try_wait()
                .ok()
                .flatten()
                .map(|status| format!("; worker exited with status {status}"))
                .unwrap_or_default();
            format!("{error}{exited}")
        })?;
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

fn wait_for_worker_reply(
    socket: &UnixDatagram,
    child: &mut Child,
    request_id: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + WORKER_IO_TIMEOUT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!(
                "Unix provider worker timed out before replying to `{request_id}`"
            ));
        }
        let timeout_ms = remaining.as_millis().min(250) as libc::c_int;
        let mut descriptor = libc::pollfd {
            fd: socket.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let status = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
        if status < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(format!(
                "failed to poll Unix provider worker reply: {error}"
            ));
        }
        if status > 0 {
            if descriptor.revents & libc::POLLIN != 0 {
                return Ok(());
            }
            return Err(format!(
                "Unix provider worker reply socket failed with events 0x{:x}",
                descriptor.revents
            ));
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to inspect Unix provider worker: {error}"))?
        {
            let mut diagnostic = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                let _ = stderr.read_to_string(&mut diagnostic);
            }
            let diagnostic = diagnostic.trim();
            return Err(format!(
                "Unix provider worker exited with status {status} before replying to `{request_id}`{}",
                if diagnostic.is_empty() {
                    String::new()
                } else {
                    format!(": {diagnostic}")
                }
            ));
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
    output_descriptor_capability: ProviderWorkerOutputDescriptorCapability,
) -> Result<UnixWorkerProcessReply, String> {
    let (packet, output_descriptors) = receive_reply_packet(socket)?;
    let envelope = decode_provider_worker_reply(&packet, output_descriptors.len())?;
    if envelope.output_descriptor_count > output_descriptor_capability.max_output_descriptors {
        return Err("Unix provider worker output descriptor capacity exceeded".to_owned());
    }
    let expected_payload_hash = crate::provider_sample_artifact::fnv1a64_hex(expected_payload);
    if envelope.lease_id != expected_lease_id
        || envelope.sequence != expected_sequence
        || envelope.request_id != expected_request_id
        || envelope.worker_pid != expected_worker_pid
        || envelope.request_payload_hash != expected_payload_hash
        || envelope.request_payload_length != expected_payload.len()
        || envelope.payload_hash != expected_payload_hash
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
    let output_descriptor_payloads = verify_output_descriptors(
        &output_descriptors,
        &envelope.output_descriptor_byte_lengths,
        &envelope.output_descriptor_hashes,
        &envelope.output_descriptor_modes,
    )?;
    Ok(UnixWorkerProcessReply {
        sequence: expected_sequence,
        request_id: expected_request_id.to_owned(),
        worker_pid: expected_worker_pid,
        descriptor_count: envelope.descriptor_count,
        first_byte_sum: envelope.first_byte_sum,
        dispatch_status: envelope.dispatch_status,
        payload_hash: envelope.payload_hash,
        descriptor_roles: envelope.descriptor_roles,
        output_descriptors,
        output_descriptor_roles: envelope.output_descriptor_roles,
        output_descriptor_byte_lengths: envelope.output_descriptor_byte_lengths,
        output_descriptor_hashes: envelope.output_descriptor_hashes,
        output_descriptor_modes: envelope.output_descriptor_modes,
        output_descriptor_payloads,
        adapter_protocol: envelope.adapter_protocol,
    })
}

fn receive_reply_packet(socket: &UnixDatagram) -> Result<(Vec<u8>, Vec<OwnedFd>), String> {
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
        .map_err(|_| "provider worker reply control length overflow")?;
    let received = unsafe { libc::recvmsg(socket.as_raw_fd(), &mut message, 0) };
    if received < 0 {
        return Err(format!(
            "failed to receive Unix provider worker reply: {}",
            std::io::Error::last_os_error()
        ));
    }
    if message.msg_flags & (libc::MSG_TRUNC | libc::MSG_CTRUNC) != 0 {
        return Err("provider worker reply or output descriptors were truncated".to_owned());
    }
    let descriptors = receive_descriptors(&message)?;
    for descriptor in &descriptors {
        set_close_on_exec(descriptor.as_raw_fd())?;
    }
    frame.truncate(received as usize);
    Ok((frame, descriptors))
}

fn verify_output_descriptors(
    descriptors: &[OwnedFd],
    byte_lengths: &[usize],
    expected_hashes: &[String],
    modes: &[String],
) -> Result<Vec<Vec<u8>>, String> {
    if descriptors.is_empty() {
        return (byte_lengths.is_empty() && expected_hashes.is_empty() && modes.is_empty())
            .then(Vec::new)
            .ok_or_else(|| "provider worker empty output receipt is inconsistent".to_owned());
    }
    if descriptors.len() != byte_lengths.len()
        || descriptors.len() != expected_hashes.len()
        || descriptors.len() != modes.len()
    {
        return Err("provider worker output descriptor metadata count mismatch".to_owned());
    }
    let mut payloads = Vec::with_capacity(descriptors.len());
    for (((descriptor, byte_length), expected_hash), mode) in descriptors
        .iter()
        .zip(byte_lengths)
        .zip(expected_hashes)
        .zip(modes)
    {
        if mode == "nuispfd1-result" {
            payloads.push(Vec::new());
            continue;
        }
        if mode != "protocol-stdout" || *byte_length == 0 || *byte_length > MAX_FRAME_BYTES {
            return Err("provider worker output descriptor mode is unsupported".to_owned());
        }
        let mut bytes = vec![0u8; *byte_length];
        let read = unsafe {
            libc::pread(
                descriptor.as_raw_fd(),
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                0,
            )
        };
        if read < 0 || read as usize != bytes.len() {
            return Err("provider worker output descriptor payload is unreadable".to_owned());
        }
        if crate::provider_sample_artifact::fnv1a64_hex(&bytes) != *expected_hash {
            return Err("provider worker output descriptor hash mismatch".to_owned());
        }
        payloads.push(bytes);
    }
    Ok(payloads)
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
#[path = "provider_worker_transport_unix_tests.rs"]
mod tests;
