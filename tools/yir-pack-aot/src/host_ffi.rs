use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use yir_core::ffi::ffi_symbol_signature_hash;
use yir_core::ffi::is_ffi_symbol_hash_token;
use yir_core::ffi::FFI_SYMBOL_HASH_PREFIX;
use yir_core::YirModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostFfiReturnType {
    I64,
    I32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostFfiArgType {
    I64,
    I32,
    RefBuffer,
}

pub(super) trait HostFfiScalarType {
    fn token(self) -> &'static str;
    fn c_type(self) -> &'static str;
}

impl HostFfiScalarType for HostFfiReturnType {
    fn token(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
        }
    }

    fn c_type(self) -> &'static str {
        match self {
            Self::I64 => "int64_t",
            Self::I32 => "int32_t",
        }
    }
}

impl HostFfiScalarType for HostFfiArgType {
    fn token(self) -> &'static str {
        match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
            Self::RefBuffer => "ref_Buffer",
        }
    }

    fn c_type(self) -> &'static str {
        match self {
            Self::I64 => "int64_t",
            Self::I32 => "int32_t",
            Self::RefBuffer => "void *",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HostFfiSignature {
    pub(super) abi: String,
    pub(super) return_type: HostFfiReturnType,
    pub(super) arg_types: Vec<HostFfiArgType>,
}

const PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES: &[&str] = &[
    "nurs:ffi_symbol:HostRenderCurves__color_bias=i64(i64)|ffi_symbol:HostRenderCurves__speed_curve=i64(i64)|ffi_symbol:HostRenderCurves__radius_curve=i64(i64)|ffi_symbol:HostRenderCurves__mix_tick=i64(i64,i64)|ffi_symbol:HostMath__speed_curve=i64(i64)",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HostFfiRegistryLines {
    pub(super) source: String,
    pub(super) lines: Vec<String>,
}

pub(super) fn default_host_ffi_registry() -> HostFfiRegistryLines {
    load_default_host_ffi_registry().unwrap_or_else(|error| HostFfiRegistryLines {
        source: format!("packer-builtin-fallback:{error}"),
        lines: PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES
            .iter()
            .map(|line| line.to_string())
            .collect(),
    })
}

fn load_default_host_ffi_registry() -> Result<HostFfiRegistryLines, String> {
    let manifest_path = host_ffi_registry_manifest_candidates()
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| {
            "could not find `nustar-packages/cpu.toml` for host FFI registry loading".to_owned()
        })?;
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let mut lines = PACKER_BUILTIN_HOST_FFI_REGISTRY_LINES
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    lines.extend(
        parse_manifest_string_array(&source, "abi_capabilities").ok_or_else(|| {
            format!(
                "manifest `{}` does not declare `abi_capabilities`",
                manifest_path.display()
            )
        })?,
    );
    parse_host_ffi_registry_lines(&lines)?;
    Ok(HostFfiRegistryLines {
        source: format!("cpu-manifest:{}", manifest_path.display()),
        lines,
    })
}

fn host_ffi_registry_manifest_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(path) = env::var("NUIS_HOST_FFI_REGISTRY_MANIFEST") {
        out.push(PathBuf::from(path));
    }
    out.push(PathBuf::from("nustar-packages/cpu.toml"));
    out.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../nustar-packages/cpu.toml"));
    out
}

pub(super) fn parse_manifest_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let key_start = source.find(&format!("{key} = ["))?;
    let after_open = &source[key_start..].split_once('[')?.1;
    let mut in_string = false;
    let mut escaped = false;
    let mut end_index = None;
    for (index, ch) in after_open.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ']' if !in_string => {
                end_index = Some(index);
                break;
            }
            _ => {}
        }
    }
    let body = &after_open[..end_index?];
    Some(split_quoted_array_items(body))
}

fn split_quoted_array_items(value: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for ch in value.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                push_manifest_array_item(&mut values, &current);
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    push_manifest_array_item(&mut values, &current);
    values
}

fn push_manifest_array_item(values: &mut Vec<String>, raw: &str) {
    let item = raw.trim().trim_matches('"').trim();
    if !item.is_empty() {
        values.push(item.to_owned());
    }
}

impl HostFfiSignature {
    pub(super) fn arg_count(&self) -> usize {
        self.arg_types.len()
    }

