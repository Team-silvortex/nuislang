use super::*;
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

struct TempTrustRoot(PathBuf);

impl TempTrustRoot {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuis_galaxy_trust_{name}_{nonce}"));
        fs::create_dir_all(root.join("provider")).unwrap();
        fs::create_dir_all(root.join("trusted")).unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn provider(&self) -> GalaxyResolutionProviderDescriptor {
        GalaxyResolutionProviderDescriptor {
            provider_id: "fixture.offline".to_owned(),
            provider_kind: "offline-layout".to_owned(),
            root: self.0.join("provider"),
            trust_policy: Some(GalaxyResolutionProviderTrustPolicy {
                registry_path: self.0.join("trusted/registry.toml"),
                state_path: self.0.join("trusted/state.toml"),
            }),
        }
    }
}

impl Drop for TempTrustRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn signer(byte: char) -> String {
    format!("ed25519:sha256:{}", byte.to_string().repeat(64))
}

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn write_registry(
    provider: &GalaxyResolutionProviderDescriptor,
    generation: u64,
    signers: &[(&str, &str)],
) {
    let policy = provider.trust_policy.as_ref().unwrap();
    let mut entries = signers
        .iter()
        .map(|(signer_id, status)| TrustSigner {
            signer_id: (*signer_id).to_owned(),
            status: (*status).to_owned(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.signer_id.cmp(&right.signer_id));
    fs::write(
        &policy.registry_path,
        render_registry(&TrustRegistry {
            provider_id: provider.provider_id.clone(),
            provider_kind: provider.provider_kind.clone(),
            generation,
            signers: entries,
        }),
    )
    .unwrap();
}

#[test]
fn trusted_candidate_initializes_and_reuses_canonical_state() {
    let root = TempTrustRoot::new("initialize");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);

    let first =
        enforce_candidate_set_trust(&provider, 7, &digest('b'), std::slice::from_ref(&active))
            .unwrap()
            .unwrap();
    assert_eq!(first.contract, GALAXY_PROVIDER_TRUST_STATE_CONTRACT);
    assert_eq!(first.status, "verified-persistent-trust");
    assert_eq!(first.registry_generation, 1);
    assert_eq!(first.highest_candidate_generation, 7);
    assert_eq!(
        first.active_signer_ids.as_slice(),
        std::slice::from_ref(&active)
    );
    assert!(first.revoked_signer_ids.is_empty());

    let state_path = &provider.trust_policy.as_ref().unwrap().state_path;
    let first_source = fs::read_to_string(state_path).unwrap();
    assert!(!first_source.contains(&root.path().display().to_string()));
    let second = enforce_candidate_set_trust(&provider, 7, &digest('b'), &[active])
        .unwrap()
        .unwrap();
    assert_eq!(second, first);
    assert_eq!(fs::read_to_string(state_path).unwrap(), first_source);
}

#[test]
fn revoked_and_unknown_signers_fail_before_state_creation() {
    let root = TempTrustRoot::new("signers");
    let provider = root.provider();
    let active = signer('a');
    let revoked = signer('b');
    let unknown = signer('c');
    write_registry(&provider, 1, &[(&active, "active"), (&revoked, "revoked")]);

    let error = enforce_candidate_set_trust(&provider, 1, &digest('d'), &[revoked]).unwrap_err();
    assert!(error.contains("is revoked"), "{error}");
    let error = enforce_candidate_set_trust(&provider, 1, &digest('d'), &[unknown]).unwrap_err();
    assert!(error.contains("is not authorized"), "{error}");
    assert!(!provider.trust_policy.as_ref().unwrap().state_path.exists());
}

#[test]
fn candidate_generation_rejects_rollback_and_same_generation_fork() {
    let root = TempTrustRoot::new("candidate_generation");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);
    enforce_candidate_set_trust(&provider, 7, &digest('b'), std::slice::from_ref(&active)).unwrap();

    let rollback =
        enforce_candidate_set_trust(&provider, 6, &digest('c'), std::slice::from_ref(&active))
            .unwrap_err();
    assert!(rollback.contains("candidate-set rollback"), "{rollback}");
    let fork =
        enforce_candidate_set_trust(&provider, 7, &digest('c'), std::slice::from_ref(&active))
            .unwrap_err();
    assert!(fork.contains("same-generation fork"), "{fork}");

    let advanced = enforce_candidate_set_trust(&provider, 8, &digest('c'), &[active])
        .unwrap()
        .unwrap();
    assert_eq!(advanced.status, "verified-persistent-trust");
    assert_eq!(advanced.highest_candidate_generation, 8);
}

