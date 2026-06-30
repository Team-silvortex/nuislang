use super::*;

pub(in crate::project::rendering) fn selected_lowering_target_for_domain(
    domain: &str,
    abi: &str,
    registered_lowering_targets: &[String],
) -> Result<Option<String>, String> {
    match domain {
        "cpu" => {
            crate::aot::resolve_cpu_build_target_from_abi(Path::new("nustar-packages"), abi)?;
            Ok(Some("llvm".to_owned()))
        }
        "shader" | "kernel" | "network" => Ok(resolve_registered_abi_target(
            domain,
            Some(&ProjectAbiResolution {
                requirements: vec![super::ProjectAbiRequirement {
                    domain: domain.to_owned(),
                    abi: abi.to_owned(),
                }],
                explicit: true,
            }),
        )?
        .and_then(|target| {
            selected_lowering_target_for_registered_abi_target(
                domain,
                &target,
                registered_lowering_targets,
            )
        })),
        _ => Ok(None),
    }
}

pub fn project_abi_selection_views(
    resolution: &ProjectAbiResolution,
) -> Vec<ProjectAbiSelectionView> {
    let mut views = resolution
        .requirements
        .iter()
        .map(|item| {
            let target = resolve_registered_abi_target(&item.domain, Some(resolution))
                .ok()
                .flatten();
            ProjectAbiSelectionView {
                domain: item.domain.clone(),
                abi: item.abi.clone(),
                machine_arch: target.as_ref().map(|target| target.machine_arch.clone()),
                machine_os: target.as_ref().map(|target| target.machine_os.clone()),
                object_format: target.as_ref().map(|target| target.object_format.clone()),
                calling_abi: target.as_ref().map(|target| target.calling_abi.clone()),
                clang_target: target.as_ref().map(|target| target.clang_target.clone()),
                backend_family: target
                    .as_ref()
                    .and_then(|target| target.backend_family.clone()),
                vendor: target.as_ref().and_then(|target| target.vendor.clone()),
                device_class: target
                    .as_ref()
                    .and_then(|target| target.device_class.clone()),
                host_adaptive: target.as_ref().map(|target| target.host_adaptive),
            }
        })
        .collect::<Vec<_>>();
    views.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    views
}

pub fn project_abi_selection_view_json(item: &ProjectAbiSelectionView) -> String {
    format!(
        "{{\"domain\":\"{}\",\"abi\":\"{}\",\"abi_target_machine\":{},\"abi_target_object\":{},\"abi_target_calling\":{},\"abi_target_clang\":{},\"abi_target_backend\":{},\"abi_target_vendor\":{},\"abi_target_device\":{},\"abi_target_host_adaptive\":{}}}",
        json_escape(&item.domain),
        json_escape(&item.abi),
        item.machine_arch
            .as_deref()
            .zip(item.machine_os.as_deref())
            .map(|(arch, os)| format!("\"{}-{}\"", json_escape(arch), json_escape(os)))
            .unwrap_or_else(|| "null".to_owned()),
        item.object_format
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.calling_abi
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.clang_target
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.backend_family
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.vendor
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.device_class
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.host_adaptive
            .map(|value| if value { "true" } else { "false" }.to_owned())
            .unwrap_or_else(|| "null".to_owned())
    )
}

pub fn render_project_abi_selection_lines(resolution: &ProjectAbiResolution) -> Vec<String> {
    let mut out = String::new();
    write_project_abi_selection_lines(&mut out, resolution)
        .expect("writing project abi selection lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn render_project_abi_selection_view_lines(item: &ProjectAbiSelectionView) -> Vec<String> {
    let mut out = String::new();
    write_project_abi_selection_view_lines(&mut out, item)
        .expect("writing project abi selection view lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_abi_selection_lines<W: fmt::Write>(
    out: &mut W,
    resolution: &ProjectAbiResolution,
) -> fmt::Result {
    for item in project_abi_selection_views(resolution) {
        write_project_abi_selection_view_lines(out, &item)?;
    }
    Ok(())
}

pub fn write_project_abi_selection_view_lines<W: fmt::Write>(
    out: &mut W,
    item: &ProjectAbiSelectionView,
) -> fmt::Result {
    writeln!(out, "abi: {}={}", item.domain, item.abi)?;
    if let (Some(machine_arch), Some(machine_os)) =
        (item.machine_arch.as_deref(), item.machine_os.as_deref())
    {
        writeln!(out, "  abi_target_machine: {}-{}", machine_arch, machine_os)?;
    }
    if let Some(object_format) = item.object_format.as_deref() {
        writeln!(out, "  abi_target_object: {}", object_format)?;
    }
    if let Some(calling_abi) = item.calling_abi.as_deref() {
        writeln!(out, "  abi_target_calling: {}", calling_abi)?;
    }
    if let Some(clang_target) = item.clang_target.as_deref() {
        writeln!(out, "  abi_target_clang: {}", clang_target)?;
    }
    if let Some(backend_family) = item.backend_family.as_deref() {
        writeln!(out, "  abi_target_backend: {}", backend_family)?;
    }
    if let Some(vendor) = item.vendor.as_deref() {
        writeln!(out, "  abi_target_vendor: {}", vendor)?;
    }
    if let Some(device_class) = item.device_class.as_deref() {
        writeln!(out, "  abi_target_device: {}", device_class)?;
    }
    if let Some(host_adaptive) = item.host_adaptive {
        writeln!(
            out,
            "  abi_target_host_adaptive: {}",
            if host_adaptive { "true" } else { "false" }
        )?;
    }
    Ok(())
}