    pub(super) fn render(&self) -> String {
        format!(
            "{}({})",
            self.return_type.token(),
            self.arg_types
                .iter()
                .map(|arg| arg.token())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    pub(super) fn hash(&self, symbol: &str) -> String {
        ffi_symbol_signature_hash(&self.abi, symbol, &self.render())
    }
}

pub(super) fn validate_host_ffi_symbols(module: &YirModule) -> Result<(), String> {
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            return Err(format!(
                "cpu.{} node `{}` must include abi and symbol arguments",
                node.op.instruction, node.name
            ));
        }
        let abi = node.op.args[0].as_str();
        if abi != "nurs" && abi != "c" && abi != "libc" {
            return Err(format!(
                "cpu.{} node `{}` uses unsupported abi `{abi}`; expected `nurs`, `c`, or `libc`",
                node.op.instruction, node.name
            ));
        }
        let symbol = node.op.args[1].as_str();
        if symbol.trim().is_empty() {
            return Err(format!(
                "cpu.{} node `{}` has an empty symbol name",
                node.op.instruction, node.name
            ));
        }
    }

    for (symbol, signature) in collect_host_ffi_symbols(module)? {
        let expected = if symbol.ends_with("color_bias")
            || symbol.ends_with("speed_curve")
            || symbol.ends_with("radius_curve")
        {
            Some(1usize)
        } else if symbol.ends_with("mix_tick") {
            Some(2usize)
        } else {
            None
        };
        if let Some(expected_arg_count) = expected {
            if signature.arg_count() != expected_arg_count {
                return Err(format!(
                    "host ffi symbol `{symbol}` expects {expected_arg_count} argument(s) but YIR uses {}",
                    signature.arg_count()
                ));
            }
        }
    }

    Ok(())
}

pub(super) fn collect_host_ffi_symbols(
    module: &YirModule,
) -> Result<BTreeMap<String, HostFfiSignature>, String> {
    let mut out = BTreeMap::new();
    let nodes_by_name = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            continue;
        }
        let abi = node.op.args[0].clone();
        let symbol = node.op.args[1].clone();
        let signature = HostFfiSignature {
            abi: abi.clone(),
            return_type: host_ffi_return_type(&node.op.instruction).ok_or_else(|| {
                format!(
                    "cpu.{} node `{}` has no known host FFI return type",
                    node.op.instruction, node.name
                )
            })?,
            arg_types: node
                .op
                .args
                .iter()
                .skip(2)
                .map(|arg| host_ffi_arg_type_for_input(&abi, arg, &nodes_by_name))
                .collect(),
        };
        if let Some(existing) = out.insert(symbol.clone(), signature.clone()) {
            if existing != signature {
                return Err(format!(
                    "host ffi symbol `{symbol}` is used with conflicting signatures: {} and {}",
                    existing.render(),
                    signature.render()
                ));
            }
        }
    }
    Ok(out)
}

fn is_cpu_extern_call_instruction(instruction: &str) -> bool {
    matches!(instruction, "extern_call_i64" | "extern_call_i32")
}

fn host_ffi_return_type(instruction: &str) -> Option<HostFfiReturnType> {
    match instruction {
        "extern_call_i64" => Some(HostFfiReturnType::I64),
        "extern_call_i32" => Some(HostFfiReturnType::I32),
        _ => None,
    }
}

fn host_ffi_arg_type_for_input(
    abi: &str,
    input: &str,
    nodes_by_name: &BTreeMap<&str, &yir_core::Node>,
) -> HostFfiArgType {
    nodes_by_name
        .get(input)
        .map(|node| host_ffi_arg_type_for_node(abi, node))
        .unwrap_or(HostFfiArgType::I64)
}

fn host_ffi_arg_type_for_node(abi: &str, node: &yir_core::Node) -> HostFfiArgType {
    if node.op.module != "cpu" {
        return HostFfiArgType::I64;
    }
    match node.op.instruction.as_str() {
        "const_i32" | "cast_i64_to_i32" | "extern_call_i32" | "call_i32" | "param_i32" => {
            HostFfiArgType::I32
        }
        "text" | "null" | "borrow" | "move_ptr" | "alloc_node" | "alloc_buffer" | "load_next"
            if abi == "libc" =>
        {
            HostFfiArgType::RefBuffer
        }
        _ => HostFfiArgType::I64,
    }
}

