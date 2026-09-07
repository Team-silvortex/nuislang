use super::*;
use std::{
    io::Write,
    net::Shutdown,
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread::{self, JoinHandle},
};
use yir_core::provider_runtime_ipc::{
    hash_bytes, DispatchArguments, DispatchTarget, DispatchUpload,
};

struct Peer {
    stream: UnixStream,
    replies: Receiver<Result<Message, String>>,
    worker: Option<JoinHandle<()>>,
}

impl Peer {
    fn new(requests: usize) -> Self {
        let (mut stream, peer) = UnixStream::pair().unwrap();
        let timeout = Duration::from_millis(100);
        configure_connection(&stream, timeout).unwrap();
        let (sender, replies) = mpsc::channel();
        let worker = thread::spawn(move || {
            for _ in 0..requests {
                let reply = read_request(&mut stream, timeout);
                let failed = reply.is_err();
                if sender.send(reply).is_err() || failed {
                    break;
                }
            }
        });
        Self {
            stream: peer,
            replies,
            worker: Some(worker),
        }
    }

    fn receive(&self) -> Result<Message, String> {
        self.replies
            .recv_timeout(Duration::from_secs(3))
            .expect("bounded request read")
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
        let _ = self.worker.take().unwrap().join();
    }
}

fn upload() -> Message {
    let mut arguments = DispatchArguments::parse("test.v1|count:u64:4").unwrap();
    arguments.uploads.insert(
        "input.0".to_owned(),
        DispatchUpload::new("u8", vec![4], vec![1, 2, 3, 4]).unwrap(),
    );
    Message::Dispatch {
        sequence: 0,
        target: DispatchTarget {
            source_yir_fnv1a64: hash_bytes(b"source"),
            module: "shader".to_owned(),
            instruction: "draw_instanced".to_owned(),
            node: "draw".to_owned(),
            resource: "gpu".to_owned(),
        },
        arguments,
    }
}

fn wire(message: &Message) -> Vec<u8> {
    let mut bytes = Vec::new();
    message.write_to(&mut bytes).unwrap();
    bytes
}

#[test]
fn idle_wait_survives_each_request_timeout_on_the_same_connection() {
    let mut peer = Peer::new(2);
    for sequence in [0, 1] {
        assert_eq!(
            peer.replies.recv_timeout(Duration::from_millis(250)),
            Err(RecvTimeoutError::Timeout)
        );
        let message = Message::Finish(sequence);
        message.write_to(&mut peer.stream).unwrap();
        assert_eq!(peer.receive().unwrap(), message);
    }
}

#[test]
fn buffered_requests_and_uploads_keep_the_existing_v3_framing() {
    let mut peer = Peer::new(2);
    let dispatch = upload();
    let mut bytes = wire(&dispatch);
    bytes.extend(wire(&Message::Finish(1)));
    peer.stream.write_all(&bytes).unwrap();
    assert_eq!(peer.receive().unwrap(), dispatch);
    assert_eq!(peer.receive().unwrap(), Message::Finish(1));
}

#[test]
fn partial_prefix_header_and_upload_fail_with_a_request_deadline() {
    let bytes = wire(&upload());
    let header = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
    for end in [1, 3, 4 + header - 1, bytes.len() - 1] {
        let mut peer = Peer::new(1);
        peer.stream.write_all(&bytes[..end]).unwrap();
        let error = peer.receive().unwrap_err();
        assert!(
            error.contains("request deadline exceeded"),
            "cut {end}: {error}"
        );
    }
}

#[test]
fn partial_progress_does_not_extend_the_request_deadline() {
    let (mut stream, mut peer) = UnixStream::pair().unwrap();
    let timeout = Duration::from_millis(100);
    configure_connection(&stream, timeout).unwrap();
    let deadline = Instant::now() + timeout;
    let mut reader = RequestReader {
        stream: &mut stream,
        deadline,
    };
    peer.write_all(b"a").unwrap();
    let mut byte = [0];
    reader.read_exact(&mut byte).unwrap();
    thread::sleep(Duration::from_millis(150));
    peer.write_all(b"b").unwrap();
    assert_eq!(
        reader.read_exact(&mut byte).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(reader.deadline, deadline);
    // Rejection did not consume queued bytes as a fresh request or reset a timer.
    stream.read_exact(&mut byte).unwrap();
    assert_eq!(&byte, b"b");
}

#[test]
fn peer_eof_wakes_idle_and_partial_reads_without_a_successful_message() {
    let bytes = wire(&upload());
    for end in [0, 1, bytes.len() - 1] {
        let mut peer = Peer::new(1);
        peer.stream.write_all(&bytes[..end]).unwrap();
        peer.stream.shutdown(Shutdown::Write).unwrap();
        let error = peer.receive().unwrap_err();
        assert!(error.contains("read failed"), "{error}");
        assert!(!error.contains("deadline exceeded"), "{error}");
    }
}

#[test]
fn idle_policy_still_rejects_invalid_framing_and_timeout_configuration() {
    let mut peer = Peer::new(1);
    peer.stream.write_all(&u32::MAX.to_le_bytes()).unwrap();
    assert!(peer.receive().unwrap_err().contains("header size"));
    assert!(read_request(&mut peer.stream, Duration::ZERO)
        .unwrap_err()
        .contains("positive"));
    assert!(configure_connection(&peer.stream, Duration::MAX)
        .unwrap_err()
        .contains("bounded"));
}
