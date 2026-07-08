use yir_core::Node;

use crate::project_contracts::parse_semicolon_kv_contract;

pub(crate) fn verify_abi_selection_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
    domain: &str,
) -> Result<(), String> {
    if target.op.module != domain || target.op.instruction != "target_config" {
        return Err(format!(
            "project ABI selection contract node `{node_name}` must target `{domain}.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI selection contract")?;
    let mode = fields.get("mode").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `mode` field")
    })?;
    let abi = fields.get("abi").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `abi` field")
    })?;
    let arch = fields.get("arch").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `arch` field")
    })?;
    let runtime = fields.get("runtime").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `runtime` field")
    })?;
    let lane_width = fields.get("lane_width").copied().ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` is missing `lane_width` field")
    })?;
    let backend_features = fields.get("backend_features").copied();
    let mode = mode.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `mode=symbol:...`")
    })?;
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    let abi = abi.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `abi=symbol:...`")
    })?;
    if abi.is_empty() {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires non-empty `abi`"
        ));
    }
    let arch = arch.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `arch=symbol:...`")
    })?;
    let runtime = runtime.strip_prefix("symbol:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `runtime=symbol:...`")
    })?;
    let lane_width = lane_width.strip_prefix("i64:").ok_or_else(|| {
        format!("project ABI selection contract node `{node_name}` expects `lane_width=i64:...`")
    })?;
    let lane_width = lane_width.parse::<i64>().map_err(|_| {
        format!("project ABI selection contract node `{node_name}` has non-integer lane_width")
    })?;
    if lane_width <= 0 {
        return Err(format!(
            "project ABI selection contract node `{node_name}` requires `lane_width > 0`, got `{lane_width}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "project ABI selection contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_runtime = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "project ABI selection contract node `{node_name}` references `{}` without runtime arg",
            target.name
        )
    })?;
    let target_lane_width = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "project ABI selection contract node `{node_name}` references `{}` without lane width arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project ABI selection contract node `{node_name}` references `{}` with non-integer lane width",
                target.name
            )
        })?;
    if target_arch != arch {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_runtime != runtime {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `runtime={runtime}`, but `{}` uses `{target_runtime}`",
            target.name
        ));
    }
    if target_lane_width != lane_width {
        return Err(format!(
            "project ABI selection contract node `{node_name}` encodes `lane_width={lane_width}`, but `{}` uses `{target_lane_width}`",
            target.name
        ));
    }
    if let Some(backend_features) = backend_features {
        let backend_features = backend_features.strip_prefix("list:").ok_or_else(|| {
            format!(
                "project ABI selection contract node `{node_name}` expects `backend_features=list:...`"
            )
        })?;
        let target_backend_features = target.op.args.get(3).map(String::as_str);
        if target_backend_features != Some(backend_features) {
            return Err(format!(
                "project ABI selection contract node `{node_name}` encodes `backend_features={backend_features}`, but `{}` uses `{}`",
                target.name,
                target_backend_features.unwrap_or("<none>")
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_abi_selection_summary_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "text" {
        return Err(format!(
            "project ABI summary node `{node_name}` must target `cpu.text`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let target_value = target
        .op
        .args
        .first()
        .map(|item| item.trim())
        .ok_or_else(|| {
            format!(
                "project ABI summary node `{node_name}` references `{}` without summary payload",
                target.name
            )
        })?;
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI summary contract")?;
    for key in ["mode", "abi", "arch", "os", "object", "calling", "backend"] {
        let raw = fields.get(key).copied().ok_or_else(|| {
            format!("project ABI summary node `{node_name}` is missing `{key}` field")
        })?;
        let parsed = raw.strip_prefix("symbol:").ok_or_else(|| {
            format!("project ABI summary node `{node_name}` expects `{key}=symbol:...`")
        })?;
        if parsed.is_empty() {
            return Err(format!(
                "project ABI summary node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    let mode = fields
        .get("mode")
        .and_then(|value| value.strip_prefix("symbol:"))
        .unwrap_or_default();
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI summary node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    if value != target_value {
        return Err(format!(
            "project ABI summary node `{node_name}` encodes `{value}`, but `{}` uses `{target_value}`",
            target.name
        ));
    }
    Ok(())
}

pub(crate) fn verify_abi_graph_summary_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "text" {
        return Err(format!(
            "project ABI graph summary node `{node_name}` must target `cpu.text`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let target_value = target
        .op
        .args
        .first()
        .map(|item| item.trim())
        .ok_or_else(|| {
            format!(
                "project ABI graph summary node `{node_name}` references `{}` without summary payload",
                target.name
            )
        })?;
    let fields = parse_semicolon_kv_contract(node_name, value, "ABI graph summary")?;
    for key in [
        "mode",
        "domains",
        "cpu_summary",
        "data_summary",
        "kernel_target",
        "shader_target",
        "network_target",
    ] {
        let raw = fields.get(key).copied().ok_or_else(|| {
            format!("project ABI graph summary node `{node_name}` is missing `{key}` field")
        })?;
        let parsed = raw.strip_prefix("symbol:").ok_or_else(|| {
            format!("project ABI graph summary node `{node_name}` expects `{key}=symbol:...`")
        })?;
        if parsed.is_empty() {
            return Err(format!(
                "project ABI graph summary node `{node_name}` requires non-empty `{key}`"
            ));
        }
    }
    let mode = fields
        .get("mode")
        .and_then(|value| value.strip_prefix("symbol:"))
        .unwrap_or_default();
    if mode != "explicit" && mode != "auto" {
        return Err(format!(
            "project ABI graph summary node `{node_name}` requires `mode` to be `explicit` or `auto`, got `{mode}`"
        ));
    }
    if value != target_value {
        return Err(format!(
            "project ABI graph summary node `{node_name}` encodes `{value}`, but `{}` uses `{target_value}`",
            target.name
        ));
    }
    Ok(())
}
