use super::*;

pub(in crate::project) fn render_project_abi_index(
    project: &LoadedProject,
) -> Result<String, String> {
    let mut out = String::new();
    write_project_abi_index(&mut out, project)
        .expect("writing project abi index to String should not fail");
    Ok(out)
}

pub(in crate::project) fn write_project_abi_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> Result<(), String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok(());
    }
    let mode = if resolution.explicit {
        "# mode=explicit"
    } else {
        "# mode=auto-recommended"
    };
    writeln!(out, "{mode}").map_err(|error| error.to_string())?;
    write_project_abi_graph_line(out, &resolution).map_err(|error| error.to_string())?;
    writeln!(out).map_err(|error| error.to_string())?;
    for item in project_abi_selection_views(&resolution) {
        let arch = item.machine_arch.as_deref().unwrap_or("unknown");
        let os = item.machine_os.as_deref().unwrap_or("unknown");
        let object = item.object_format.as_deref().unwrap_or("unknown");
        let calling = item.calling_abi.as_deref().unwrap_or("unknown");
        let backend = item.backend_family.as_deref().unwrap_or("none");
        writeln!(
            out,
            "domain\t{}\tabi={}\tarch={}\tos={}\tobject={}\tcalling={}\tbackend={}",
            item.domain, item.abi, arch, os, object, calling, backend
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn render_project_abi_graph_line(resolution: &ProjectAbiResolution) -> String {
    let mut out = String::new();
    write_project_abi_graph_line(&mut out, resolution)
        .expect("writing project abi graph line to String should not fail");
    out
}

pub fn describe_project_abi_graph(project: &LoadedProject) -> Result<String, String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok("graph\tmode=none\tdomains=<none>".to_owned());
    }
    Ok(render_project_abi_graph_line(&resolution))
}
