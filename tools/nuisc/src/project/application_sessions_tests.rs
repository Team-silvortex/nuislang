use super::*;

const ENTRY: &str = "image open=App.open event=draw close=App.close state=state";

#[test]
fn resource_registration_manifest_is_bounded_and_strict() {
    const RESOURCE: &str = "bytes open=start event=step close=stop min=1 max=4096";
    let manifest = |s: &str| format!("resource_sessions = [\"{s}\"]");
    let good = manifest(RESOURCE);
    let registrations = parse_resource_sessions(&good).unwrap();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].max_words, 4096);
    let ordered = parse_resource_sessions("resource_sessions = [\n# bounded\n\"bytes max=4096 close=stop min=1 event=step open=start\",\n] # end\n").unwrap();
    assert_eq!(ordered, registrations);
    let mixed = parse_resource_sessions(&manifest(&format!("{RESOURCE} state=Canvas"))).unwrap();
    assert_eq!(mixed[0].state_type.as_deref(), Some("Canvas"));
    assert_eq!(
        mixed[0].contract,
        yir_core::native_resource_session::MIXED_CONTRACT
    );
    let checked =
        parse_resource_sessions(&manifest(&format!("{RESOURCE} state=Canvas result=Result")))
            .unwrap();
    assert_eq!(checked[0].result_type.as_deref(), Some("Result"));
    assert_eq!(
        checked[0].contract,
        yir_core::native_resource_session::CHECKED_CONTRACT
    );
    assert_eq!(
        yir_core::native_resource_session::ResourceSessionRegistration::from_fields(
            &checked[0]
                .canonical()
                .split_whitespace()
                .collect::<Vec<_>>()
        )
        .unwrap(),
        checked[0]
    );
    for invalid in [
        format!("{RESOURCE} state="),
        format!("{RESOURCE} state=Canvas state=Other"),
        format!("{RESOURCE} result=Result"),
        format!("{RESOURCE} state=Canvas result="),
        format!("{RESOURCE} state=Canvas result=Canvas"),
        format!("{RESOURCE} state=Canvas result=Result result=Other"),
    ] {
        assert!(parse_resource_sessions(&manifest(&invalid)).is_err());
    }
    assert!(parse_resource_sessions("name = \"legacy\"")
        .unwrap()
        .is_empty());
    for bad in [
        RESOURCE.replace(" min=1", ""),
        RESOURCE.replace("min=1", "min=01"),
        RESOURCE.replace("max=4096", "max=4097"),
        RESOURCE.replace("max=4096", "max=0"),
        RESOURCE.replace("min=1", "min=4097"),
        RESOURCE.replace("event=step", "event=start"),
        format!("{RESOURCE} min=1"),
        format!("{RESOURCE} provider=metal"),
    ] {
        assert!(parse_resource_sessions(&manifest(&bad)).is_err(), "{bad}");
    }
    for bad in [
        good.clone() + " trailing",
        good.clone() + "\nresource_sessions=[]",
        format!("resource_sessions=[\"{RESOURCE}\",\"{RESOURCE}\"]"),
        "resource_sessions=[3]".to_owned(),
    ] {
        assert!(parse_resource_sessions(&bad).is_err(), "{bad}");
    }
}

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
