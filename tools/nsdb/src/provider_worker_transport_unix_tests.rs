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
fn output_descriptor_receipt_rejects_hash_mismatch() {
    let file = File::open("Cargo.toml").expect("file");
    let descriptors = vec![OwnedFd::from(file)];
    let error = verify_output_descriptors(
        &descriptors,
        &[1],
        &["0x0000000000000000".to_owned()],
        &["protocol-stdout".to_owned()],
    )
    .expect_err("forged output hash");
    assert!(error.contains("output descriptor hash mismatch"));
}

#[test]
fn nuis_worker_negotiates_three_inputs_control_and_two_output_fan_out() {
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
    for (path, byte) in [
        (&paths.first, 17u8),
        (&paths.second, 29),
        (&paths.third, 31),
        (&paths.control, 43),
    ] {
        fs::write(path, [byte]).expect("probe file");
    }
    let first = File::open(&paths.first).expect("first file");
    let second = File::open(&paths.second).expect("second file");
    let third = File::open(&paths.third).expect("third file");
    let control = File::open(&paths.control).expect("control file");
    let mut command = cached_image.command();
    let mut worker = UnixWorkerProcessTransport::spawn(
        &mut command,
        "lease:persistent",
        cached_image.registration.descriptor_capability,
        cached_image.registration.output_descriptor_capability,
    )
    .expect("worker");
    let descriptors = [
        UnixWorkerDescriptor {
            role: "input.0",
            descriptor: first.as_fd(),
        },
        UnixWorkerDescriptor {
            role: "input.1",
            descriptor: second.as_fd(),
        },
        UnixWorkerDescriptor {
            role: "input.2",
            descriptor: third.as_fd(),
        },
        UnixWorkerDescriptor {
            role: "control.adapter",
            descriptor: control.as_fd(),
        },
    ];
    let payload =
        b"capsule_token=capsule-token:101\ninvoker_token=invoker-token:301\ninput_roles=input.0,input.1,input.2\noutput_roles=output.primary,output.audit\ninputs=3\noutputs=2\n";
    let reply = worker
        .request("fan-in", payload, &descriptors)
        .expect("fan-in request");
    assert_eq!(reply.sequence, 0);
    assert_eq!(reply.descriptor_count, 4);
    assert_eq!(reply.first_byte_sum, 77);
    assert_eq!(reply.dispatch_status, 1);
    assert_eq!(
        reply.descriptor_roles,
        ["input.0", "input.1", "input.2", "control.adapter"]
    );
    assert_eq!(
        reply.output_descriptor_roles,
        ["output.primary", "output.audit"]
    );
    assert_eq!(reply.output_descriptors.len(), 2);
    assert_eq!(reply.output_descriptor_byte_lengths, [24, 24]);
    assert_eq!(
        reply.output_descriptor_modes,
        ["protocol-stdout", "protocol-stdout"]
    );
    assert_eq!(reply.output_descriptor_payloads.len(), 2);
    assert_ne!(
        reply.output_descriptor_hashes[0],
        reply.output_descriptor_hashes[1]
    );
    assert_eq!(
        reply.payload_hash,
        crate::provider_sample_artifact::fnv1a64_hex(payload)
    );
    assert_eq!(worker.close().expect("close"), reply.worker_pid);
}

struct WorkerProbePaths {
    output_dir: PathBuf,
    cache_output_dir: PathBuf,
    first: PathBuf,
    second: PathBuf,
    third: PathBuf,
    control: PathBuf,
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
            third: temp.join(format!("{stem}.third")),
            control: temp.join(format!("{stem}.control")),
        }
    }
}

impl Drop for WorkerProbePaths {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.output_dir);
        let _ = fs::remove_dir_all(&self.cache_output_dir);
        for path in [&self.first, &self.second, &self.third, &self.control] {
            let _ = fs::remove_file(path);
        }
    }
}
