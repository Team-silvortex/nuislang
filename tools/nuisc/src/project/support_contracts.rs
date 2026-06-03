use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn support_surface_for_domain(
    cache: &mut BTreeMap<String, BTreeSet<String>>,
    domain: &str,
) -> Result<BTreeSet<String>, String> {
    if let Some(surface) = cache.get(domain) {
        return Ok(surface.clone());
    }
    let manifest = crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)?;
    let surface = manifest
        .support_surface
        .into_iter()
        .collect::<BTreeSet<_>>();
    cache.insert(domain.to_owned(), surface.clone());
    Ok(surface)
}

pub(super) fn require_declared_support_surface(
    declared_surface: &BTreeSet<String>,
    domain: &str,
    unit: &str,
    required_surface: &str,
) -> Result<(), String> {
    if declared_surface.contains(required_surface) {
        return Ok(());
    }
    Err(format!(
        "project {} unit `{}.{}` requires nustar to declare support surface `{}`",
        domain, domain, unit, required_surface
    ))
}

pub(super) fn support_profile_slots_for_domain(domain: &str) -> Result<BTreeSet<String>, String> {
    let manifest = crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)?;
    Ok(manifest
        .support_profile_slots
        .into_iter()
        .collect::<BTreeSet<_>>())
}

pub(super) fn require_declared_profile_slot(
    declared_slots: &BTreeSet<String>,
    domain: &str,
    unit: &str,
    required_slot: &str,
) -> Result<(), String> {
    if declared_slots.contains(required_slot) {
        return Ok(());
    }
    Err(format!(
        "project {} unit `{}.{}` requires nustar to declare profile slot `{}`",
        domain, domain, unit, required_slot
    ))
}

pub(super) fn shader_support_surface_contract() -> &'static [&'static str] {
    &[
        "shader.profile.packet.v1",
        "shader.inline.wgsl.v1",
        "shader.profile.target.v1",
        "shader.profile.viewport.v1",
        "shader.profile.pipeline.v1",
        "shader.profile.draw-budget.v1",
        "shader.profile.packet-slots.v1",
        "shader.profile.packet-tag.v1",
        "shader.profile.material-mode.v1",
        "shader.profile.pass-kind.v1",
        "shader.profile.packet-field-count.v1",
    ]
}

pub(super) fn kernel_support_surface_contract() -> &'static [&'static str] {
    &[
        "kernel.profile.bind-core.v1",
        "kernel.profile.queue-depth.v1",
        "kernel.profile.batch-lanes.v1",
        "kernel.profile.entry.v1",
    ]
}

pub(super) fn network_support_surface_contract() -> &'static [&'static str] {
    &[
        "network.profile.bind-core.v1",
        "network.profile.endpoint-kind.v1",
    ]
}

pub(super) fn data_support_surface_contract() -> &'static [&'static str] {
    &[
        "data.profile.bind-core.v1",
        "data.profile.handle-table.v1",
        "data.profile.window-layout.v1",
        "data.profile.sync-markers.v1",
        "data.profile.pipe-markers.v1",
        "data.profile.pipe-class.v1",
        "data.profile.payload-class.v1",
        "data.profile.payload-shape.v1",
        "data.profile.window-policy.v1",
    ]
}

pub(super) fn shader_profile_slot_targets(
    unit: &str,
    packet_slots: Option<&[&'static str]>,
) -> Vec<(&'static str, String)> {
    let mut slots = vec![
        (
            "target",
            super::profile_targets::resolve_project_profile_target_name("shader", unit, "target"),
        ),
        (
            "viewport",
            super::profile_targets::resolve_project_profile_target_name("shader", unit, "viewport"),
        ),
        (
            "pipeline",
            super::profile_targets::resolve_project_profile_target_name("shader", unit, "pipeline"),
        ),
        (
            "vertex_count",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "vertex_count",
            ),
        ),
        (
            "instance_count",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "instance_count",
            ),
        ),
        (
            "packet_tag",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "packet_tag",
            ),
        ),
        (
            "material_mode",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "material_mode",
            ),
        ),
        (
            "pass_kind",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "pass_kind",
            ),
        ),
        (
            "packet_field_count",
            super::profile_targets::resolve_project_profile_target_name(
                "shader",
                unit,
                "packet_field_count",
            ),
        ),
    ];
    let packet_slots = packet_slots.unwrap_or(&[
        "packet_color_slot",
        "packet_speed_slot",
        "packet_radius_slot",
    ]);
    for slot in packet_slots {
        slots.push((
            slot,
            super::profile_targets::resolve_project_profile_target_name("shader", unit, slot),
        ));
    }
    slots
}

pub(super) fn kernel_profile_slot_targets(unit: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "bind_core",
            super::profile_targets::resolve_project_profile_target_name(
                "kernel",
                unit,
                "bind_core",
            ),
        ),
        (
            "queue_depth",
            super::profile_targets::resolve_project_profile_target_name(
                "kernel",
                unit,
                "queue_depth",
            ),
        ),
        (
            "batch_lanes",
            super::profile_targets::resolve_project_profile_target_name(
                "kernel",
                unit,
                "batch_lanes",
            ),
        ),
    ]
}

pub(super) fn data_profile_required_slots_for_link(
    from_domain: &str,
    to_domain: &str,
) -> Vec<&'static str> {
    let mut slots = vec![
        "bind_core",
        "window_offset",
        "uplink_len",
        "downlink_len",
        "handle_table",
        "marker:uplink_pipe",
        "marker:downlink_pipe",
        "marker:uplink_pipe_class",
        "marker:downlink_pipe_class",
        "marker:uplink_payload_class",
        "marker:downlink_payload_class",
        "marker:uplink_payload_shape",
        "marker:downlink_payload_shape",
        "marker:uplink_window_policy",
        "marker:downlink_window_policy",
    ];
    match (from_domain, to_domain) {
        ("cpu", "shader") => slots.push("marker:cpu_to_shader"),
        ("shader", "cpu") => slots.push("marker:shader_to_cpu"),
        ("cpu", "kernel") => slots.push("marker:cpu_to_kernel"),
        ("kernel", "cpu") => slots.push("marker:kernel_to_cpu"),
        _ => {}
    }
    slots
}
