use crate::registry::{HostFfiMemoryKind, HostFfiMemorySlot, HostFfiRegistryView};
use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef};
use yir_core::ffi::ffi_symbol_signature_hash;

pub(super) fn validate_externs(
    ast: &AstModule,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    if ast.externs.is_empty() && ast.extern_interfaces.is_empty() {
        return Ok(());
    }
    if ast.domain != "cffi" {
        return Err(
            "extern declarations must be wrapped by a registered `mod cffi <unit>` boundary"
                .to_owned(),
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

pub(super) fn validate_benchmark_harness_externs(
    ast: &AstModule,
    lowering_manifest: &crate::registry::NustarPackageManifest,
) -> Result<(), String> {
    const GENERATED_SYMBOLS: [&str; 5] = [
        "host_monotonic_time_ns",
        "host_serialize_i64_into",
        "host_deserialize_text_from",
        "host_text_len",
        "host_stdout_write",
    ];
    if ast.domain == "cffi" {
        return validate_externs(ast, lowering_manifest);
    }
    if ast.domain != "cpu" || !ast.extern_interfaces.is_empty() {
        return Err(
            "benchmark harness host services require a generated `mod cpu` boundary".to_owned(),
        );
    }
    if ast.externs.len() != GENERATED_SYMBOLS.len()
        || GENERATED_SYMBOLS.iter().any(|name| {
            ast.externs
                .iter()
                .filter(|function| function.name == *name)
                .count()
                != 1
        })
    {
        return Err(
            "benchmark harness host service set does not match the compiler-owned contract"
                .to_owned(),
        );
    }
    for function in &ast.externs {
        if function.abi != "c" || function.interface.is_some() || function.host_symbol.is_some() {
            return Err(format!(
                "benchmark harness host service `{}` has an invalid generated ABI surface",
                function.name
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
    let ffi_registry = HostFfiRegistryView::try_from_manifest(lowering_manifest)?;
    let signature = extern_signature_pattern(function);
    let symbol = extern_ffi_symbol_name(function);
    let actual_hash = ffi_symbol_signature_hash(&function.abi, &symbol, &signature);
    let symbol_allowlist = ffi_registry.symbol_registrations(&function.abi, &symbol);
    if !symbol_allowlist.is_empty() {
        if symbol_allowlist.iter().any(|entry| {
            entry.matches(
                |pattern| ffi_signature_pattern_matches(pattern, &signature),
                &actual_hash,
            )
        }) {
            validate_extern_memory_capabilities(function, &ffi_registry, &symbol, &actual_hash)?;
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
        validate_extern_memory_capabilities(function, &ffi_registry, &symbol, &actual_hash)?;
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

fn validate_extern_memory_capabilities(
    function: &AstExternFunction,
    registry: &HostFfiRegistryView,
    symbol: &str,
    signature_hash: &str,
) -> Result<(), String> {
    let capabilities = registry.memory_capabilities(&function.abi, symbol, signature_hash);
    for (index, param) in function.params.iter().enumerate() {
        if param.ty.name != "String" || param.ty.is_ref || param.ty.is_optional {
            continue;
        }
        if !capabilities.iter().any(|capability| {
            capability.kind == HostFfiMemoryKind::BorrowedUtf8
                && capability.slot == HostFfiMemorySlot::Arg(index)
        }) {
            return Err(format!(
                "extern `{}` ABI `{}` symbol `{symbol}` String parameter `{}` at arg:{index} requires a hash-bound `borrowed_utf8` host FFI memory capability before lowering",
                function.name, function.abi, param.name
            ));
        }
    }

    if function.return_type.name == "Buffer"
        && function.return_type.is_ref
        && !function.return_type.is_optional
    {
        let registered = capabilities.iter().any(|capability| {
            capability.kind == HostFfiMemoryKind::OwnedReturnBuffer
                && capability.slot == HostFfiMemorySlot::Return
        });
        if !registered {
            return Err(format!(
                "extern `{}` ABI `{}` symbol `{symbol}` ref Buffer return requires a hash-bound `owned_return_buffer` host FFI memory capability before lowering",
                function.name, function.abi
            ));
        }
    }
    if function.return_type.name == "String"
        && function.return_type.is_ref
        && !function.return_type.is_optional
    {
        let registered = capabilities.iter().any(|capability| {
            capability.kind == HostFfiMemoryKind::OwnedReturnUtf8
                && capability.slot == HostFfiMemorySlot::Return
        });
        if !registered {
            return Err(format!(
                "extern `{}` ABI `{}` symbol `{symbol}` ref String return requires a hash-bound `owned_return_utf8` host FFI memory capability before lowering",
                function.name, function.abi
            ));
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::HostFfiMemoryCapability;

    fn cffi_manifest() -> crate::registry::NustarPackageManifest {
        crate::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), "cffi")
            .unwrap()
    }

    #[test]
    fn rejects_string_extern_without_hash_bound_memory_capability() {
        let ast = crate::frontend::parse_nuis_ast(
            r#"
            mod cffi Main {
              extern "libc" fn puts(message: String) -> i32;
              fn main() -> i64 { return 0; }
            }
            "#,
        )
        .unwrap();
        let mut manifest = cffi_manifest();
        manifest
            .host_ffi_memory_capabilities
            .retain(|entry| !entry.starts_with("libc:puts@"));

        let error = validate_externs(&ast, &manifest)
            .expect_err("String extern without memory authority must be rejected");

        assert!(error.contains("String parameter `message` at arg:0"));
        assert!(error.contains("requires a hash-bound `borrowed_utf8`"));
        assert!(error.contains("before lowering"));
    }

    #[test]
    fn accepts_owned_return_buffer_after_contract_validation() {
        let ast = crate::frontend::parse_nuis_ast(
            r#"
            mod cffi Main {
              extern "c" fn test_buffer_acquire(seed: i64) -> ref Buffer;
              fn main() -> i64 { return 0; }
            }
            "#,
        )
        .unwrap();
        let mut manifest = cffi_manifest();
        let acquire_hash = ffi_symbol_signature_hash("c", "test_buffer_acquire", "ref_Buffer(i64)");
        let release_hash = ffi_symbol_signature_hash("c", "test_buffer_release", "i64(ref_Buffer)");
        manifest.abi_capabilities.push(
            "c:ffi_symbol:test_buffer_acquire=ref_Buffer(i64)|ffi_symbol:test_buffer_release=i64(ref_Buffer)"
                .to_owned(),
        );
        manifest.host_ffi_memory_capabilities.push(
            HostFfiMemoryCapability::owned_return_buffer(
                "c",
                "test_buffer_acquire",
                &acquire_hash,
                "test_buffer_release",
                &release_hash,
            )
            .render(),
        );

        validate_externs(&ast, &manifest)
            .expect("an exact owned return capability should open frontend validation");
    }
}
