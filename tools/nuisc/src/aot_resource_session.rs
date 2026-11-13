use yir_core::YirModule;
use yir_lower_llvm::native_resource_session::NativeResourceBridge;

pub const PACKAGING_PREFIX: &str = "native-resource-session-aot-bundle:";
const FRAME_KEY: &str = "native_resource_frame_contract";

pub fn verify_frame_export_capability(bundle: &str) -> Result<(), String> {
    use yir_core::native_resource_session::{
        CHECKED_CONTRACT, CONTRACT, FRAME_EXPORT_CONTRACT, MIXED_CONTRACT,
        MIXED_FRAME_EXPORT_CONTRACT,
    };
    let contracts = bundle
        .lines()
        .filter_map(|line| line.strip_prefix("native_resource_contract="))
        .collect::<Vec<_>>();
    let expected = match contracts.as_slice() {
        [CONTRACT] => FRAME_EXPORT_CONTRACT,
        [MIXED_CONTRACT] | [CHECKED_CONTRACT] => MIXED_FRAME_EXPORT_CONTRACT,
        _ => return Err("ambiguous resource frame State contract".to_owned()),
    };
    let values = bundle
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| *key == FRAME_KEY)
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    if values != [expected] {
        return Err(
            "resource artifact does not declare the exact native RGBA8 frame export contract"
                .to_owned(),
        );
    }
    Ok(())
}

pub fn registration_id(mode: &str) -> Result<Option<&str>, String> {
    if mode == "native-resource-session-aot-bundle" {
        return Err("resource session packaging requires :<registration-id>".to_owned());
    }
    let Some(id) = mode.strip_prefix(PACKAGING_PREFIX) else {
        return Ok(None);
    };
    yir_core::YirApplicationSession::validate_identifier(id)?;
    Ok(Some(id))
}

pub fn emit_for_packaging_mode(
    module: &YirModule,
    mode: &str,
) -> Result<NativeResourceBridge, String> {
    let id = registration_id(mode)?.ok_or("not a resource session packaging mode")?;
    let registration = yir_core::native_resource_session::registered(module, id)?;
    yir_lower_llvm::native_resource_session::emit_registered(module, registration)
}

pub fn verify_checkpoint(
    module: &YirModule,
    mode: &str,
    llvm: &str,
    bundle: &str,
) -> Result<(), String> {
    let bridge = emit_for_packaging_mode(module, mode)?;
    if bridge.llvm_ir != llvm {
        return Err("resource session LLVM checkpoint identity mismatch".to_owned());
    }
    let canonical = bridge.registration.canonical();
    let claims = [
        ("cpu_host_binary_mode", "native_resource_session"),
        ("runtime_bootstrap_mode", "static_native_resource_session"),
        (
            "native_resource_contract",
            bridge.registration.contract.as_str(),
        ),
        ("native_resource_identity", "exact-yir-graph"),
        ("native_resource_registration", canonical.as_str()),
        ("native_resource_open_export", bridge.symbols[0].as_str()),
        ("native_resource_event_export", bridge.symbols[1].as_str()),
        ("native_resource_close_export", bridge.symbols[2].as_str()),
        ("single_binary", "true"),
    ];
    let fields = bundle
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect::<Vec<_>>();
    // Older resource-only artifacts remain valid, but cannot grant frame export.
    if fields.iter().any(|(key, _)| *key == FRAME_KEY) {
        verify_frame_export_capability(bundle)?;
    }
    for (key, value) in claims {
        if fields
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .collect::<Vec<_>>()
            != [value]
        {
            return Err(format!(
                "resource session bundle requires exactly one `{key}={value}`"
            ));
        }
    }
    if fields.iter().any(|(key, _)| {
        key.starts_with("native_session_")
            || *key == "application_session_id"
            || (key.starts_with("native_resource_")
                && *key != FRAME_KEY
                && !claims.iter().any(|(k, _)| k == key))
    }) {
        return Err("unexpected resource session bundle claim".to_owned());
    }
    Ok(())
}
