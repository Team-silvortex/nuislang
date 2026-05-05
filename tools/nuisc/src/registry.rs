use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

const INDEX_FILE: &str = "index.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageIndexEntry {
    pub package_id: String,
    pub manifest: String,
    pub domain_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageManifest {
    pub manifest_schema: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub binary_extension: String,
    pub package_layout: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub abi_targets: Vec<String>,
    pub implementation_kinds: Vec<String>,
    pub loader_entry: String,
    pub loader_abi: String,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: String,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub default_lanes: Vec<String>,
    pub profiles: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredAbiTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub backend_family: Option<String>,
    pub host_adaptive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinding {
    pub package_id: String,
    pub domain_family: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub default_lanes: Vec<String>,
    pub matched_support_surface: Vec<String>,
    pub matched_support_profile_slots: Vec<String>,
    pub covered_support_profile_slots: Vec<String>,
    pub uncovered_support_profile_slots: Vec<String>,
    pub registered_units: Vec<String>,
    pub bound_unit: Option<String>,
    pub used_units: Vec<String>,
    pub instantiated_units: Vec<String>,
    pub used_host_ffi_abis: Vec<String>,
    pub used_host_ffi_symbols: Vec<String>,
    pub matched_resources: Vec<String>,
    pub matched_ops: Vec<String>,
    pub undeclared_ops: Vec<String>,
    pub frontend: String,
    pub entry_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBindingPlan {
    pub bindings: Vec<NustarBinding>,
}

pub fn load_index(root: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let path = root.join(INDEX_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_index(&source, &path)
}

pub fn load_manifest(root: &Path, package_id: &str) -> Result<NustarPackageManifest, String> {
    let index = load_index(root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.package_id == package_id)
        .ok_or_else(|| {
            format!(
                "nustar package `{package_id}` is not present in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(root, &entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_manifest_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarPackageManifest, String> {
    let index = load_index(root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.domain_family == domain_family)
        .ok_or_else(|| {
            format!(
                "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(root, &entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_all_manifests(root: &Path) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for entry in load_index(root)? {
        manifests.push(load_manifest(root, &entry.package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn required_package_ids(module: &YirModule) -> Vec<String> {
    let mut package_ids = BTreeSet::new();
    for node in &module.nodes {
        package_ids.insert(format!("official.{}", node.op.module));
        if node.op.module == "cpu" && node.op.instruction == "instantiate_unit" {
            if let Some(domain) = node.op.args.first() {
                package_ids.insert(format!("official.{domain}"));
            }
        }
    }
    package_ids.into_iter().collect()
}

pub fn load_required_manifests(
    root: &Path,
    module: &YirModule,
) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for package_id in required_package_ids(module) {
        manifests.push(load_manifest(root, &package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn plan_bindings(
    root: &Path,
    nir: &NirModule,
    module: &YirModule,
    domain: &str,
    unit: &str,
    declared_used_units: &[(String, String)],
    declared_externs: &[(String, String)],
) -> Result<NustarBindingPlan, String> {
    let manifests = load_required_manifests(root, module)?;
    validate_unit_binding(&manifests, domain, unit)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let registered_units = manifest
            .unit_types
            .iter()
            .filter(|unit| !unit.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let bound_unit = if manifest.domain_family == domain {
            Some(unit.to_owned())
        } else {
            None
        };
        let used_units = declared_used_units
            .iter()
            .filter(|(used_domain, _)| used_domain == &manifest.domain_family)
            .map(|(_, used_unit)| used_unit.clone())
            .collect::<Vec<_>>();
        let instantiated_units = module
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "instantiate_unit"
                    && node.op.args.first().map(String::as_str)
                        == Some(manifest.domain_family.as_str())
            })
            .filter_map(|node| node.op.args.get(1).cloned())
            .collect::<Vec<_>>();
        let used_host_ffi_abis = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(abi, _)| abi.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let used_host_ffi_symbols = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(_, symbol)| symbol.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let (matched_support_surface, matched_support_profile_slots) =
            detect_matched_support_usage(nir, &manifest.domain_family);
        let covered_support_profile_slots = covered_profile_slots(
            &manifest.domain_family,
            &matched_support_surface,
            &matched_support_profile_slots,
        );
        let uncovered_support_profile_slots = manifest
            .support_profile_slots
            .iter()
            .filter(|slot| {
                !covered_support_profile_slots
                    .iter()
                    .any(|covered| covered == *slot)
            })
            .cloned()
            .collect::<Vec<_>>();

        let matched_resources = module
            .resources
            .iter()
            .filter(|resource| {
                manifest
                    .resource_families
                    .iter()
                    .any(|family| family == resource.kind.family())
            })
            .map(|resource| resource.name.clone())
            .collect::<Vec<_>>();

        let matched_ops = module
            .nodes
            .iter()
            .filter(|node| node.op.module == manifest.domain_family)
            .map(|node| node.op.full_name())
            .collect::<Vec<_>>();

        if matched_ops.is_empty() && instantiated_units.is_empty() && used_units.is_empty() {
            return Err(format!(
                "nustar package `{}` was selected but no matching ops were bound",
                manifest.package_id
            ));
        }

        let undeclared_ops = matched_ops
            .iter()
            .filter(|op| !manifest.ops.iter().any(|candidate| candidate == *op))
            .cloned()
            .collect::<Vec<_>>();

        bindings.push(NustarBinding {
            package_id: manifest.package_id,
            domain_family: manifest.domain_family,
            ast_entry: manifest.ast_entry,
            nir_entry: manifest.nir_entry,
            yir_lowering_entry: manifest.yir_lowering_entry,
            part_verify_entry: manifest.part_verify_entry,
            machine_abi_policy: manifest.machine_abi_policy,
            abi_profiles: manifest.abi_profiles,
            abi_capabilities: manifest.abi_capabilities,
            ast_surface: manifest.ast_surface,
            nir_surface: manifest.nir_surface,
            yir_lowering: manifest.yir_lowering,
            part_verify: manifest.part_verify,
            support_surface: manifest.support_surface,
            support_profile_slots: manifest.support_profile_slots,
            default_lanes: manifest.default_lanes,
            matched_support_surface,
            matched_support_profile_slots,
            covered_support_profile_slots,
            uncovered_support_profile_slots,
            registered_units,
            bound_unit,
            used_units,
            instantiated_units,
            used_host_ffi_abis,
            used_host_ffi_symbols,
            matched_resources,
            matched_ops,
            undeclared_ops,
            frontend: manifest.frontend,
            entry_crate: manifest.entry_crate,
        });
    }

    bindings.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(NustarBindingPlan { bindings })
}

fn covered_profile_slots(
    domain_family: &str,
    matched_support_surface: &[String],
    matched_support_profile_slots: &[String],
) -> Vec<String> {
    let mut covered = matched_support_profile_slots
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for surface in matched_support_surface {
        for slot in implied_slots_for_surface(domain_family, surface) {
            covered.insert(slot.to_string());
        }
    }
    covered.into_iter().collect::<Vec<_>>()
}

fn implied_slots_for_surface(domain_family: &str, surface: &str) -> &'static [&'static str] {
    match (domain_family, surface) {
        ("shader", "shader.profile.render.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.seed.color.v1") => &["packet_color_slot", "material_mode"],
        ("shader", "shader.profile.seed.speed.v1") => &["packet_speed_slot", "packet_tag"],
        ("shader", "shader.profile.seed.radius.v1") => {
            &["packet_radius_slot", "packet_field_count"]
        }
        ("shader", "shader.profile.packet.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
        ],
        ("shader", "shader.profile.target.v1") => &["target"],
        ("shader", "shader.profile.viewport.v1") => &["viewport"],
        ("shader", "shader.profile.pipeline.v1") => &["pipeline"],
        ("shader", "shader.profile.draw-budget.v1") => &["vertex_count", "instance_count"],
        ("shader", "shader.profile.packet-slots.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
        ],
        ("shader", "shader.profile.packet-tag.v1") => &["packet_tag"],
        ("shader", "shader.profile.material-mode.v1") => &["material_mode"],
        ("shader", "shader.profile.pass-kind.v1") => &["pass_kind"],
        ("shader", "shader.profile.packet-field-count.v1") => &["packet_field_count"],
        ("data", "data.profile.bind-core.v1") => &["bind_core"],
        ("data", "data.profile.send.uplink.v1") => &[
            "window_offset",
            "uplink_len",
            "marker:cpu_to_shader",
            "marker:uplink_pipe",
            "marker:uplink_pipe_class",
            "marker:uplink_payload_class",
            "marker:uplink_payload_shape",
            "marker:uplink_window_policy",
        ],
        ("data", "data.profile.send.downlink.v1") => &[
            "window_offset",
            "downlink_len",
            "marker:shader_to_cpu",
            "marker:downlink_pipe",
            "marker:downlink_pipe_class",
            "marker:downlink_payload_class",
            "marker:downlink_payload_shape",
            "marker:downlink_window_policy",
        ],
        ("data", "data.profile.handle-table.v1") => &["handle_table"],
        ("data", "data.profile.window-layout.v1") => {
            &["window_offset", "uplink_len", "downlink_len"]
        }
        ("data", "data.profile.sync-markers.v1") => {
            &["marker:cpu_to_shader", "marker:shader_to_cpu"]
        }
        ("data", "data.profile.pipe-markers.v1") => &["marker:uplink_pipe", "marker:downlink_pipe"],
        ("data", "data.profile.pipe-class.v1") => {
            &["marker:uplink_pipe_class", "marker:downlink_pipe_class"]
        }
        ("data", "data.profile.payload-class.v1") => &[
            "marker:uplink_payload_class",
            "marker:downlink_payload_class",
        ],
        ("data", "data.profile.payload-shape.v1") => &[
            "marker:uplink_payload_shape",
            "marker:downlink_payload_shape",
        ],
        ("data", "data.profile.window-policy.v1") => &[
            "marker:uplink_window_policy",
            "marker:downlink_window_policy",
        ],
        _ => &[],
    }
}

fn detect_matched_support_usage(
    module: &NirModule,
    domain_family: &str,
) -> (Vec<String>, Vec<String>) {
    let mut surfaces = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_support_usage_stmt(stmt, domain_family, &mut surfaces, &mut slots);
        }
    }
    (
        surfaces.into_iter().collect::<Vec<_>>(),
        slots.into_iter().collect::<Vec<_>>(),
    )
}

fn collect_support_usage_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_support_usage_expr(value, domain_family, surfaces, slots),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in then_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
            for stmt in else_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_support_usage_expr(value, domain_family, surfaces, slots);
            }
        }
    }
}

fn collect_support_usage_expr(
    expr: &NirExpr,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::ShaderProfileTargetRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.target.v1".to_owned());
            slots.insert("target".to_owned());
        }
        NirExpr::ShaderProfileViewportRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.viewport.v1".to_owned());
            slots.insert("viewport".to_owned());
        }
        NirExpr::ShaderProfilePipelineRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pipeline.v1".to_owned());
            slots.insert("pipeline".to_owned());
        }
        NirExpr::ShaderProfileVertexCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("vertex_count".to_owned());
        }
        NirExpr::ShaderProfileInstanceCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("instance_count".to_owned());
        }
        NirExpr::ShaderProfilePacketColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_color_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_speed_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_radius_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketTagRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-tag.v1".to_owned());
            slots.insert("packet_tag".to_owned());
        }
        NirExpr::ShaderProfileMaterialModeRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.material-mode.v1".to_owned());
            slots.insert("material_mode".to_owned());
        }
        NirExpr::ShaderProfilePassKindRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pass-kind.v1".to_owned());
            slots.insert("pass_kind".to_owned());
        }
        NirExpr::ShaderProfilePacketFieldCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-field-count.v1".to_owned());
            slots.insert("packet_field_count".to_owned());
        }
        NirExpr::ShaderProfileColorSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.color.v1".to_owned());
        }
        NirExpr::ShaderProfileSpeedSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.speed.v1".to_owned());
        }
        NirExpr::ShaderProfileRadiusSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.radius.v1".to_owned());
        }
        NirExpr::ShaderProfilePacket { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet.v1".to_owned());
        }
        NirExpr::ShaderProfileRender { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.render.v1".to_owned());
        }
        NirExpr::ShaderInlineWgsl { .. } if domain_family == "shader" => {
            surfaces.insert("shader.inline.wgsl.v1".to_owned());
        }
        NirExpr::DataProfileBindCoreRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::DataProfileWindowOffsetRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("window_offset".to_owned());
        }
        NirExpr::DataProfileUplinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("uplink_len".to_owned());
        }
        NirExpr::DataProfileDownlinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("downlink_len".to_owned());
        }
        NirExpr::DataProfileHandleTableRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.handle-table.v1".to_owned());
            slots.insert("handle_table".to_owned());
        }
        NirExpr::DataProfileMarkerRef { tag, .. } if domain_family == "data" => {
            let (surface, slot) = match tag.as_str() {
                "cpu_to_shader" | "shader_to_cpu" => {
                    ("data.profile.sync-markers.v1", format!("marker:{tag}"))
                }
                "uplink_pipe" | "downlink_pipe" => {
                    ("data.profile.pipe-markers.v1", format!("marker:{tag}"))
                }
                "uplink_pipe_class" | "downlink_pipe_class" => {
                    ("data.profile.pipe-class.v1", format!("marker:{tag}"))
                }
                "uplink_payload_class" | "downlink_payload_class" => {
                    ("data.profile.payload-class.v1", format!("marker:{tag}"))
                }
                "uplink_payload_shape" | "downlink_payload_shape" => {
                    ("data.profile.payload-shape.v1", format!("marker:{tag}"))
                }
                "uplink_window_policy" | "downlink_window_policy" => {
                    ("data.profile.window-policy.v1", format!("marker:{tag}"))
                }
                _ => ("data.profile.sync-markers.v1", format!("marker:{tag}")),
            };
            surfaces.insert(surface.to_owned());
            slots.insert(slot);
        }
        NirExpr::KernelProfileBindCoreRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::KernelProfileQueueDepthRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.queue-depth.v1".to_owned());
            slots.insert("queue_depth".to_owned());
        }
        NirExpr::KernelProfileBatchLanesRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.batch-lanes.v1".to_owned());
            slots.insert("batch_lanes".to_owned());
        }
        NirExpr::DataProfileSendUplink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.uplink.v1".to_owned());
        }
        NirExpr::DataProfileSendDownlink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.downlink.v1".to_owned());
        }
        _ => {}
    }

    walk_child_exprs(expr, &mut |child| {
        collect_support_usage_expr(child, domain_family, surfaces, slots);
    });
}

