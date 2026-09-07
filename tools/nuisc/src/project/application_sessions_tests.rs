use super::*;

const ENTRY: &str = "image open=App.open event=draw close=App.close state=state";

#[test]
fn declarations_accept_multiline_comments_and_named_field_order() {
    let source = format!("name = \"demo\"\napplication_sessions=[\n# explicit host selection\n\"{ENTRY}\",\n\"other state=state close=stop event=step open=start\",\n] # bindings\nentry=\"main.ns\"\n");
    let registrations = parse_application_sessions(&source).unwrap();
    assert_eq!(registrations.len(), 2);
    assert_eq!(registrations[0].event, "draw");
    assert_eq!(registrations[1].open, "start");
    assert!(parse_application_sessions("name = \"legacy\"\n")
        .unwrap()
        .is_empty());
    assert!(parse_application_sessions("application_sessions = []")
        .unwrap()
        .is_empty());
}

#[test]
fn malformed_declarations_are_not_silently_treated_as_absent() {
    for value in [
        format!("application_sessions = [\"{ENTRY}\""),
        format!("application_sessions = [\"{ENTRY}\"] trailing"),
        format!("application_sessions = [\"{ENTRY}\", \"{ENTRY}\"]"),
        format!("application_sessions = [\"{ENTRY}\"]\napplication_sessions = []"),
        format!("application_sessions = [\"{ENTRY}\" \"other\"]"),
        format!("application_sessions = [\"{ENTRY}\",,]"),
        "application_sessions = 3".to_owned(),
        "application_sessions = [3]".to_owned(),
        "application_sessions = [\"image open=start event=step close=stop\"]".to_owned(),
        format!("application_sessions = [\"{ENTRY} open=second\"]"),
        format!("application_sessions = [\"{ENTRY} provider=metal\"]"),
    ] {
        assert!(
            parse_application_sessions(&value).is_err(),
            "accepted {value}"
        );
    }
}
