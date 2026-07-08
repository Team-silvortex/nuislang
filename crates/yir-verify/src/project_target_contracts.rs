use std::collections::BTreeMap;

use yir_core::Node;

use crate::project_contracts::parse_semicolon_kv_contract;

pub(crate) fn verify_kernel_slot_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    let fields = value
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            let (key, raw) = entry.split_once('=').ok_or_else(|| {
                format!(
                    "project contract node `{node_name}` has invalid kernel slot field `{entry}`"
                )
            })?;
            let raw = raw.strip_prefix("i64:").ok_or_else(|| {
                format!(
                    "project contract node `{node_name}` expects i64-encoded kernel slot value in `{entry}`"
                )
            })?;
            let parsed = raw.parse::<i64>().map_err(|_| {
                format!(
                    "project contract node `{node_name}` has non-integer kernel slot value `{entry}`"
                )
            })?;
            Ok((key.trim(), parsed))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;

    let bind_core = *fields.get("bind_core").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `bind_core` field")
    })?;
    let queue_depth = *fields.get("queue_depth").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `queue_depth` field")
    })?;
    let batch_lanes = *fields.get("batch_lanes").ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `batch_lanes` field")
    })?;

    if bind_core < 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `bind_core >= 0`, got `{bind_core}`"
        ));
    }
    if queue_depth <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `queue_depth > 0`, got `{queue_depth}`"
        ));
    }
    if batch_lanes <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `batch_lanes > 0`, got `{batch_lanes}`"
        ));
    }

    let target_batch_lanes = target
        .op
        .args
        .last()
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` references kernel profile `{}` without target_config args",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project contract node `{node_name}` references kernel profile `{}` with non-integer batch lanes",
                target.name
            )
        })?;

    if target_batch_lanes != batch_lanes {
        return Err(format!(
            "project contract node `{node_name}` encodes `batch_lanes={batch_lanes}`, but `{}` uses `{target_batch_lanes}`",
            target.name
        ));
    }

    Ok(())
}

pub(crate) fn verify_target_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
    domain: &str,
) -> Result<(), String> {
    if target.op.module != domain || target.op.instruction != "target_config" {
        return Err(format!(
            "project contract node `{node_name}` must target `{domain}.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "target contract")?;
    let arch = fields
        .get("arch")
        .copied()
        .ok_or_else(|| format!("project contract node `{node_name}` is missing `arch` field"))?;
    let runtime = fields
        .get("runtime")
        .copied()
        .ok_or_else(|| format!("project contract node `{node_name}` is missing `runtime` field"))?;
    let lane_width = fields.get("lane_width").copied().ok_or_else(|| {
        format!("project contract node `{node_name}` is missing `lane_width` field")
    })?;
    let backend_features = fields.get("backend_features").copied();
    let arch = arch
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("project contract node `{node_name}` expects `arch=symbol:...`"))?;
    let runtime = runtime.strip_prefix("symbol:").ok_or_else(|| {
        format!("project contract node `{node_name}` expects `runtime=symbol:...`")
    })?;
    let lane_width = lane_width.strip_prefix("i64:").ok_or_else(|| {
        format!("project contract node `{node_name}` expects `lane_width=i64:...`")
    })?;
    let lane_width = lane_width
        .parse::<i64>()
        .map_err(|_| format!("project contract node `{node_name}` has non-integer lane_width"))?;
    if lane_width <= 0 {
        return Err(format!(
            "project contract node `{node_name}` requires `lane_width > 0`, got `{lane_width}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "project contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_runtime = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "project contract node `{node_name}` references `{}` without runtime arg",
            target.name
        )
    })?;
    let target_lane_width = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "project contract node `{node_name}` references `{}` without lane width arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "project contract node `{node_name}` references `{}` with non-integer lane width",
                target.name
            )
        })?;
    if target_arch != arch {
        return Err(format!(
            "project contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_runtime != runtime {
        return Err(format!(
            "project contract node `{node_name}` encodes `runtime={runtime}`, but `{}` uses `{target_runtime}`",
            target.name
        ));
    }
    if target_lane_width != lane_width {
        return Err(format!(
            "project contract node `{node_name}` encodes `lane_width={lane_width}`, but `{}` uses `{target_lane_width}`",
            target.name
        ));
    }
    if let Some(backend_features) = backend_features {
        let backend_features = backend_features.strip_prefix("list:").ok_or_else(|| {
            format!("project contract node `{node_name}` expects `backend_features=list:...`")
        })?;
        let target_backend_features = target.op.args.get(3).map(String::as_str);
        if target_backend_features != Some(backend_features) {
            return Err(format!(
                "project contract node `{node_name}` encodes `backend_features={backend_features}`, but `{}` uses `{}`",
                target.name,
                target_backend_features.unwrap_or("<none>")
            ));
        }
    }
    Ok(())
}

pub(crate) fn verify_cpu_target_contract_text(
    node_name: &str,
    value: &str,
    target: &Node,
) -> Result<(), String> {
    if target.op.module != "cpu" || target.op.instruction != "target_config" {
        return Err(format!(
            "lowering contract node `{node_name}` must target `cpu.target_config`, got `{}.{}`",
            target.op.module, target.op.instruction
        ));
    }
    let fields = parse_semicolon_kv_contract(node_name, value, "cpu target contract")?;
    let arch = fields
        .get("arch")
        .copied()
        .ok_or_else(|| format!("lowering contract node `{node_name}` is missing `arch` field"))?;
    let abi = fields
        .get("abi")
        .copied()
        .ok_or_else(|| format!("lowering contract node `{node_name}` is missing `abi` field"))?;
    let vector_bits = fields.get("vector_bits").copied().ok_or_else(|| {
        format!("lowering contract node `{node_name}` is missing `vector_bits` field")
    })?;
    let isa_family = fields.get("isa_family").copied();
    let isa_features = fields.get("isa_features").copied();
    let arch = arch
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("lowering contract node `{node_name}` expects `arch=symbol:...`"))?;
    let abi = abi
        .strip_prefix("symbol:")
        .ok_or_else(|| format!("lowering contract node `{node_name}` expects `abi=symbol:...`"))?;
    let vector_bits = vector_bits.strip_prefix("i64:").ok_or_else(|| {
        format!("lowering contract node `{node_name}` expects `vector_bits=i64:...`")
    })?;
    let vector_bits = vector_bits
        .parse::<i64>()
        .map_err(|_| format!("lowering contract node `{node_name}` has non-integer vector_bits"))?;
    if vector_bits <= 0 {
        return Err(format!(
            "lowering contract node `{node_name}` requires `vector_bits > 0`, got `{vector_bits}`"
        ));
    }
    let target_arch = target.op.args.first().map(String::as_str).ok_or_else(|| {
        format!(
            "lowering contract node `{node_name}` references `{}` without arch arg",
            target.name
        )
    })?;
    let target_abi = target.op.args.get(1).map(String::as_str).ok_or_else(|| {
        format!(
            "lowering contract node `{node_name}` references `{}` without abi arg",
            target.name
        )
    })?;
    let target_vector_bits = target
        .op
        .args
        .get(2)
        .ok_or_else(|| {
            format!(
                "lowering contract node `{node_name}` references `{}` without vector_bits arg",
                target.name
            )
        })?
        .parse::<i64>()
        .map_err(|_| {
            format!(
                "lowering contract node `{node_name}` references `{}` with non-integer vector_bits",
                target.name
            )
        })?;
    let target_isa_family = target.op.args.get(3).map(String::as_str);
    let target_isa_features = target.op.args.get(4).map(String::as_str);
    if target_arch != arch {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `arch={arch}`, but `{}` uses `{target_arch}`",
            target.name
        ));
    }
    if target_abi != abi {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `abi={abi}`, but `{}` uses `{target_abi}`",
            target.name
        ));
    }
    if target_vector_bits != vector_bits {
        return Err(format!(
            "lowering contract node `{node_name}` encodes `vector_bits={vector_bits}`, but `{}` uses `{target_vector_bits}`",
            target.name
        ));
    }
    if let Some(isa_family) = isa_family {
        let isa_family = isa_family.strip_prefix("symbol:").ok_or_else(|| {
            format!("lowering contract node `{node_name}` expects `isa_family=symbol:...`")
        })?;
        if target_isa_family != Some(isa_family) {
            return Err(format!(
                "lowering contract node `{node_name}` encodes `isa_family={isa_family}`, but `{}` uses `{}`",
                target.name,
                target_isa_family.unwrap_or("<none>")
            ));
        }
    }
    if let Some(isa_features) = isa_features {
        let isa_features = isa_features.strip_prefix("list:").ok_or_else(|| {
            format!("lowering contract node `{node_name}` expects `isa_features=list:...`")
        })?;
        if target_isa_features != Some(isa_features) {
            return Err(format!(
                "lowering contract node `{node_name}` encodes `isa_features={isa_features}`, but `{}` uses `{}`",
                target.name,
                target_isa_features.unwrap_or("<none>")
            ));
        }
    }
    Ok(())
}