fn walk_child_exprs(expr: &NirExpr, f: &mut dyn FnMut(&NirExpr)) {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => f(inner),
        NirExpr::AllocNode { value, next } => {
            f(value);
            f(next);
        }
        NirExpr::AllocBuffer { len, fill } => {
            f(len);
            f(fill);
        }
        NirExpr::LoadAt { buffer, index } => {
            f(buffer);
            f(index);
        }
        NirExpr::StoreValue { target, value } => {
            f(target);
            f(value);
        }
        NirExpr::StoreNext { target, next } => {
            f(target);
            f(next);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            f(buffer);
            f(index);
            f(value);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => {
            f(input)
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            f(input);
            f(offset);
            f(len);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            f(base);
            f(delta);
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            f(color);
            f(speed);
            f(radius);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            f(delta);
            f(scale);
            f(base);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. }
        | NirExpr::FieldAccess { base: input, .. } => f(input),
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuExternCall { args, .. } | NirExpr::Call { args, .. } => {
            for arg in args {
                f(arg);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            f(task);
            f(limit);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            f(receiver);
            for arg in args {
                f(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                f(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            f(target);
            f(pipeline);
            f(viewport);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            f(pass);
            f(packet);
            f(vertex_count);
            f(instance_count);
        }
        _ => {}
    }
}

pub fn validate_unit_binding(
    manifests: &[NustarPackageManifest],
    domain: &str,
    unit: &str,
) -> Result<(), String> {
    let manifest = manifests
        .iter()
        .find(|manifest| manifest.domain_family == domain)
        .ok_or_else(|| format!("no nustar manifest loaded for mod domain `{domain}`"))?;

    if manifest.unit_types.is_empty() {
        return Ok(());
    }

    if manifest
        .unit_types
        .iter()
        .any(|candidate| candidate == unit)
    {
        return Ok(());
    }

    Err(format!(
        "unit `{unit}` is not registered by nustar package `{}` for mod domain `{domain}`",
        manifest.package_id
    ))
}

pub fn validate_manifest_abi(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<(), String> {
    if manifest
        .abi_profiles
        .iter()
        .any(|profile| profile == required_abi)
    {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare required ABI `{}`; declared ABI profiles: {}",
        manifest.package_id,
        manifest.domain_family,
        required_abi,
        if manifest.abi_profiles.is_empty() {
            "<none>".to_owned()
        } else {
            manifest.abi_profiles.join(", ")
        }
    ))
}

pub fn registered_abi_target(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        if abi.trim() != required_abi {
            continue;
        }
        return parse_registered_abi_target(required_abi, fields, manifest, raw);
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare abi target metadata for `{}`",
        manifest.package_id, manifest.domain_family, required_abi
    ))
}

pub fn registered_abi_target_for_clang(
    manifest: &NustarPackageManifest,
    clang_target: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    let mut matches = Vec::new();
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        let target = parse_registered_abi_target(abi.trim(), fields, manifest, raw)?;
        if target.clang_target == clang_target {
            matches.push(target);
        }
    }
    matches.into_iter().next().ok_or_else(|| {
        format!(
            "nustar package `{}` for domain `{}` does not register clang target `{}` in `abi_targets`",
            manifest.package_id, manifest.domain_family, clang_target
        )
    })
}

pub fn used_ops_for_domain(module: &YirModule, domain_family: &str) -> Vec<String> {
    let mut ops = module
        .nodes
        .iter()
        .filter(|node| node.op.module == domain_family)
        .map(|node| node.op.full_name())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ops.sort();
    ops
}

pub fn validate_abi_capabilities(
    manifest: &NustarPackageManifest,
    required_abi: &str,
    used_surfaces: &[String],
    used_ops: &[String],
) -> Result<(), String> {
    if manifest.abi_capabilities.is_empty() {
        return Ok(());
    }

    let mut surface_allowed = BTreeSet::new();
    let mut op_allowed = BTreeSet::new();
    let mut saw_required_abi = false;
    for raw in &manifest.abi_capabilities {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; ABI id must not be empty",
                manifest.package_id, raw
            ));
        }
        let abi_matches = abi.trim() == required_abi;
        if !abi_matches {
            continue;
        }
        saw_required_abi = true;
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(value) = cap.strip_prefix("surface:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `surface:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                surface_allowed.insert(value.to_owned());
            } else if let Some(value) = cap.strip_prefix("op:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `op:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                op_allowed.insert(value.to_owned());
            } else {
                return Err(format!(
                    "nustar package `{}` has invalid abi_capabilities capability `{}` in `{}`; expected `surface:<pattern>` or `op:<pattern>`",
                    manifest.package_id, cap, raw
                ));
            }
        }
    }

    if !saw_required_abi {
        return Err(format!(
            "ABI `{}` of nustar package `{}` has no abi_capabilities mapping; add `{}:...` in manifest",
            required_abi, manifest.package_id, required_abi
        ));
    }

    if !surface_allowed.is_empty() && !surface_allowed.contains("*") {
        for surface in used_surfaces {
            if !surface_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, surface))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow support surface `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    surface,
                    surface_allowed
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    if !op_allowed.is_empty() && !op_allowed.contains("*") {
        for op in used_ops {
            if !op_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, op))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow op `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    op,
                    op_allowed.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    Ok(())
}