pub(super) fn render_host_ffi_symbol_manifest(
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> String {
    host_ffi_symbols
        .iter()
        .map(|(symbol, signature)| format!("{}@{}:{}", symbol, signature.abi, signature.render()))
        .collect::<Vec<_>>()
        .join(";")
}

pub(super) fn render_host_ffi_symbol_hash_manifest(
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> String {
    host_ffi_symbols
        .iter()
        .map(|(symbol, signature)| format!("{}:{}", symbol, signature.hash(symbol)))
        .collect::<Vec<_>>()
        .join(";")
}

pub(super) fn host_ffi_footprint_hash(symbol_list: &str, hash_list: &str) -> String {
    fnv1a64_hash(&format!("nuis-ffi-footprint-v1|{symbol_list}|{hash_list}"))
}

pub(super) fn host_ffi_registry_hash(registry_lines: &[String]) -> String {
    let mut canonical_lines = registry_lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    canonical_lines.sort();
    fnv1a64_hash(&format!(
        "nuis-ffi-registry-v1|{}",
        canonical_lines.join("|")
    ))
}

pub(super) fn host_ffi_registry_abis(
    registry: &BTreeMap<(String, String), Vec<HostFfiRegistration>>,
) -> String {
    let abis = registry
        .keys()
        .map(|(abi, _)| abi.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if abis.is_empty() {
        "none".to_owned()
    } else {
        abis.join(",")
    }
}

fn fnv1a64_hash(value: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{FFI_SYMBOL_HASH_PREFIX}{hash:016x}")
}

pub(super) fn append_host_ffi_manifest_entries(
    manifest: &mut Vec<String>,
    host_ffi_symbols: &BTreeMap<String, HostFfiSignature>,
) -> Result<(), String> {
    if host_ffi_symbols.is_empty() {
        manifest.push("host_ffi_symbols=none".to_owned());
        manifest.push("host_ffi_symbol_hashes=none".to_owned());
        manifest.push("host_ffi_footprint_hash=none".to_owned());
        manifest.push("host_ffi_used_symbols=0".to_owned());
        manifest.push("host_ffi_used_abis=none".to_owned());
        manifest.push("host_ffi_registry_source=none".to_owned());
        manifest.push("host_ffi_registry_lines=0".to_owned());
        manifest.push("host_ffi_registry_symbols=0".to_owned());
        manifest.push("host_ffi_registry_abis=none".to_owned());
        manifest.push("host_ffi_registry_hash=none".to_owned());
        return Ok(());
    }

    let symbol_list = render_host_ffi_symbol_manifest(host_ffi_symbols);
    let hash_list = render_host_ffi_symbol_hash_manifest(host_ffi_symbols);
    verify_host_ffi_manifest_lines(&symbol_list, &hash_list)?;
    let registry = default_host_ffi_registry();
    let registry_view = parse_host_ffi_registry_lines(&registry.lines)?;
    let registry_symbols = registry_view.len();
    let registry_abis = host_ffi_registry_abis(&registry_view);
    verify_host_ffi_manifest_against_registry(&symbol_list, &hash_list, &registry_view)?;
    let used_abis = host_ffi_symbols
        .values()
        .map(|signature| signature.abi.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(",");
    manifest.push(format!("host_ffi_symbols={symbol_list}"));
    manifest.push(format!("host_ffi_symbol_hashes={hash_list}"));
    manifest.push(format!(
        "host_ffi_footprint_hash={}",
        host_ffi_footprint_hash(&symbol_list, &hash_list)
    ));
    manifest.push(format!("host_ffi_used_symbols={}", host_ffi_symbols.len()));
    manifest.push(format!("host_ffi_used_abis={used_abis}"));
    manifest.push(format!("host_ffi_registry_source={}", registry.source));
    manifest.push(format!("host_ffi_registry_lines={}", registry.lines.len()));
    manifest.push(format!("host_ffi_registry_symbols={registry_symbols}"));
    manifest.push(format!("host_ffi_registry_abis={registry_abis}"));
    manifest.push(format!(
        "host_ffi_registry_hash={}",
        host_ffi_registry_hash(&registry.lines)
    ));
    Ok(())
}

pub(super) fn verify_host_ffi_manifest_lines(
    symbol_list: &str,
    hash_list: &str,
) -> Result<(), String> {
    let symbols = parse_host_ffi_symbol_manifest_entries(symbol_list)?;
    let hashes = parse_host_ffi_hash_manifest_entries(hash_list)?;
    for (symbol, (abi, signature)) in &symbols {
        let Some(actual_hash) = hashes.get(symbol) else {
            return Err(format!(
                "host ffi manifest is missing hash for symbol `{symbol}`"
            ));
        };
        let expected_hash = ffi_symbol_signature_hash(abi, symbol, signature);
        if actual_hash != &expected_hash {
            return Err(format!(
                "host ffi manifest hash mismatch for `{symbol}`: expected {expected_hash}, found {actual_hash}"
            ));
        }
    }
    for symbol in hashes.keys() {
        if !symbols.contains_key(symbol) {
            return Err(format!(
                "host ffi manifest contains hash for undeclared symbol `{symbol}`"
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum HostFfiRegistration {
    Signature(String),
    Hash(String),
}

impl HostFfiRegistration {
    fn allows(&self, signature: &str, hash: &str) -> bool {
        match self {
            Self::Signature(expected) => expected == signature,
            Self::Hash(expected) => expected == hash,
        }
    }
}

#[cfg(test)]
pub(super) fn verify_host_ffi_manifest_against_registry_lines(
    symbol_list: &str,
    hash_list: &str,
    registry_lines: &[String],
) -> Result<(), String> {
    let registry = parse_host_ffi_registry_lines(registry_lines)?;
    verify_host_ffi_manifest_against_registry(symbol_list, hash_list, &registry)
}

pub(super) fn verify_host_ffi_manifest_against_registry(
    symbol_list: &str,
    hash_list: &str,
    registry: &BTreeMap<(String, String), Vec<HostFfiRegistration>>,
) -> Result<(), String> {
    let symbols = parse_host_ffi_symbol_manifest_entries(symbol_list)?;
    let hashes = parse_host_ffi_hash_manifest_entries(hash_list)?;
    for (symbol, (abi, signature)) in &symbols {
        let Some(actual_hash) = hashes.get(symbol) else {
            return Err(format!(
                "host ffi manifest is missing hash for symbol `{symbol}`"
            ));
        };
        let allowed = registry
            .get(&(abi.clone(), symbol.clone()))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if allowed.is_empty() {
            return Err(format!(
                "host ffi symbol `{symbol}` ABI `{abi}` is not registered by the packer host FFI registry"
            ));
        }
        if !allowed
            .iter()
            .any(|entry| entry.allows(signature, actual_hash))
        {
            return Err(format!(
                "host ffi symbol `{symbol}` ABI `{abi}` signature `{signature}` hash `{actual_hash}` is not allowed by the packer host FFI registry"
            ));
        }
    }
    Ok(())
}

pub(super) fn parse_host_ffi_registry_lines(
    registry_lines: &[String],
) -> Result<BTreeMap<(String, String), Vec<HostFfiRegistration>>, String> {
    let mut out = BTreeMap::new();
    for raw in registry_lines {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "invalid host ffi registry entry `{raw}`; expected `abi:capability[...]`"
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "invalid host ffi registry entry `{raw}`; ABI is required"
            ));
        }
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(entry) = cap.strip_prefix("ffi_symbol:") {
                let Some((symbol, signature)) = entry.split_once('=') else {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; expected `ffi_symbol:symbol=signature`"
                    ));
                };
                out.entry((abi.to_owned(), symbol.trim().to_owned()))
                    .or_insert_with(Vec::new)
                    .push(HostFfiRegistration::Signature(signature.trim().to_owned()));
            } else if let Some(entry) = cap.strip_prefix("ffi_symbol_hash:") {
                let Some((symbol, hash)) = entry.split_once('=') else {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; expected `ffi_symbol_hash:symbol=fnv1a64:<hex>`"
                    ));
                };
                if !is_ffi_symbol_hash_token(hash.trim()) {
                    return Err(format!(
                        "invalid host ffi registry capability `{cap}`; hash must use `fnv1a64:<hex>`"
                    ));
                }
                out.entry((abi.to_owned(), symbol.trim().to_owned()))
                    .or_insert_with(Vec::new)
                    .push(HostFfiRegistration::Hash(hash.trim().to_owned()));
            }
        }
    }
    Ok(out)
}

fn parse_host_ffi_symbol_manifest_entries(
    value: &str,
) -> Result<BTreeMap<String, (String, String)>, String> {
    let mut out = BTreeMap::new();
    if value == "none" || value.trim().is_empty() {
        return Ok(out);
    }
    for entry in value.split(';') {
        let Some((symbol_abi, signature)) = entry.split_once(':') else {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; expected `symbol@abi:signature`"
            ));
        };
        let Some((symbol, abi)) = symbol_abi.split_once('@') else {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; expected `symbol@abi:signature`"
            ));
        };
        if symbol.trim().is_empty() || abi.trim().is_empty() || signature.trim().is_empty() {
            return Err(format!(
                "invalid host_ffi_symbols entry `{entry}`; symbol, abi, and signature are required"
            ));
        }
        out.insert(symbol.to_owned(), (abi.to_owned(), signature.to_owned()));
    }
    Ok(out)
}

fn parse_host_ffi_hash_manifest_entries(value: &str) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    if value == "none" || value.trim().is_empty() {
        return Ok(out);
    }
    for entry in value.split(';') {
        let Some((symbol, payload)) = entry.split_once(':') else {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; expected `symbol:fnv1a64:<hex>`"
            ));
        };
        if symbol.trim().is_empty() || payload.trim().is_empty() {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; symbol and hash are required"
            ));
        }
        if !is_ffi_symbol_hash_token(payload) {
            return Err(format!(
                "invalid host_ffi_symbol_hashes entry `{entry}`; hash must use `fnv1a64:<hex>`"
            ));
        }
        out.insert(symbol.to_owned(), payload.to_owned());
    }
    Ok(out)
}
