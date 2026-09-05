use super::*;
use std::{
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
};

#[test]
fn accepted_connection_waits_for_delayed_dispatch_even_if_nonblocking_was_inherited() {
    let (mut stream, mut peer) = UnixStream::pair().unwrap();
    stream.set_nonblocking(true).unwrap();
    configure_connection(&stream).unwrap();
    let sender = thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        peer.write_all(b"dispatch")
    });
    let mut received = [0; 8];
    let result = stream.read_exact(&mut received);
    sender.join().unwrap().unwrap();
    result.unwrap();
    assert_eq!(&received, b"dispatch");
}

#[test]
fn unconnected_server_rejects_success_and_removes_private_socket() {
    let mut server = RuntimeProviderServer::start(Path::new(".")).unwrap();
    let directory = server.directory.clone();
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert!(directory.join("dispatch").exists());
    assert!(server
        .finish()
        .unwrap_err()
        .contains("did not complete a provider lifecycle"));
    drop(server);
    assert!(!directory.exists());
}