fn capability_matches(pattern: &str, actual: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return actual.starts_with(prefix);
    }
    pattern == actual
}

pub fn manifest_path(root: &Path, entry: &NustarPackageIndexEntry) -> PathBuf {
    root.join(&entry.manifest)
}

fn parse_index(source: &str, path: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if !current.is_empty() {
                entries.push(parse_index_entry(&current.join("\n"), path)?);
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }

    if !current.is_empty() {
        entries.push(parse_index_entry(&current.join("\n"), path)?);
    }

    entries.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(entries)
}

fn parse_index_entry(source: &str, path: &Path) -> Result<NustarPackageIndexEntry, String> {
    Ok(NustarPackageIndexEntry {
        package_id: parse_required_string(source, "package_id", path)?,
        manifest: parse_required_string(source, "manifest", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
    })
}

fn parse_manifest(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    let manifest_schema = parse_optional_string(source, "manifest_schema")
        .unwrap_or_else(|| "nustar-manifest-v1".to_owned());
    let package_id = parse_required_string(source, "package_id", path)?;
    let domain_family = parse_required_string(source, "domain_family", path)?;
    let frontend = parse_required_string(source, "frontend", path)?;
    let entry_crate = parse_required_string(source, "entry_crate", path)?;
    let ast_entry = parse_optional_string(source, "ast_entry")
        .unwrap_or_else(|| format!("{}.ast.bootstrap.v1", domain_family));
    let nir_entry = parse_optional_string(source, "nir_entry")
        .unwrap_or_else(|| format!("{}.nir.bootstrap.v1", domain_family));
    let yir_lowering_entry = parse_optional_string(source, "yir_lowering_entry")
        .unwrap_or_else(|| format!("{}.yir.lowering.v1", domain_family));
    let part_verify_entry = parse_optional_string(source, "part_verify_entry")
        .unwrap_or_else(|| format!("{}.verify.partial.v1", domain_family));
    let ast_surface = parse_optional_string_array(source, "ast_surface")
        .unwrap_or_else(|| vec![format!("{domain_family}.mod-ast.v1")]);
    let nir_surface = parse_optional_string_array(source, "nir_surface")
        .unwrap_or_else(|| vec![format!("nir.{domain_family}.surface.v1")]);
    let yir_lowering = parse_optional_string_array(source, "yir_lowering")
        .unwrap_or_else(|| vec![format!("yir.{domain_family}.lowering.v1")]);
    let part_verify = parse_optional_string_array(source, "part_verify")
        .unwrap_or_else(|| vec![format!("verify.{domain_family}.contract.v1")]);
    let binary_extension =
        parse_optional_string(source, "binary_extension").unwrap_or_else(|| "nustar".to_owned());
    let package_layout = parse_optional_string(source, "package_layout")
        .unwrap_or_else(|| "single-envelope".to_owned());
    let machine_abi_policy = parse_optional_string(source, "machine_abi_policy")
        .unwrap_or_else(|| "exact-match".to_owned());
    let abi_profiles = parse_optional_string_array(source, "abi_profiles").unwrap_or_default();
    let abi_capabilities =
        parse_optional_string_array(source, "abi_capabilities").unwrap_or_default();
    let abi_targets = parse_optional_string_array(source, "abi_targets").unwrap_or_default();
    let implementation_kinds = parse_optional_string_array(source, "implementation_kinds")
        .unwrap_or_else(|| vec!["native-stub".to_owned()]);
    let loader_entry = parse_optional_string(source, "loader_entry")
        .unwrap_or_else(|| "nustar.bootstrap.v1".to_owned());
    let loader_abi = parse_optional_string(source, "loader_abi")
        .unwrap_or_else(|| "nustar-loader-v1".to_owned());
    let host_ffi_surface =
        parse_optional_string_array(source, "host_ffi_surface").unwrap_or_default();
    let host_ffi_abis = parse_optional_string_array(source, "host_ffi_abis").unwrap_or_default();
    let host_ffi_bridge =
        parse_optional_string(source, "host_ffi_bridge").unwrap_or_else(|| "none".to_owned());
    let support_surface =
        parse_optional_string_array(source, "support_surface").unwrap_or_default();
    let support_profile_slots =
        parse_optional_string_array(source, "support_profile_slots").unwrap_or_default();
    let default_lanes = parse_optional_string_array(source, "default_lanes").unwrap_or_default();
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let unit_types = parse_optional_string_array(source, "unit_types").unwrap_or_default();
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        ast_entry,
        nir_entry,
        yir_lowering_entry,
        part_verify_entry,
        ast_surface,
        nir_surface,
        yir_lowering,
        part_verify,
        binary_extension,
        package_layout,
        machine_abi_policy,
        abi_profiles,
        abi_capabilities,
        abi_targets,
        implementation_kinds,
        loader_entry,
        loader_abi,
        host_ffi_surface,
        host_ffi_abis,
        host_ffi_bridge,
        support_surface,
        support_profile_slots,
        default_lanes,
        profiles,
        resource_families,
        unit_types,
        lowering_targets,
        ops,
    })
}

