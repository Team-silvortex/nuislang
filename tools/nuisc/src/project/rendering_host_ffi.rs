use super::*;
use crate::aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY;
use crate::registry::{
    HostFfiMemoryCapability, HostFfiMemoryDestructor, HostFfiRegistryView,
    HostFfiSymbolRegistration,
};
use yir_core::ffi::ffi_symbol_signature_hash;

#[cfg(test)]
pub(in crate::project) fn render_project_host_ffi_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_host_ffi_index(&mut out, project)
        .expect("registered project host ffi index should render");
    out
}

pub(in crate::project) fn write_project_host_ffi_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> Result<(), String> {
    let manifest = crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), "cffi")?;
    let registry = HostFfiRegistryView::try_from_manifest(&manifest)?;
    let mut destructor_authorities = BTreeMap::new();
    let mut rendered_symbols = BTreeMap::new();

    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();

        for function in &module.ast.externs {
            write_source_host_ffi_entry(
                out,
                &registry,
                &mut destructor_authorities,
                &mut rendered_symbols,
                &relative,
                &module.ast.domain,
                &module.ast.unit,
                function.interface.as_deref().unwrap_or("-"),
                function,
            )?;
        }

        for interface in &module.ast.extern_interfaces {
            for method in &interface.methods {
                write_source_host_ffi_entry(
                    out,
                    &registry,
                    &mut destructor_authorities,
                    &mut rendered_symbols,
                    &relative,
                    &module.ast.domain,
                    &module.ast.unit,
                    &interface.name,
                    method,
                )?;
            }
        }
    }
    for ((abi, symbol, signature_hash), signature_pattern) in destructor_authorities {
        if rendered_symbols.contains_key(&(abi.clone(), symbol.clone(), signature_hash.clone())) {
            continue;
        }
        writeln!(
            out,
            "@nustar-memory-authority\tmod cffi RegisteredHostMemory\tabi={abi}\tinterface=-\tsymbol={symbol}\tsignature={signature_pattern}\tsignature_pattern={signature_pattern}\tsignature_hash={signature_hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}\tmemory_capability_count=0\tmemory_capabilities=-"
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn host_ffi_memory_capabilities(
    registry: &HostFfiRegistryView,
    abi: &str,
    symbol: &str,
    signature_hash: &str,
) -> Vec<HostFfiMemoryCapability> {
    registry
        .memory_capabilities(abi, symbol, signature_hash)
        .to_vec()
}

#[allow(clippy::too_many_arguments)]
fn write_source_host_ffi_entry<W: fmt::Write>(
    out: &mut W,
    registry: &HostFfiRegistryView,
    destructor_authorities: &mut BTreeMap<(String, String, String), String>,
    rendered_symbols: &mut BTreeMap<(String, String, String), ()>,
    relative: &str,
    domain: &str,
    unit: &str,
    interface: &str,
    function: &AstExternFunction,
) -> Result<(), String> {
    let symbol = host_ffi_symbol_name(function);
    let signature_pattern = host_ffi_signature_pattern(function);
    let signature_hash = ffi_symbol_signature_hash(&function.abi, &symbol, &signature_pattern);
    let memory_capabilities =
        host_ffi_memory_capabilities(registry, &function.abi, &symbol, &signature_hash);
    collect_destructor_authorities(registry, &memory_capabilities, destructor_authorities)?;
    rendered_symbols.insert(
        (function.abi.clone(), symbol.clone(), signature_hash.clone()),
        (),
    );
    write!(
        out,
        "{relative}\tmod {domain} {unit}\tabi={}\tinterface={interface}\tsymbol={symbol}\tsignature=",
        function.abi,
    )
    .map_err(|error| error.to_string())?;
    write_host_ffi_signature(out, function).map_err(|error| error.to_string())?;
    writeln!(
        out,
        "\tsignature_pattern={signature_pattern}\tsignature_hash={signature_hash}\tpolicy={SIGNATURE_WHITELIST_POLICY}\tmemory_capability_count={}\tmemory_capabilities={}",
        memory_capabilities.len(),
        render_memory_capabilities(&memory_capabilities)
    )
    .map_err(|error| error.to_string())
}

fn collect_destructor_authorities(
    registry: &HostFfiRegistryView,
    capabilities: &[HostFfiMemoryCapability],
    authorities: &mut BTreeMap<(String, String, String), String>,
) -> Result<(), String> {
    for capability in capabilities {
        let HostFfiMemoryDestructor::Registered {
            symbol,
            signature_hash,
        } = &capability.destructor
        else {
            continue;
        };
        let signature = registry
            .symbol_registrations(&capability.abi, symbol)
            .iter()
            .find_map(|registration| match registration {
                HostFfiSymbolRegistration::Signature(signature)
                    if ffi_symbol_signature_hash(&capability.abi, symbol, signature)
                        == *signature_hash =>
                {
                    Some(signature.clone())
                }
                _ => None,
            })
            .ok_or_else(|| {
                format!(
                    "registered host FFI destructor `{symbol}` ABI `{}` hash `{signature_hash}` has no exact signature authority",
                    capability.abi
                )
            })?;
        authorities.insert(
            (
                capability.abi.clone(),
                symbol.clone(),
                signature_hash.clone(),
            ),
            signature,
        );
    }
    Ok(())
}

fn render_memory_capabilities(capabilities: &[HostFfiMemoryCapability]) -> String {
    if capabilities.is_empty() {
        "-".to_owned()
    } else {
        capabilities
            .iter()
            .map(HostFfiMemoryCapability::render)
            .collect::<Vec<_>>()
            .join(";")
    }
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
