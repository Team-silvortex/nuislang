use yir_core::YirModule;
use yir_lower_llvm::native_session::{LiteralPrintBuildPolicy, NativeSessionBridge};

pub const PACKAGING_PREFIX: &str = "native-session-aot-bundle:";
pub const POLICY_PACKAGING_PREFIX: &str = "native-session-policy-aot-bundle:";

pub fn literal_print_packaging_mode<I, S>(
    id: &str,
    sites: I,
    loop_work_limit: u64,
    helper_entry_limit: u64,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    yir_core::YirApplicationSession::validate_identifier(id)?;
    let policy = LiteralPrintBuildPolicy::new(sites, loop_work_limit, helper_entry_limit)?;
    Ok(format!("{POLICY_PACKAGING_PREFIX}{}:{id}", policy.token()))
}

pub fn build_policy(mode: &str) -> Result<Option<LiteralPrintBuildPolicy>, String> {
    mode.strip_prefix(POLICY_PACKAGING_PREFIX)
        .map(|rest| {
            let (token, id) = rest
                .split_once(':')
                .ok_or("native policy packaging requires <policy-token>:<registration-id>")?;
            yir_core::YirApplicationSession::validate_identifier(id)?;
            LiteralPrintBuildPolicy::parse(token)
        })
        .transpose()
}

pub fn emit_for_packaging_mode(
    module: &YirModule,
    mode: &str,
) -> Result<NativeSessionBridge, String> {
    let id = registration_id(mode)?.ok_or("not a native session packaging mode")?;
    match build_policy(mode)? {
        Some(policy) => policy.emit(module, id),
        None => yir_lower_llvm::native_session::emit_registered(module, id),
    }
}

/// The registration is part of artifact and cache identity, not a runtime default.
pub fn registration_id(mode: &str) -> Result<Option<&str>, String> {
    if let Some(rest) = mode.strip_prefix(POLICY_PACKAGING_PREFIX) {
        build_policy(mode)?;
        return Ok(Some(rest.split_once(':').unwrap().1));
    }
    let Some(id) = mode.strip_prefix(PACKAGING_PREFIX) else {
        if matches!(
            mode,
            "native-session-aot-bundle" | "native-session-policy-aot-bundle"
        ) {
            return Err("native session packaging requires :<registration-id>".to_owned());
        }
        return Ok(None);
    };
    yir_core::YirApplicationSession::validate_identifier(id)?;
    Ok(Some(id))
}

pub(crate) fn has_bound_inputs(mode: &str) -> bool {
    mode == "headless-aot-bundle"
        || mode.starts_with(PACKAGING_PREFIX)
        || mode.starts_with(POLICY_PACKAGING_PREFIX)
}

/// Validate claims against the admitted lowering, not mutable host capabilities.
pub fn verify_checkpoint(
    module: &YirModule,
    id: &str,
    llvm_ir: &str,
    bundle: &str,
) -> Result<String, String> {
    verify_checkpoint_for_packaging_mode(
        module,
        &format!("{PACKAGING_PREFIX}{id}"),
        llvm_ir,
        bundle,
    )
}

pub fn verify_checkpoint_for_packaging_mode(
    module: &YirModule,
    mode: &str,
    llvm_ir: &str,
    bundle: &str,
) -> Result<String, String> {
    let bridge = emit_for_packaging_mode(module, mode)?;
    let id = bridge.session_id.as_str();
    if bridge.llvm_ir != llvm_ir {
        return Err(
            "native session LLVM checkpoint does not match the selected YIR lowering".to_owned(),
        );
    }
    for (key, expected) in [
        ("cpu_host_binary_mode", "native_scalar_session"),
        ("runtime_bootstrap_mode", "static_native_session"),
        ("application_session_id", id),
        (
            "native_session_contract",
            yir_core::native_scalar_session::CONTRACT,
        ),
        ("native_session_identity", "exact-yir-graph"),
        ("native_session_layout", bridge.state_layout.source()),
        ("single_binary", "true"),
    ] {
        let values = bundle
            .lines()
            .filter_map(|line| line.split_once('='))
            .filter(|(candidate, _)| *candidate == key)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        if values != [expected] {
            return Err(format!(
                "native session bundle requires exactly one `{key}={expected}`"
            ));
        }
    }
    let claims = build_policy(mode)?
        .map(|p| p.bundle_claims())
        .unwrap_or_default();
    let reserved = |key: &str| {
        key.starts_with("native_session_policy")
            || matches!(
                key,
                "native_session_loop_work_limit"
                    | "native_session_helper_entry_limit"
                    | "native_session_literal_print_bound"
            )
    };
    for (key, _) in bundle.lines().filter_map(|line| line.split_once('=')) {
        if reserved(key) && !claims.iter().any(|(expected, _)| *expected == key) {
            return Err(format!(
                "native session bundle has an unexpected policy claim `{key}`"
            ));
        }
    }
    for (key, expected) in claims {
        let values = bundle
            .lines()
            .filter_map(|line| line.split_once('='))
            .filter(|(candidate, _)| *candidate == key)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        if values != [expected.as_str()] {
            return Err(format!(
                "native session bundle requires exactly one `{key}={expected}`"
            ));
        }
    }
    Ok(bridge.state_layout.source().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaging_identity_is_explicit_bounded_and_utf8() {
        assert_eq!(registration_id("native-cpu-llvm").unwrap(), None);
        for id in ["counter", "\u{4f1a}\u{8bdd}", "a:b"] {
            assert_eq!(
                registration_id(&format!("{PACKAGING_PREFIX}{id}")).unwrap(),
                Some(id)
            );
        }
        for mode in [
            "native-session-aot-bundle",
            "native-session-aot-bundle:",
            "native-session-aot-bundle:a b",
            "native-session-aot-bundle:a\n",
        ] {
            assert!(registration_id(mode).is_err());
        }
        assert!(registration_id(&format!("{PACKAGING_PREFIX}{}", "x".repeat(257))).is_err());
        for id in ["counter", "\u{4f1a}\u{8bdd}", "a:b"] {
            let mode = format!("{POLICY_PACKAGING_PREFIX}v1.0.2.61:{id}");
            assert_eq!(registration_id(&mode).unwrap(), Some(id));
            assert!(has_bound_inputs(&mode));
            assert_eq!(build_policy(&mode).unwrap().unwrap().token(), "v1.0.2.61");
        }
        for suffix in ["", "v1.0.1", "v1.0.1:", "v1.00.1:id", "v1.0.1.61.61:id"] {
            assert!(registration_id(&format!("{POLICY_PACKAGING_PREFIX}{suffix}")).is_err());
        }
    }
}