fn parse_registered_abi_target(
    abi: &str,
    fields: &str,
    manifest: &NustarPackageManifest,
    raw: &str,
) -> Result<RegisteredAbiTarget, String> {
    let mut host_adaptive = false;
    let mut machine_arch = None::<String>;
    let mut machine_os = None::<String>;
    let mut object_format = None::<String>;
    let mut calling_abi = None::<String>;
    let mut clang_target = None::<String>;
    let mut backend_family = None::<String>;
    for field in fields.split('|').map(str::trim).filter(|field| !field.is_empty()) {
        let Some((key, value)) = field.split_once('=') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets field `{}` in `{}`; expected `key=value`",
                manifest.package_id, field, raw
            ));
        };
        let value = value.trim();
        if value == "host" {
            host_adaptive = true;
        }
        match key.trim() {
            "arch" => machine_arch = Some(resolve_host_adaptive_arch(value).to_owned()),
            "os" => machine_os = Some(resolve_host_adaptive_os(value).to_owned()),
            "object" => object_format = Some(resolve_host_adaptive_object(value).to_owned()),
            "calling" => calling_abi = Some(resolve_host_adaptive_calling(value).to_owned()),
            "clang" => clang_target = Some(resolve_host_adaptive_clang(value).to_owned()),
            "backend" => backend_family = Some(value.to_owned()),
            other => {
                return Err(format!(
                    "nustar package `{}` has invalid abi_targets key `{}` in `{}`; expected `arch`, `os`, `object`, `calling`, `clang`, or `backend`",
                    manifest.package_id, other, raw
                ));
            }
        }
    }
    Ok(RegisteredAbiTarget {
        abi: abi.to_owned(),
        machine_arch: machine_arch.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `arch=`",
                manifest.package_id, raw
            )
        })?,
        machine_os: machine_os.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `os=`",
                manifest.package_id, raw
            )
        })?,
        object_format: object_format.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `object=`",
                manifest.package_id, raw
            )
        })?,
        calling_abi: calling_abi.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `calling=`",
                manifest.package_id, raw
            )
        })?,
        clang_target: clang_target.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `clang=`",
                manifest.package_id, raw
            )
        })?,
        backend_family,
        host_adaptive,
    })
}