#[test]
fn registry_generation_rejects_rollback_and_same_generation_fork() {
    let root = TempTrustRoot::new("registry_generation");
    let provider = root.provider();
    let active = signer('a');
    let future = signer('b');
    write_registry(&provider, 1, &[(&active, "active")]);
    enforce_candidate_set_trust(&provider, 7, &digest('c'), std::slice::from_ref(&active)).unwrap();

    write_registry(&provider, 2, &[(&active, "active"), (&future, "revoked")]);
    let advanced =
        enforce_candidate_set_trust(&provider, 7, &digest('c'), std::slice::from_ref(&active))
            .unwrap()
            .unwrap();
    assert_eq!(advanced.status, "verified-persistent-trust");
    assert_eq!(
        advanced.revoked_signer_ids.as_slice(),
        std::slice::from_ref(&future)
    );

    write_registry(&provider, 1, &[(&active, "active")]);
    let rollback =
        enforce_candidate_set_trust(&provider, 7, &digest('c'), std::slice::from_ref(&active))
            .unwrap_err();
    assert!(rollback.contains("registry rollback"), "{rollback}");

    write_registry(&provider, 2, &[(&active, "active"), (&future, "active")]);
    let fork = enforce_candidate_set_trust(&provider, 7, &digest('c'), &[active]).unwrap_err();
    assert!(fork.contains("registry same-generation fork"), "{fork}");
}

#[test]
fn policy_files_inside_provider_root_fail_closed() {
    let root = TempTrustRoot::new("provider_root");
    let mut provider = root.provider();
    let active = signer('a');
    let registry_path = provider.root.join("registry.toml");
    provider.trust_policy = Some(GalaxyResolutionProviderTrustPolicy {
        registry_path: registry_path.clone(),
        state_path: provider.root.join("state.toml"),
    });
    fs::write(
        &registry_path,
        render_registry(&TrustRegistry {
            provider_id: provider.provider_id.clone(),
            provider_kind: provider.provider_kind.clone(),
            generation: 1,
            signers: vec![TrustSigner {
                signer_id: active.clone(),
                status: "active".to_owned(),
            }],
        }),
    )
    .unwrap();

    let error = enforce_candidate_set_trust(&provider, 1, &digest('b'), &[active]).unwrap_err();
    assert!(
        error.contains("must live outside the provider root"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn writable_registry_or_control_directory_fails_closed() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempTrustRoot::new("writable_controls");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);
    let policy = provider.trust_policy.as_ref().unwrap();

    fs::set_permissions(&policy.registry_path, fs::Permissions::from_mode(0o666)).unwrap();
    let error =
        enforce_candidate_set_trust(&provider, 1, &digest('b'), std::slice::from_ref(&active))
            .unwrap_err();
    assert!(
        error.contains("must not be group or other writable"),
        "{error}"
    );

    fs::set_permissions(&policy.registry_path, fs::Permissions::from_mode(0o644)).unwrap();
    let parent = policy.state_path.parent().unwrap();
    fs::set_permissions(parent, fs::Permissions::from_mode(0o777)).unwrap();
    let error = enforce_candidate_set_trust(&provider, 1, &digest('b'), &[active]).unwrap_err();
    assert!(
        error.contains("parent") && error.contains("writable"),
        "{error}"
    );
}

#[test]
fn tampered_state_identity_fails_closed() {
    let root = TempTrustRoot::new("state_tamper");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);
    enforce_candidate_set_trust(&provider, 1, &digest('b'), std::slice::from_ref(&active)).unwrap();
    let state_path = &provider.trust_policy.as_ref().unwrap().state_path;
    let source = fs::read_to_string(state_path).unwrap();
    fs::write(
        state_path,
        source.replacen(
            "highest_candidate_generation = 1",
            "highest_candidate_generation = 2",
            1,
        ),
    )
    .unwrap();

    let error = enforce_candidate_set_trust(&provider, 2, &digest('c'), &[active]).unwrap_err();
    assert!(error.contains("identity drifted"), "{error}");
}

#[test]
fn oversized_state_fails_at_the_bounded_reader() {
    let root = TempTrustRoot::new("state_size");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);
    enforce_candidate_set_trust(&provider, 1, &digest('b'), std::slice::from_ref(&active)).unwrap();
    let state_path = &provider.trust_policy.as_ref().unwrap().state_path;
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(state_path)
        .unwrap()
        .set_len(MAX_TRUST_DOCUMENT_BYTES + 1)
        .unwrap();

    let error = enforce_candidate_set_trust(&provider, 2, &digest('c'), &[active]).unwrap_err();
    assert!(error.contains("exceeds the 1048576-byte limit"), "{error}");
}

#[test]
fn concurrent_replay_serializes_to_one_canonical_state() {
    let root = TempTrustRoot::new("concurrent");
    let provider = root.provider();
    let active = signer('a');
    write_registry(&provider, 1, &[(&active, "active")]);

    let reports = std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                let provider = provider.clone();
                let active = active.clone();
                scope.spawn(move || {
                    enforce_candidate_set_trust(&provider, 9, &digest('b'), &[active])
                        .unwrap()
                        .unwrap()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(reports.iter().all(|report| report == &reports[0]));
    let state = read_state(&provider.trust_policy.as_ref().unwrap().state_path)
        .unwrap()
        .unwrap();
    assert_eq!(state.highest_candidate_generation, 9);
    assert_eq!(state.state_sha256, reports[0].state_sha256);
}
