use crate::registry::HostFfiRegistryView;
use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef};
use yir_core::ffi::ffi_symbol_signature_hash;

pub(super) fn validate_externs(
    ast: &AstModule,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    if ast.externs.is_empty() && ast.extern_interfaces.is_empty() {
        return Ok(());
    }
    if ast.domain != "cpu" {
        return Err(
            "extern declarations are currently only supported inside `mod cpu <unit>`".to_owned(),
        );
    }
    for function in ast.externs.iter().chain(
        ast.extern_interfaces
            .iter()
            .flat_map(|item| item.methods.iter()),
    ) {
        if !lowering_manifest
            .host_ffi_abis
            .iter()
            .any(|abi| abi == &function.abi)
        {
            return Err(format!(
                "extern ABI `{}` is not registered by nustar package `{}` for mod domain `{}`",
                function.abi, lowering_manifest.package_id, ast.domain
            ));
        }
        validate_extern_signature_allowlist(function, lowering_manifest)?;
    }
    Ok(())
}

fn validate_extern_signature_allowlist(
    function: &AstExternFunction,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    let ffi_registry = HostFfiRegistryView::from_manifest(lowering_manifest);
    let signature = extern_signature_pattern(function);
    let symbol = extern_ffi_symbol_name(function);
    let symbol_allowlist = ffi_registry.symbol_registrations(&function.abi, &symbol);
    if !symbol_allowlist.is_empty() {
        let actual_hash = ffi_symbol_signature_hash(&function.abi, &symbol, &signature);
        if symbol_allowlist.iter().any(|entry| {
            entry.matches(
                |pattern| ffi_signature_pattern_matches(pattern, &signature),
                &actual_hash,
            )
        }) {
            return Ok(());
        }
        return Err(format!(
            "extern `{}` ABI `{}` symbol `{}` signature `{}` hash `{}` is not allowed by nustar package `{}`; allowed symbol registrations: {}",
            function.name,
            function.abi,
            symbol,
            signature,
            actual_hash,
            lowering_manifest.package_id,
            symbol_allowlist
                .iter()
                .map(|entry| entry.render())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let allowed = host_ffi_signature_allowlist(lowering_manifest, &ffi_registry, &function.abi)?;
    if allowed
        .iter()
        .any(|pattern| ffi_signature_pattern_matches(pattern, &signature))
    {
        return Ok(());
    }
    Err(format!(
        "extern `{}` ABI `{}` signature `{}` is not allowed by nustar package `{}`; allowed signatures: {}",
        function.name,
        function.abi,
        signature,
        lowering_manifest.package_id,
        allowed.join(", ")
    ))
}

fn extern_ffi_symbol_name(function: &AstExternFunction) -> String {
    if let Some(symbol) = &function.host_symbol {
        return symbol.clone();
    }
    match &function.interface {
        Some(interface) => format!("{interface}__{}", function.name),
        None => function.name.clone(),
    }
}

fn host_ffi_signature_allowlist(
    lowering_manifest: &crate::registry::NustarPackageManifest,
    ffi_registry: &HostFfiRegistryView,
    abi: &str,
) -> Result<Vec<String>, String> {
    if !ffi_registry.has_abi(abi) {
        return Err(format!(
            "extern ABI `{}` has no abi_capabilities mapping in nustar package `{}`",
            abi, lowering_manifest.package_id
        ));
    }
    let out = ffi_registry.signature_families(abi).to_vec();
    if out.is_empty() {
        return Err(format!(
            "extern ABI `{}` in nustar package `{}` has no `ffi:` signature allowlist entries",
            abi, lowering_manifest.package_id
        ));
    }
    Ok(out)
}

fn extern_signature_pattern(function: &AstExternFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| ffi_type_token(&param.ty))
        .collect::<Vec<_>>();
    format!(
        "{}({})",
        ffi_type_token(&function.return_type),
        params.join(",")
    )
}

fn ffi_type_token(ty: &AstTypeRef) -> String {
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

fn render_ast_type_ref(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type_ref)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
}

fn ffi_signature_pattern_matches(pattern: &str, signature: &str) -> bool {
    if pattern == "*" || pattern == signature {
        return true;
    }
    let Some((pattern_return, pattern_params)) = pattern.split_once('(') else {
        return false;
    };
    let Some((signature_return, signature_params)) = signature.split_once('(') else {
        return false;
    };
    if pattern_return != "*" && pattern_return != signature_return {
        return false;
    }
    let pattern_params = pattern_params.trim_end_matches(')');
    let signature_params = signature_params.trim_end_matches(')');
    if pattern_params == "*" {
        return true;
    }
    ffi_param_pattern_matches(pattern_params, signature_params)
}

fn ffi_param_pattern_matches(pattern_params: &str, signature_params: &str) -> bool {
    if pattern_params == signature_params {
        return true;
    }
    let pattern = split_ffi_params(pattern_params);
    let signature = split_ffi_params(signature_params);
    if pattern.last().is_some_and(|item| *item == "*") {
        let prefix = &pattern[..pattern.len().saturating_sub(1)];
        return signature.len() >= prefix.len()
            && prefix
                .iter()
                .zip(signature.iter())
                .all(|(pattern, actual)| pattern == actual);
    }
    pattern.len() == signature.len()
        && pattern
            .iter()
            .zip(signature.iter())
            .all(|(pattern, actual)| *pattern == "*" || pattern == actual)
}

fn split_ffi_params(params: &str) -> Vec<&str> {
    if params.is_empty() {
        Vec::new()
    } else {
        params.split([',', '+']).collect()
    }
}
