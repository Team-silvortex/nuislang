use super::*;

#[test]
fn parses_galaxy_provider_trust_paths_as_a_pair() {
    let command = parse_args(
        [
            "galaxy".to_owned(),
            "resolve-deps".to_owned(),
            "project".to_owned(),
            "--provider-root".to_owned(),
            "mirror".to_owned(),
            "--trust-registry".to_owned(),
            "trust/registry.toml".to_owned(),
            "--trust-state".to_owned(),
            "state/provider.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("Galaxy provider trust paths parse");
    assert_eq!(
        command,
        CommandKind::Galaxy(GalaxyCommand::ResolveDeps {
            input: PathBuf::from("project"),
            provider_root: PathBuf::from("mirror"),
            provider_id: "offline.layout".to_owned(),
            provider_kind: "offline-layout".to_owned(),
            trust_registry: Some(PathBuf::from("trust/registry.toml")),
            trust_state: Some(PathBuf::from("state/provider.toml")),
        })
    );

    assert!(parse_args(
        [
            "galaxy".to_owned(),
            "resolve-deps".to_owned(),
            "--provider-root".to_owned(),
            "mirror".to_owned(),
            "--trust-registry".to_owned(),
            "trust/registry.toml".to_owned(),
        ]
        .into_iter(),
    )
    .is_err());
}
