use super::*;

pub(in crate::project) fn render_project_host_ffi_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_host_ffi_index(&mut out, project)
        .expect("writing project host ffi index to String should not fail");
    out
}

pub(in crate::project) fn write_project_host_ffi_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();

        for function in &module.ast.externs {
            write!(
                out,
                "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}\tsignature=",
                relative,
                module.ast.domain,
                module.ast.unit,
                function.abi,
                function.interface.as_deref().unwrap_or("-"),
                function.name,
            )?;
            write_host_ffi_signature(out, function)?;
            writeln!(out)?;
        }

        for interface in &module.ast.extern_interfaces {
            for method in &interface.methods {
                write!(
                    out,
                    "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}__{}\tsignature=",
                    relative,
                    module.ast.domain,
                    module.ast.unit,
                    interface.abi,
                    interface.name,
                    interface.name,
                    method.name,
                )?;
                write_host_ffi_signature(out, method)?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
