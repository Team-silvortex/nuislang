use super::*;
use std::io::{Read, Write};

fn connection(sequence: usize) -> (ProviderRuntimeClient, UnixStream) {
    let (target, _, _) = fixture();
    let (stream, peer) = UnixStream::pair().unwrap();
    for stream in [&stream, &peer] {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
    }
    (
        ProviderRuntimeClient {
            stream,
            target,
            sequence,
            frontier: true,
        },
        peer,
    )
}

#[test]
fn drain_requires_exact_receipt_and_is_terminal_even_after_failure() {
    for case in 0..7 {
        let (mut client, mut peer) = connection(MAX_DISPATCHES);
        let drain = SessionDrain {
            sequence: MAX_DISPATCHES,
            target: client.target.clone(),
        };
        let worker = thread::spawn(move || {
            assert_eq!(
                Message::read_from(&mut peer).unwrap(),
                Message::Drain(drain.clone())
            );
            let reply = match case {
                0 => Message::Drained(drain),
                1 => Message::Drained(SessionDrain {
                    sequence: 0,
                    ..drain
                }),
                2 => {
                    let mut wrong = drain;
                    wrong.target.resource = "other".to_owned();
                    Message::Drained(wrong)
                }
                3 => Message::Closed(MAX_DISPATCHES),
                4 => Message::Rejected(Rejection::new(
                    RejectionPhase::Finish,
                    MAX_DISPATCHES,
                    RejectionCode::Finalization,
                    "wrong phase",
                )),
                5 => Message::Rejected(Rejection::new(
                    RejectionPhase::Drain,
                    MAX_DISPATCHES,
                    RejectionCode::Finalization,
                    "worker close failed",
                )),
                _ => Message::Rejected(Rejection::new(
                    RejectionPhase::Receive,
                    MAX_DISPATCHES,
                    RejectionCode::Exchange,
                    "legacy peer rejected extension",
                )),
            };
            reply.write_to(&mut peer).unwrap();
            assert_eq!(
                peer.read(&mut [0]).unwrap(),
                0,
                "terminal exchange was retried"
            );
        });
        let result = client.drain();
        if case == 0 {
            assert_eq!(result.unwrap(), Some(MAX_DISPATCHES));
        } else {
            let expected = match case {
                5 => ApplicationFailureKind::ProviderFinalization,
                6 => ApplicationFailureKind::ProviderExchange,
                _ => ApplicationFailureKind::ProviderContract,
            };
            assert_eq!(result.unwrap_err().kind, expected);
        }
        assert_eq!(client.drain().unwrap(), None);
        assert!(client.finish().is_err());
        drop(client);
        worker.join().unwrap();
    }
}

#[test]
fn damaged_dispatch_never_sends_drain_finish_or_another_dispatch() {
    for case in 0..4 {
        let (mut client, mut peer) = connection(0);
        let (_, node, mut frame) = fixture();
        let arguments = frame.arguments.clone();
        let worker = thread::spawn(move || {
            assert!(matches!(
                Message::read_from(&mut peer).unwrap(),
                Message::Dispatch { sequence: 0, .. }
            ));
            match case {
                0 => frame.sequence = 1,
                1 => {
                    frame.arguments.scalars.insert("count".to_owned(), 9);
                }
                2 => frame.shape = vec![2, 1],
                _ => {
                    peer.write_all(&[20, 0, 0, 0, b'x']).unwrap();
                    peer.shutdown(std::net::Shutdown::Write).unwrap();
                }
            }
            if case < 3 {
                Message::Frame(frame).write_to(&mut peer).unwrap();
            }
            assert_eq!(
                peer.read(&mut [0]).unwrap(),
                0,
                "damaged exchange was retried"
            );
        });
        assert!(client.take(&node, &arguments).is_err());
        assert_eq!(client.sequence, 0);
        assert_eq!(client.drain().unwrap(), None);
        assert!(client.finish().is_err());
        assert!(client.take(&node, &arguments).is_err());
        drop(client);
        worker.join().unwrap();
    }
}

#[test]
fn local_dispatch_limit_does_not_prevent_drain_at_the_validated_frontier() {
    let (mut client, mut peer) = connection(MAX_DISPATCHES);
    let (_, node, frame) = fixture();
    assert_eq!(
        client.take(&node, &frame.arguments).err().unwrap().kind,
        ApplicationFailureKind::DispatchLimit
    );
    let worker = thread::spawn(move || {
        let Message::Drain(drain) = Message::read_from(&mut peer).unwrap() else {
            panic!("expected drain")
        };
        assert_eq!(drain.sequence, MAX_DISPATCHES);
        Message::Drained(drain).write_to(&mut peer).unwrap();
    });
    assert_eq!(client.drain().unwrap(), Some(MAX_DISPATCHES));
    worker.join().unwrap();
}

#[test]
fn lost_drain_receipt_is_not_retirement_and_cannot_be_retried() {
    let (mut client, mut peer) = connection(0);
    let worker = thread::spawn(move || {
        assert!(matches!(
            Message::read_from(&mut peer).unwrap(),
            Message::Drain(_)
        ));
        peer.write_all(&[12, 0, 0, 0, b'x']).unwrap();
    });
    assert_eq!(
        client.drain().unwrap_err().kind,
        ApplicationFailureKind::ProviderExchange
    );
    assert_eq!(client.drain().unwrap(), None);
    assert!(client.finish().is_err());
    worker.join().unwrap();
}
