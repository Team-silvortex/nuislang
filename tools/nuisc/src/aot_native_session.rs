use yir_core::YirModule;

pub const PACKAGING_PREFIX: &str = "native-session-aot-bundle:";

/// The registration is part of artifact and cache identity, not a runtime default.
pub fn registration_id(mode: &str) -> Result<Option<&str>, String> {
    let Some(id) = mode.strip_prefix(PACKAGING_PREFIX) else {
        if mode == "native-session-aot-bundle" {
            return Err("native session packaging requires :<registration-id>".to_owned());
        }
        return Ok(None);
    };
    yir_core::YirApplicationSession::validate_identifier(id)?;
    Ok(Some(id))
}

pub(crate) fn has_bound_inputs(mode: &str) -> bool {
    mode == "headless-aot-bundle" || mode.starts_with(PACKAGING_PREFIX)
}

/// Validate claims against the admitted lowering, not mutable host capabilities.
pub fn verify_checkpoint(
    module: &YirModule,
    id: &str,
    llvm_ir: &str,
    bundle: &str,
) -> Result<String, String> {
    let bridge = yir_lower_llvm::native_session::emit_registered(module, id)?;
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
    }
}
