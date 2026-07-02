use super::*;
use crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY;
use yir_core::ffi::ffi_symbol_signature_hash;

#[cfg(test)]
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
            let symbol = host_ffi_symbol_name(function);
            let signature_pattern = host_ffi_signature_pattern(function);
            let signature_hash =
                ffi_symbol_signature_hash(&function.abi, &symbol, &signature_pattern);
            write!(
                out,
                "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}\tsignature=",
                relative,
                module.ast.domain,
                module.ast.unit,
                function.abi,
                function.interface.as_deref().unwrap_or("-"),
                symbol,
            )?;
            write_host_ffi_signature(out, function)?;
            writeln!(
                out,
                "\tsignature_pattern={signature_pattern}\tsignature_hash={signature_hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}"
            )?;
        }

        for interface in &module.ast.extern_interfaces {
            for method in &interface.methods {
                let symbol = host_ffi_symbol_name(method);
                let signature_pattern = host_ffi_signature_pattern(method);
                let signature_hash =
                    ffi_symbol_signature_hash(&method.abi, &symbol, &signature_pattern);
                write!(
                    out,
                    "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}\tsignature=",
                    relative,
                    module.ast.domain,
                    module.ast.unit,
                    interface.abi,
                    interface.name,
                    symbol,
                )?;
                write_host_ffi_signature(out, method)?;
                writeln!(
                    out,
                    "\tsignature_pattern={signature_pattern}\tsignature_hash={signature_hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}"
                )?;
            }
        }
    }
    Ok(())
}

fn host_ffi_symbol_name(function: &AstExternFunction) -> String {
    if let Some(symbol) = &function.host_symbol {
        return symbol.clone();
    }
    match &function.interface {
        Some(interface) => format!("{interface}__{}", function.name),
        None => function.name.clone(),
    }
}

fn host_ffi_signature_pattern(function: &AstExternFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| host_ffi_type_token(&param.ty))
        .collect::<Vec<_>>();
    format!(
        "{}({})",
        host_ffi_type_token(&function.return_type),
        params.join(",")
    )
}

fn host_ffi_type_token(ty: &AstTypeRef) -> String {
    render_ast_type_ref(ty)
        .chars()
        .map(|ch| match ch {
            ' ' | '<' | '>' | ',' => '_',
            _ => ch,
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned()
}
