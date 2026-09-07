use yir_core::APPLICATION_SESSION_CONTRACT;

#[test]
fn registrations_are_records_not_executable_nodes() {
    let source = format!("yir 0.1\napplication-session app {APPLICATION_SESSION_CONTRACT} start step stop state\nfunction start cpu helper\n");
    let module = yir_syntax::parse_explicit_module(&source).unwrap();
    assert!(module.nodes.is_empty());
    assert_eq!(module.application_sessions[0].open, "start");
    assert_eq!(module.application_sessions[0].id, "app");
    assert_eq!(
        yir_syntax::parse_module(&source)
            .unwrap()
            .application_sessions,
        module.application_sessions
    );
}

#[test]
fn malformed_duplicate_and_unknown_version_records_fail_closed() {
    let record =
        format!("application-session app {APPLICATION_SESSION_CONTRACT} start step stop state");
    for source in [
        format!("{record}\n{record}"),
        record.replace(APPLICATION_SESSION_CONTRACT, "unknown-v2"),
        format!("{record} extra"),
        record.replace("start step stop", "start start stop"),
    ] {
        assert!(
            yir_syntax::parse_explicit_module(&source).is_err(),
            "accepted {source}"
        );
    }
}