fn resolve_host_adaptive_arch(value: &str) -> &'static str {
    if value == "host" {
        host_arch()
    } else {
        match value {
            "arm64" => "arm64",
            "x86_64" => "x86_64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_os(value: &str) -> &'static str {
    if value == "host" {
        host_os()
    } else {
        match value {
            "darwin" => "darwin",
            "linux" => "linux",
            "windows" => "windows",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_object(value: &str) -> &'static str {
    if value == "host" {
        host_object_format()
    } else {
        match value {
            "mach-o" => "mach-o",
            "elf" => "elf",
            "coff" => "coff",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_calling(value: &str) -> &'static str {
    if value == "host" {
        host_calling_abi()
    } else {
        match value {
            "aapcs64-darwin" => "aapcs64-darwin",
            "aapcs64" => "aapcs64",
            "sysv64" => "sysv64",
            "win64" => "win64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_clang(value: &str) -> String {
    if value == "host" {
        host_clang_target()
    } else {
        value.to_owned()
    }
}

fn host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_calling_abi() -> &'static str {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

fn host_clang_target() -> String {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_owned(),
        (arch, os) => format!("{arch}-unknown-{os}"),
    }
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid string value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid array value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(trimmed[1..trimmed.len() - 1].to_owned())
    } else {
        None
    }
}

fn parse_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    for part in inner.split(',') {
        items.push(parse_quoted(part.trim())?);
    }
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu_manifest_with_host_target() -> NustarPackageManifest {
        NustarPackageManifest {
            manifest_schema: "nustar-manifest-v1".to_owned(),
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            frontend: "nustar-cpu".to_owned(),
            entry_crate: "crates/yir-domain-cpu".to_owned(),
            ast_entry: "cpu.ast.bootstrap.v1".to_owned(),
            nir_entry: "cpu.nir.bootstrap.v1".to_owned(),
            yir_lowering_entry: "cpu.yir.lowering.v1".to_owned(),
            part_verify_entry: "cpu.verify.partial.v1".to_owned(),
            ast_surface: vec!["cpu.mod-ast.v1".to_owned()],
            nir_surface: vec!["nir.cpu.surface.v1".to_owned()],
            yir_lowering: vec!["yir.cpu.lowering.v1".to_owned()],
            part_verify: vec!["verify.cpu.contract.v1".to_owned()],
            binary_extension: "nustar".to_owned(),
            package_layout: "single-envelope".to_owned(),
            machine_abi_policy: "exact-match".to_owned(),
            abi_profiles: vec!["cpu.host.v1".to_owned()],
            abi_capabilities: vec!["cpu.host.v1:op:cpu.*".to_owned()],
            abi_targets: vec![
                "cpu.host.v1:arch=host|os=host|object=host|calling=host|clang=host".to_owned(),
            ],
            implementation_kinds: vec!["native-stub".to_owned()],
            loader_entry: "nustar.bootstrap.v1".to_owned(),
            loader_abi: "nustar-loader-v1".to_owned(),
            host_ffi_surface: Vec::new(),
            host_ffi_abis: Vec::new(),
            host_ffi_bridge: "none".to_owned(),
            support_surface: Vec::new(),
            support_profile_slots: Vec::new(),
            default_lanes: Vec::new(),
            profiles: vec!["aot".to_owned()],
            resource_families: vec!["cpu".to_owned()],
            unit_types: vec!["Main".to_owned()],
            lowering_targets: vec!["llvm".to_owned()],
            ops: vec!["cpu.const".to_owned()],
        }
    }

    #[test]
    fn registered_abi_target_expands_host_adaptive_contract() {
        let manifest = cpu_manifest_with_host_target();
        let target = registered_abi_target(&manifest, "cpu.host.v1").unwrap();
        assert_eq!(target.machine_arch, host_arch());
        assert_eq!(target.machine_os, host_os());
        assert_eq!(target.object_format, host_object_format());
        assert_eq!(target.calling_abi, host_calling_abi());
        assert_eq!(target.clang_target, host_clang_target());
        assert!(target.host_adaptive);
    }

    #[test]
    fn registered_abi_target_preserves_backend_family() {
        let mut manifest = cpu_manifest_with_host_target();
        manifest.abi_profiles = vec!["cpu.backend.v1".to_owned()];
        manifest.abi_capabilities = vec!["cpu.backend.v1:op:cpu.*".to_owned()];
        manifest.abi_targets = vec![
            "cpu.backend.v1:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin|backend=metal".to_owned(),
        ];
        let target = registered_abi_target(&manifest, "cpu.backend.v1").unwrap();
        assert_eq!(target.backend_family.as_deref(), Some("metal"));
        assert!(!target.host_adaptive);
    }
}
