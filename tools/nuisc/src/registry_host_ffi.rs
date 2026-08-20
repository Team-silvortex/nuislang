use std::collections::{BTreeMap, BTreeSet};

use crate::registry::NustarPackageManifest;
use yir_core::ffi::{
    ffi_memory_capability_hash, ffi_symbol_signature_hash, is_ffi_symbol_hash_token,
    OWNED_BUFFER_DESTRUCTOR_SIGNATURE, OWNED_OBJECT_DESTRUCTOR_SIGNATURE,
    OWNED_OBJECT_RETURN_READ_POLICY, OWNED_OBJECT_RETURN_SIZE_POLICY,
    OWNED_UTF8_DESTRUCTOR_SIGNATURE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostFfiMemoryKind {
    BorrowedUtf8,
    OwnedReturnBuffer,
    OwnedReturnObject,
    OwnedReturnUtf8,
}

impl HostFfiMemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BorrowedUtf8 => "borrowed_utf8",
            Self::OwnedReturnBuffer => "owned_return_buffer",
            Self::OwnedReturnObject => "owned_return_object",
            Self::OwnedReturnUtf8 => "owned_return_utf8",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostFfiMemorySlot {
    Arg(usize),
    Return,
}

impl HostFfiMemorySlot {
    pub fn render(&self) -> String {
        match self {
            Self::Arg(index) => format!("arg:{index}"),
            Self::Return => "return".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostFfiMemoryDestructor {
    None,
    Registered {
        symbol: String,
        signature_hash: String,
    },
}

impl HostFfiMemoryDestructor {
    pub fn render(&self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Registered {
                symbol,
                signature_hash,
            } => format!("{symbol}@{signature_hash}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostFfiMemoryCapability {
    pub abi: String,
    pub symbol: String,
    pub signature_hash: String,
    pub capability_hash: String,
    pub kind: HostFfiMemoryKind,
    pub slot: HostFfiMemorySlot,
    pub length: String,
    pub size: Option<String>,
    pub read: Option<String>,
    pub mutability: String,
    pub lifetime: String,
    pub destructor: HostFfiMemoryDestructor,
}

impl HostFfiMemoryCapability {
    pub fn borrowed_utf8(abi: &str, symbol: &str, signature_hash: &str, arg_index: usize) -> Self {
        Self::new(
            abi,
            symbol,
            signature_hash,
            HostFfiMemoryKind::BorrowedUtf8,
            HostFfiMemorySlot::Arg(arg_index),
            "nul_terminated",
            "read_only",
            "call",
            HostFfiMemoryDestructor::None,
        )
    }

    pub fn owned_return_buffer(
        abi: &str,
        symbol: &str,
        signature_hash: &str,
        destructor_symbol: &str,
        destructor_signature_hash: &str,
    ) -> Self {
        Self::new(
            abi,
            symbol,
            signature_hash,
            HostFfiMemoryKind::OwnedReturnBuffer,
            HostFfiMemorySlot::Return,
            "runtime_header",
            "unique",
            "owned",
            HostFfiMemoryDestructor::Registered {
                symbol: destructor_symbol.to_owned(),
                signature_hash: destructor_signature_hash.to_owned(),
            },
        )
    }

    pub fn owned_return_object(
        abi: &str,
        symbol: &str,
        signature_hash: &str,
        destructor_symbol: &str,
        destructor_signature_hash: &str,
    ) -> Self {
        let mut capability = Self {
            abi: abi.to_owned(),
            symbol: symbol.to_owned(),
            signature_hash: signature_hash.to_owned(),
            capability_hash: String::new(),
            kind: HostFfiMemoryKind::OwnedReturnObject,
            slot: HostFfiMemorySlot::Return,
            length: String::new(),
            size: Some(OWNED_OBJECT_RETURN_SIZE_POLICY.to_owned()),
            read: Some(OWNED_OBJECT_RETURN_READ_POLICY.to_owned()),
            mutability: "read_only".to_owned(),
            lifetime: "owned".to_owned(),
            destructor: HostFfiMemoryDestructor::Registered {
                symbol: destructor_symbol.to_owned(),
                signature_hash: destructor_signature_hash.to_owned(),
            },
        };
        capability.capability_hash = ffi_memory_capability_hash(
            &capability.abi,
            &capability.symbol,
            &capability.signature_hash,
            &capability.descriptor(),
        );
        capability
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        abi: &str,
        symbol: &str,
        signature_hash: &str,
        kind: HostFfiMemoryKind,
        slot: HostFfiMemorySlot,
        length: &str,
        mutability: &str,
        lifetime: &str,
        destructor: HostFfiMemoryDestructor,
    ) -> Self {
        let mut capability = Self {
            abi: abi.to_owned(),
            symbol: symbol.to_owned(),
            signature_hash: signature_hash.to_owned(),
            capability_hash: String::new(),
            kind,
            slot,
            length: length.to_owned(),
            size: None,
            read: None,
            mutability: mutability.to_owned(),
            lifetime: lifetime.to_owned(),
            destructor,
        };
        capability.capability_hash = ffi_memory_capability_hash(
            &capability.abi,
            &capability.symbol,
            &capability.signature_hash,
            &capability.descriptor(),
        );
        capability
    }

    pub fn descriptor(&self) -> String {
        if self.kind == HostFfiMemoryKind::OwnedReturnObject {
            return format!(
                "kind={},slot={},size={},read={},mutability={},lifetime={},destructor={}",
                self.kind.as_str(),
                self.slot.render(),
                self.size.as_deref().unwrap_or_default(),
                self.read.as_deref().unwrap_or_default(),
                self.mutability,
                self.lifetime,
                self.destructor.render()
            );
        }
        format!(
            "kind={},slot={},length={},mutability={},lifetime={},destructor={}",
            self.kind.as_str(),
            self.slot.render(),
            self.length,
            self.mutability,
            self.lifetime,
            self.destructor.render()
        )
    }

    pub fn render(&self) -> String {
        format!(
            "{}:{}@{}@{}={}",
            self.abi,
            self.symbol,
            self.signature_hash,
            self.capability_hash,
            self.descriptor()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostFfiSymbolRegistration {
    Signature(String),
    Hash(String),
}

impl HostFfiSymbolRegistration {
    pub fn render(&self) -> String {
        match self {
            Self::Signature(signature) => format!("signature:{signature}"),
            Self::Hash(hash) => format!("hash:{hash}"),
        }
    }

    pub fn matches(&self, signature_matches: impl FnOnce(&str) -> bool, actual_hash: &str) -> bool {
        match self {
            Self::Signature(pattern) => signature_matches(pattern),
            Self::Hash(expected) => expected == actual_hash,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostFfiRegistryView {
    signature_families: BTreeMap<String, Vec<String>>,
    symbol_registrations: BTreeMap<(String, String), Vec<HostFfiSymbolRegistration>>,
    memory_capabilities: BTreeMap<(String, String, String), Vec<HostFfiMemoryCapability>>,
}

impl HostFfiRegistryView {
    pub fn try_from_manifest(manifest: &NustarPackageManifest) -> Result<Self, String> {
        let mut view = Self::default();
        for raw in &manifest.abi_capabilities {
            let Some((abi, caps)) = raw.split_once(':') else {
                continue;
            };
            let abi = abi.trim();
            if abi.is_empty() {
                continue;
            }
            for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
                if let Some(pattern) = cap.strip_prefix("ffi:") {
                    view.signature_families
                        .entry(abi.to_owned())
                        .or_default()
                        .push(pattern.trim().to_owned());
                } else if let Some(entry) = cap.strip_prefix("ffi_symbol:") {
                    let Some((symbol, signature)) = entry.split_once('=') else {
                        continue;
                    };
                    view.symbol_registrations
                        .entry((abi.to_owned(), symbol.trim().to_owned()))
                        .or_default()
                        .push(HostFfiSymbolRegistration::Signature(
                            signature.trim().to_owned(),
                        ));
                } else if let Some(entry) = cap.strip_prefix("ffi_symbol_hash:") {
                    let Some((symbol, hash)) = entry.split_once('=') else {
                        continue;
                    };
                    view.symbol_registrations
                        .entry((abi.to_owned(), symbol.trim().to_owned()))
                        .or_default()
                        .push(HostFfiSymbolRegistration::Hash(hash.trim().to_owned()));
                }
            }
        }
        for raw in &manifest.host_ffi_memory_capabilities {
            let capability = parse_memory_capability(raw, &manifest.package_id)?;
            view.memory_capabilities
                .entry((
                    capability.abi.clone(),
                    capability.symbol.clone(),
                    capability.signature_hash.clone(),
                ))
                .or_default()
                .push(capability);
        }
        for values in view.signature_families.values_mut() {
            values.sort();
            values.dedup();
        }
        for values in view.symbol_registrations.values_mut() {
            values.sort_by_key(HostFfiSymbolRegistration::render);
            values.dedup();
        }
        for values in view.memory_capabilities.values_mut() {
            values.sort();
        }
        view.validate_memory_capabilities(manifest)?;
        Ok(view)
    }

    pub fn has_abi(&self, abi: &str) -> bool {
        self.signature_families.contains_key(abi)
            || self
                .symbol_registrations
                .keys()
                .any(|(entry_abi, _)| entry_abi == abi)
    }

    pub fn signature_families(&self, abi: &str) -> &[String] {
        self.signature_families
            .get(abi)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn symbol_registrations(&self, abi: &str, symbol: &str) -> &[HostFfiSymbolRegistration] {
        self.symbol_registrations
            .get(&(abi.to_owned(), symbol.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn memory_capabilities(
        &self,
        abi: &str,
        symbol: &str,
        signature_hash: &str,
    ) -> &[HostFfiMemoryCapability] {
        self.memory_capabilities
            .get(&(abi.to_owned(), symbol.to_owned(), signature_hash.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn validate_memory_capabilities(&self, manifest: &NustarPackageManifest) -> Result<(), String> {
        for ((abi, symbol, signature_hash), capabilities) in &self.memory_capabilities {
            if !manifest
                .host_ffi_abis
                .iter()
                .any(|candidate| candidate == abi)
            {
                return Err(format!(
                    "nustar package `{}` host FFI memory capability references undeclared ABI `{abi}`",
                    manifest.package_id
                ));
            }
            let signature = self
                .exact_signature_for_hash(abi, symbol, signature_hash)
                .ok_or_else(|| {
                    format!(
                        "nustar package `{}` host FFI memory capability for `{symbol}` ABI `{abi}` hash `{signature_hash}` requires a matching exact `ffi_symbol:` registration",
                        manifest.package_id
                    )
                })?;
            let mut seen = BTreeSet::new();
            for capability in capabilities {
                if !seen.insert((capability.kind, capability.slot.clone())) {
                    return Err(format!(
                        "nustar package `{}` repeats host FFI memory capability `{}` slot `{}` for `{symbol}` ABI `{abi}` hash `{signature_hash}`",
                        manifest.package_id,
                        capability.kind.as_str(),
                        capability.slot.render()
                    ));
                }
                validate_memory_capability_shape(
                    self,
                    capability,
                    &signature,
                    &manifest.package_id,
                )?;
            }
        }
        Ok(())
    }

    fn exact_signature_for_hash(
        &self,
        abi: &str,
        symbol: &str,
        expected_hash: &str,
    ) -> Option<String> {
        self.symbol_registrations(abi, symbol)
            .iter()
            .find_map(|registration| match registration {
                HostFfiSymbolRegistration::Signature(signature) if !signature.contains('*') => {
                    let signature = signature.replace('+', ",");
                    (ffi_symbol_signature_hash(abi, symbol, &signature) == expected_hash)
                        .then_some(signature)
                }
                _ => None,
            })
    }
}

pub(crate) fn parse_memory_capability(
    raw: &str,
    package_id: &str,
) -> Result<HostFfiMemoryCapability, String> {
    let (abi, registration) = raw.split_once(':').ok_or_else(|| {
        format!(
            "nustar package `{package_id}` has invalid host_ffi_memory_capabilities entry `{raw}`; expected `abi:symbol@signature_hash@capability_hash=descriptor`"
        )
    })?;
    let (identity, descriptor) = registration.split_once('=').ok_or_else(|| {
        format!(
            "nustar package `{package_id}` has invalid host_ffi_memory_capabilities entry `{raw}`; missing descriptor"
        )
    })?;
    let mut identity_parts = identity.rsplitn(3, '@');
    let capability_hash = identity_parts.next().unwrap_or_default();
    let signature_hash = identity_parts.next().unwrap_or_default();
    let symbol = identity_parts.next().unwrap_or_default();
    if abi.trim().is_empty()
        || symbol.trim().is_empty()
        || !is_ffi_symbol_hash_token(signature_hash)
        || !is_ffi_symbol_hash_token(capability_hash)
    {
        return Err(format!(
            "nustar package `{package_id}` has invalid host FFI memory capability identity `{identity}`"
        ));
    }
    let fields = parse_memory_descriptor_fields(descriptor, package_id)?;
    let kind = match required_descriptor_field(&fields, "kind", package_id)? {
        "borrowed_utf8" => HostFfiMemoryKind::BorrowedUtf8,
        "owned_return_buffer" => HostFfiMemoryKind::OwnedReturnBuffer,
        "owned_return_object" => HostFfiMemoryKind::OwnedReturnObject,
        "owned_return_utf8" => HostFfiMemoryKind::OwnedReturnUtf8,
        other => {
            return Err(format!(
                "nustar package `{package_id}` host FFI memory capability has unsupported kind `{other}`"
            ))
        }
    };
    let slot = parse_memory_slot(
        required_descriptor_field(&fields, "slot", package_id)?,
        package_id,
    )?;
    let destructor = parse_memory_destructor(
        required_descriptor_field(&fields, "destructor", package_id)?,
        package_id,
    )?;
    let object_policy = kind == HostFfiMemoryKind::OwnedReturnObject;
    let capability = HostFfiMemoryCapability {
        abi: abi.trim().to_owned(),
        symbol: symbol.trim().to_owned(),
        signature_hash: signature_hash.to_owned(),
        capability_hash: capability_hash.to_owned(),
        kind,
        slot,
        length: if object_policy {
            String::new()
        } else {
            required_descriptor_field(&fields, "length", package_id)?.to_owned()
        },
        size: object_policy
            .then(|| required_descriptor_field(&fields, "size", package_id).map(str::to_owned))
            .transpose()?,
        read: object_policy
            .then(|| required_descriptor_field(&fields, "read", package_id).map(str::to_owned))
            .transpose()?,
        mutability: required_descriptor_field(&fields, "mutability", package_id)?.to_owned(),
        lifetime: required_descriptor_field(&fields, "lifetime", package_id)?.to_owned(),
        destructor,
    };
    validate_memory_policy_fields(&capability, package_id)?;
    let expected_hash = ffi_memory_capability_hash(
        &capability.abi,
        &capability.symbol,
        &capability.signature_hash,
        &capability.descriptor(),
    );
    if capability.capability_hash != expected_hash {
        return Err(format!(
            "nustar package `{package_id}` host FFI memory capability hash mismatch for `{}`: expected `{expected_hash}`, found `{}`",
            capability.symbol, capability.capability_hash
        ));
    }
    Ok(capability)
}

fn parse_memory_descriptor_fields<'a>(
    descriptor: &'a str,
    package_id: &str,
) -> Result<BTreeMap<&'a str, &'a str>, String> {
    let mut fields = BTreeMap::new();
    for field in descriptor.split(',') {
        let (key, value) = field.split_once('=').ok_or_else(|| {
            format!(
                "nustar package `{package_id}` host FFI memory descriptor field `{field}` must use `key=value`"
            )
        })?;
        if !matches!(
            key,
            "kind" | "slot" | "length" | "size" | "read" | "mutability" | "lifetime" | "destructor"
        ) {
            return Err(format!(
                "nustar package `{package_id}` host FFI memory descriptor has unknown field `{key}`"
            ));
        }
        if value.is_empty() || fields.insert(key, value).is_some() {
            return Err(format!(
                "nustar package `{package_id}` host FFI memory descriptor has empty or repeated field `{key}`"
            ));
        }
    }
    Ok(fields)
}

fn required_descriptor_field<'a>(
    fields: &'a BTreeMap<&str, &'a str>,
    key: &str,
    package_id: &str,
) -> Result<&'a str, String> {
    fields.get(key).copied().ok_or_else(|| {
        format!("nustar package `{package_id}` host FFI memory descriptor is missing `{key}`")
    })
}

fn parse_memory_slot(value: &str, package_id: &str) -> Result<HostFfiMemorySlot, String> {
    if value == "return" {
        return Ok(HostFfiMemorySlot::Return);
    }
    let Some(index) = value.strip_prefix("arg:") else {
        return Err(format!(
            "nustar package `{package_id}` host FFI memory slot `{value}` must be `arg:N` or `return`"
        ));
    };
    index
        .parse::<usize>()
        .map(HostFfiMemorySlot::Arg)
        .map_err(|_| {
            format!("nustar package `{package_id}` has invalid host FFI argument slot `{value}`")
        })
}

fn parse_memory_destructor(
    value: &str,
    package_id: &str,
) -> Result<HostFfiMemoryDestructor, String> {
    if value == "none" {
        return Ok(HostFfiMemoryDestructor::None);
    }
    let (symbol, signature_hash) = value.rsplit_once('@').ok_or_else(|| {
        format!(
            "nustar package `{package_id}` host FFI destructor `{value}` must use `symbol@signature_hash`"
        )
    })?;
    if symbol.is_empty() || !is_ffi_symbol_hash_token(signature_hash) {
        return Err(format!(
            "nustar package `{package_id}` has invalid host FFI destructor `{value}`"
        ));
    }
    Ok(HostFfiMemoryDestructor::Registered {
        symbol: symbol.to_owned(),
        signature_hash: signature_hash.to_owned(),
    })
}

fn validate_memory_policy_fields(
    capability: &HostFfiMemoryCapability,
    package_id: &str,
) -> Result<(), String> {
    let valid = match capability.kind {
        HostFfiMemoryKind::BorrowedUtf8 => {
            matches!(capability.slot, HostFfiMemorySlot::Arg(_))
                && capability.length == "nul_terminated"
                && capability.size.is_none()
                && capability.read.is_none()
                && capability.mutability == "read_only"
                && capability.lifetime == "call"
                && capability.destructor == HostFfiMemoryDestructor::None
        }
        HostFfiMemoryKind::OwnedReturnBuffer => {
            capability.slot == HostFfiMemorySlot::Return
                && capability.length == "runtime_header"
                && capability.size.is_none()
                && capability.read.is_none()
                && capability.mutability == "unique"
                && capability.lifetime == "owned"
                && matches!(
                    capability.destructor,
                    HostFfiMemoryDestructor::Registered { .. }
                )
        }
        HostFfiMemoryKind::OwnedReturnUtf8 => {
            capability.slot == HostFfiMemorySlot::Return
                && capability.length == "runtime_header"
                && capability.size.is_none()
                && capability.read.is_none()
                && capability.mutability == "read_only"
                && capability.lifetime == "owned"
                && matches!(
                    capability.destructor,
                    HostFfiMemoryDestructor::Registered { .. }
                )
        }
        HostFfiMemoryKind::OwnedReturnObject => {
            capability.slot == HostFfiMemorySlot::Return
                && capability.length.is_empty()
                && capability.size.as_deref() == Some(OWNED_OBJECT_RETURN_SIZE_POLICY)
                && capability.read.as_deref() == Some(OWNED_OBJECT_RETURN_READ_POLICY)
                && capability.mutability == "read_only"
                && capability.lifetime == "owned"
                && matches!(
                    capability.destructor,
                    HostFfiMemoryDestructor::Registered { .. }
                )
        }
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "nustar package `{package_id}` host FFI memory capability `{}` has lifetime, length, mutability, slot, or destructor policy drift; object size/read policy may also have drifted",
            capability.kind.as_str()
        ))
    }
}

fn validate_memory_capability_shape(
    registry: &HostFfiRegistryView,
    capability: &HostFfiMemoryCapability,
    signature: &str,
    package_id: &str,
) -> Result<(), String> {
    match (&capability.kind, &capability.slot) {
        (HostFfiMemoryKind::BorrowedUtf8, HostFfiMemorySlot::Arg(index)) => {
            let args = signature_args(signature).ok_or_else(|| {
                format!("nustar package `{package_id}` has malformed FFI signature `{signature}`")
            })?;
            if args.get(*index).copied() != Some("String") {
                return Err(format!(
                    "nustar package `{package_id}` borrowed UTF-8 capability slot `arg:{index}` does not reference a `String` parameter in `{signature}`"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnBuffer, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_Buffer") {
                return Err(format!(
                    "nustar package `{package_id}` owned return-buffer capability requires `ref_Buffer` return signature, found `{signature}`"
                ));
            }
            let HostFfiMemoryDestructor::Registered {
                symbol,
                signature_hash,
            } = &capability.destructor
            else {
                unreachable!("owned return-buffer policy requires a registered destructor")
            };
            let destructor_signature = registry
                .exact_signature_for_hash(&capability.abi, symbol, signature_hash)
                .ok_or_else(|| {
                    format!(
                        "nustar package `{package_id}` owned return-buffer destructor `{symbol}` is not exact-signature registered for ABI `{}` hash `{signature_hash}`",
                        capability.abi
                    )
                })?;
            if destructor_signature != OWNED_BUFFER_DESTRUCTOR_SIGNATURE {
                return Err(format!(
                    "nustar package `{package_id}` owned return-buffer destructor `{symbol}` must use `{OWNED_BUFFER_DESTRUCTOR_SIGNATURE}`, found `{destructor_signature}`"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnUtf8, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_String") {
                return Err(format!(
                    "nustar package `{package_id}` owned UTF-8 capability requires `ref_String` return signature, found `{signature}`"
                ));
            }
            let HostFfiMemoryDestructor::Registered {
                symbol,
                signature_hash,
            } = &capability.destructor
            else {
                unreachable!("owned UTF-8 policy requires a registered destructor")
            };
            let destructor_signature = registry
                .exact_signature_for_hash(&capability.abi, symbol, signature_hash)
                .ok_or_else(|| {
                    format!(
                        "nustar package `{package_id}` owned UTF-8 destructor `{symbol}` is not exact-signature registered for ABI `{}` hash `{signature_hash}`",
                        capability.abi
                    )
                })?;
            if destructor_signature != OWNED_UTF8_DESTRUCTOR_SIGNATURE {
                return Err(format!(
                    "nustar package `{package_id}` owned UTF-8 destructor `{symbol}` must use `{OWNED_UTF8_DESTRUCTOR_SIGNATURE}`, found `{destructor_signature}`"
                ));
            }
        }
        (HostFfiMemoryKind::OwnedReturnObject, HostFfiMemorySlot::Return) => {
            if signature.split_once('(').map(|(ret, _)| ret) != Some("ref_FfiObject") {
                return Err(format!(
                    "nustar package `{package_id}` owned object capability requires `ref_FfiObject` return signature, found `{signature}`"
                ));
            }
            let HostFfiMemoryDestructor::Registered {
                symbol,
                signature_hash,
            } = &capability.destructor
            else {
                unreachable!("owned object policy requires a registered destructor")
            };
            let destructor_signature = registry
                .exact_signature_for_hash(&capability.abi, symbol, signature_hash)
                .ok_or_else(|| {
                    format!(
                        "nustar package `{package_id}` owned object destructor `{symbol}` is not exact-signature registered for ABI `{}` hash `{signature_hash}`",
                        capability.abi
                    )
                })?;
            if destructor_signature != OWNED_OBJECT_DESTRUCTOR_SIGNATURE {
                return Err(format!(
                    "nustar package `{package_id}` owned object destructor `{symbol}` must use `{OWNED_OBJECT_DESTRUCTOR_SIGNATURE}`, found `{destructor_signature}`"
                ));
            }
        }
        _ => unreachable!("memory policy validation already checked slot-kind alignment"),
    }
    Ok(())
}

fn signature_args(signature: &str) -> Option<Vec<&str>> {
    let (_, args) = signature.split_once('(')?;
    let args = args.strip_suffix(')')?;
    Some(if args.is_empty() {
        Vec::new()
    } else {
        args.split(',').collect()
    })
}
