use std::collections::BTreeSet;

use crate::model::{
    AstAttribute, AstAttributeValue, AstConstItem, AstEnumDef, AstEnumVariantKind, AstFunction,
    AstGenericParam, AstModule, AstStructDef, AstTypeAlias, AstTypeRef, AstWherePredicate,
};

#[path = "bootstrap_subset_walk.rs"]
mod walk;

pub const BOOTSTRAP_SUBSET_PROTOCOL: &str = "nuis-bootstrap-language-subset-v6";

pub const BOOTSTRAP_APPROVED_IMPORTS: &[(&str, &str)] = &[
    ("cpu", "CorePrelude"),
    ("cpu", "StdLanguageCore"),
    ("cpu", "StdCompilerData"),
    ("cpu", "StdCompilerTokenEmit"),
    ("cpu", "StdCompilerTokens"),
    ("cpu", "StdCompilerProjection"),
    ("cpu", "StdTextContracts"),
];

pub const BOOTSTRAP_PRIMITIVE_TYPES: &[&str] = &["bool", "i64", "text"];

pub const BOOTSTRAP_APPROVED_EXTERNAL_TYPES: &[&str] = &[
    "CompilerArena",
    "CompilerDiagnostic",
    "CompilerDecimalState",
    "CompilerMap",
    "CompilerPath",
    "CompilerProjectionKind",
    "CompilerProjectionRecordKind",
    "CompilerProjectionPageState",
    "CompilerProjectionState",
    "CompilerSourceSpan",
    "CompilerText",
    "CompilerTokenBuffer",
    "CompilerTokenMaterializer",
    "CompilerTokenRecord",
    "CompilerTokenStore",
    "CompilerVector",
    "Option",
    "Result",
];

const BOOTSTRAP_SCALAR_EXPORTS: &[(&str, &str, usize)] = &[
    (
        "compiler_candidate_stage_seed",
        "nuis_bootstrap_candidate_stage_seed_v1",
        1,
    ),
    (
        "compiler_candidate_stage_fold",
        "nuis_bootstrap_candidate_stage_fold_v1",
        3,
    ),
    (
        "compiler_candidate_bundle_seed",
        "nuis_bootstrap_candidate_bundle_seed_v1",
        0,
    ),
    (
        "compiler_candidate_bundle_fold",
        "nuis_bootstrap_candidate_bundle_fold_v1",
        3,
    ),
    (
        "compiler_candidate_token_start",
        "nuis_bootstrap_candidate_token_start_v1",
        0,
    ),
    (
        "compiler_candidate_token_error_mode",
        "nuis_bootstrap_candidate_token_error_mode_v1",
        0,
    ),
    (
        "compiler_candidate_token_max_bytes",
        "nuis_bootstrap_candidate_token_max_bytes_v1",
        0,
    ),
    (
        "compiler_candidate_token_semantic_seed",
        "nuis_bootstrap_candidate_token_semantic_seed_v1",
        0,
    ),
    (
        "compiler_candidate_token_step",
        "nuis_bootstrap_candidate_token_step_v1",
        2,
    ),
    (
        "compiler_candidate_token_count_step",
        "nuis_bootstrap_candidate_token_count_step_v1",
        3,
    ),
    (
        "compiler_candidate_token_semantic_step",
        "nuis_bootstrap_candidate_token_semantic_step_v1",
        3,
    ),
    (
        "compiler_candidate_token_finish",
        "nuis_bootstrap_candidate_token_finish_v1",
        2,
    ),
    (
        "compiler_candidate_token_page_identity",
        "nuis_bootstrap_candidate_token_page_identity_v1",
        20,
    ),
    (
        "compiler_candidate_ast_page_identity",
        "nuis_bootstrap_candidate_ast_page_identity_v1",
        20,
    ),
    (
        "compiler_candidate_nir_page_identity",
        "nuis_bootstrap_candidate_nir_page_identity_v1",
        20,
    ),
];

pub const CODE_UNSUPPORTED_DOMAIN: &str = "NBS001";
pub const CODE_UNAPPROVED_IMPORT: &str = "NBS002";
pub const CODE_FFI: &str = "NBS003";
pub const CODE_ATTRIBUTE: &str = "NBS004";
pub const CODE_TRAIT: &str = "NBS005";
pub const CODE_GENERIC_BOUND: &str = "NBS006";
pub const CODE_TYPE: &str = "NBS007";
pub const CODE_ADDRESS_TYPE: &str = "NBS008";
pub const CODE_ASYNC: &str = "NBS009";
pub const CODE_HARNESS: &str = "NBS010";
pub const CODE_HOST_EFFECT: &str = "NBS011";
pub const CODE_FLOAT: &str = "NBS012";
pub const CODE_LAMBDA: &str = "NBS013";
pub const CODE_AWAIT: &str = "NBS014";
pub const CODE_INSTANTIATE: &str = "NBS015";
pub const CODE_DEREF: &str = "NBS016";
pub const CODE_INTRINSIC: &str = "NBS017";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BootstrapSubsetContext {
    pub local_cpu_units: BTreeSet<String>,
    pub local_type_names: BTreeSet<String>,
}

impl BootstrapSubsetContext {
    pub fn from_modules(modules: &[AstModule]) -> Self {
        let mut context = Self::default();
        for module in modules {
            if module.domain == "cpu" {
                context.local_cpu_units.insert(module.unit.clone());
            }
            context.local_type_names.extend(
                module
                    .type_aliases
                    .iter()
                    .map(|item| item.name.clone())
                    .chain(module.structs.iter().map(|item| item.name.clone()))
                    .chain(module.enums.iter().map(|item| item.name.clone())),
            );
        }
        context
    }

    fn permits_import(&self, domain: &str, unit: &str) -> bool {
        BOOTSTRAP_APPROVED_IMPORTS.contains(&(domain, unit))
            || (domain == "cpu" && self.local_cpu_units.contains(unit))
    }

    fn permits_type(&self, name: &str) -> bool {
        BOOTSTRAP_PRIMITIVE_TYPES.contains(&name)
            || BOOTSTRAP_APPROVED_EXTERNAL_TYPES.contains(&name)
            || self.local_type_names.contains(name)
            || name
                .split_once('.')
                .is_some_and(|(owner, variant)| !variant.is_empty() && self.permits_type(owner))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSubsetDiagnostic {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSubsetReport {
    pub protocol: &'static str,
    pub domain: String,
    pub unit: String,
    pub checked_nodes: usize,
    pub diagnostics: Vec<BootstrapSubsetDiagnostic>,
}

impl BootstrapSubsetReport {
    pub fn accepted(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn module_identity(&self) -> String {
        format!("{}/{}", self.domain, self.unit)
    }
}

pub fn validate_bootstrap_subset(
    module: &AstModule,
    context: &BootstrapSubsetContext,
) -> BootstrapSubsetReport {
    Validator::new(module, context).validate(module)
}

struct Validator<'a> {
    context: &'a BootstrapSubsetContext,
    module_identity: String,
    checked_nodes: usize,
    diagnostics: Vec<BootstrapSubsetDiagnostic>,
}

impl<'a> Validator<'a> {
    fn new(module: &AstModule, context: &'a BootstrapSubsetContext) -> Self {
        Self {
            context,
            module_identity: format!("{}/{}", module.domain, module.unit),
            checked_nodes: 0,
            diagnostics: Vec::new(),
        }
    }

    fn validate(mut self, module: &AstModule) -> BootstrapSubsetReport {
        self.checked_nodes += 1;
        let root = format!("module {}", self.module_identity);
        if module.domain != "cpu" {
            self.reject(
                CODE_UNSUPPORTED_DOMAIN,
                &root,
                format!(
                    "bootstrap modules must use `mod cpu`; found `mod {}`",
                    module.domain
                ),
            );
        }
        self.validate_attributes(&module.attributes, &root);
        for item in &module.uses {
            self.checked_nodes += 1;
            if !self.context.permits_import(&item.domain, &item.unit) {
                self.reject(
                    CODE_UNAPPROVED_IMPORT,
                    &format!("{root} use {}/{}", item.domain, item.unit),
                    "bootstrap sources may import only local CPU units or the frozen approved library units",
                );
            }
        }
        for item in &module.externs {
            self.checked_nodes += 1;
            self.reject(
                CODE_FFI,
                &format!("{root} extern {}", item.name),
                "FFI declarations are outside the bootstrap subset",
            );
        }
        for item in &module.extern_interfaces {
            self.checked_nodes += 1;
            self.reject(
                CODE_FFI,
                &format!("{root} extern interface {}", item.name),
                "FFI interfaces are outside the bootstrap subset",
            );
        }
        for item in &module.traits {
            self.checked_nodes += 1;
            self.reject(
                CODE_TRAIT,
                &format!("{root} trait {}", item.name),
                "traits and impl dispatch are deferred beyond bootstrap subset v1",
            );
        }
        for item in &module.impls {
            self.checked_nodes += 1;
            self.reject(
                CODE_TRAIT,
                &format!("{root} impl {}", item.trait_name),
                "traits and impl dispatch are deferred beyond bootstrap subset v1",
            );
        }
        for item in &module.consts {
            self.validate_const(item, &root);
        }
        for item in &module.type_aliases {
            self.validate_type_alias(item, &root);
        }
        for item in &module.structs {
            self.validate_struct(item, &root);
        }
        for item in &module.enums {
            self.validate_enum(item, &root);
        }
        for item in &module.functions {
            self.validate_function(item, &root);
        }
        BootstrapSubsetReport {
            protocol: BOOTSTRAP_SUBSET_PROTOCOL,
            domain: module.domain.clone(),
            unit: module.unit.clone(),
            checked_nodes: self.checked_nodes,
            diagnostics: self.diagnostics,
        }
    }

    fn validate_const(&mut self, item: &AstConstItem, root: &str) {
        self.checked_nodes += 1;
        let path = format!("{root} const {}", item.name);
        self.validate_attributes(&item.attributes, &path);
        if let Some(ty) = &item.ty {
            self.validate_type(ty, &BTreeSet::new(), &format!("{path} type"));
        }
        self.validate_expr(&item.value, &BTreeSet::new(), &format!("{path} value"));
    }

    fn validate_type_alias(&mut self, item: &AstTypeAlias, root: &str) {
        self.checked_nodes += 1;
        let path = format!("{root} type {}", item.name);
        self.validate_attributes(&item.attributes, &path);
        let generics = self.validate_generics(
            &item.generic_params,
            &item.where_bounds,
            &format!("{path} generics"),
        );
        self.validate_type(&item.target, &generics, &format!("{path} target"));
    }

    fn validate_struct(&mut self, item: &AstStructDef, root: &str) {
        self.checked_nodes += 1;
        let path = format!("{root} struct {}", item.name);
        self.validate_attributes(&item.attributes, &path);
        let generics = self.validate_generics(
            &item.generic_params,
            &item.where_bounds,
            &format!("{path} generics"),
        );
        for field in &item.fields {
            self.checked_nodes += 1;
            let field_path = format!("{path}.{}", field.name);
            self.validate_attributes(&field.attributes, &field_path);
            self.validate_type(&field.ty, &generics, &field_path);
        }
    }

    fn validate_enum(&mut self, item: &AstEnumDef, root: &str) {
        self.checked_nodes += 1;
        let path = format!("{root} enum {}", item.name);
        self.validate_attributes(&item.attributes, &path);
        let generics = self.validate_generics(
            &item.generic_params,
            &item.where_bounds,
            &format!("{path} generics"),
        );
        for variant in &item.variants {
            self.checked_nodes += 1;
            let variant_path = format!("{path}.{}", variant.name);
            self.validate_attributes(&variant.attributes, &variant_path);
            match &variant.kind {
                AstEnumVariantKind::Unit => {}
                AstEnumVariantKind::Tuple(types) => {
                    for (index, ty) in types.iter().enumerate() {
                        self.validate_type(
                            ty,
                            &generics,
                            &format!("{variant_path} payload[{index}]"),
                        );
                    }
                }
                AstEnumVariantKind::Struct(fields) => {
                    for field in fields {
                        let field_path = format!("{variant_path}.{}", field.name);
                        self.validate_attributes(&field.attributes, &field_path);
                        self.validate_type(&field.ty, &generics, &field_path);
                    }
                }
            }
        }
    }

    fn validate_function(&mut self, item: &AstFunction, root: &str) {
        self.checked_nodes += 1;
        let path = format!("{root} fn {}", item.name);
        self.validate_function_attributes(item, &path);
        if item.is_async {
            self.reject(
                CODE_ASYNC,
                &path,
                "async functions are outside bootstrap subset v1",
            );
        }
        if item.test_name.is_some() || item.benchmark_name.is_some() {
            self.reject(
                CODE_HARNESS,
                &path,
                "test and benchmark harness metadata are not compiler-source dependencies",
            );
        }
        let generics = self.validate_generics(
            &item.generic_params,
            &item.where_bounds,
            &format!("{path} generics"),
        );
        for param in &item.params {
            self.checked_nodes += 1;
            self.validate_type(
                &param.ty,
                &generics,
                &format!("{path} parameter {}", param.name),
            );
        }
        if let Some(ty) = &item.return_type {
            self.validate_type(ty, &generics, &format!("{path} return"));
        }
        self.validate_body(&item.body, &generics, &path);
    }

    fn validate_function_attributes(&mut self, item: &AstFunction, path: &str) {
        for attribute in &item.attributes {
            self.checked_nodes += 1;
            if attribute.name == "doc" || is_approved_scalar_export(item, attribute) {
                continue;
            }
            self.reject(
                CODE_ATTRIBUTE,
                &format!("{path} @{}", attribute.name),
                "bootstrap functions permit only documentation or the exact scalar candidate producer ABI exports",
            );
        }
    }

    fn validate_generics(
        &mut self,
        params: &[AstGenericParam],
        where_bounds: &[AstWherePredicate],
        path: &str,
    ) -> BTreeSet<String> {
        let names = params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        for param in params {
            self.checked_nodes += 1;
            if !param.bounds.is_empty() {
                self.reject(
                    CODE_GENERIC_BOUND,
                    &format!("{path} parameter {}", param.name),
                    "bootstrap subset v1 permits generic parameters but not trait bounds",
                );
            }
        }
        for predicate in where_bounds {
            self.checked_nodes += 1;
            self.reject(
                CODE_GENERIC_BOUND,
                &format!("{path} where {}", predicate.param_name),
                "where bounds are deferred beyond bootstrap subset v1",
            );
        }
        names
    }

    fn validate_attributes(&mut self, attributes: &[AstAttribute], path: &str) {
        for attribute in attributes {
            self.checked_nodes += 1;
            if attribute.name != "doc" {
                self.reject(
                    CODE_ATTRIBUTE,
                    &format!("{path} @{}", attribute.name),
                    "only documentation attributes are permitted in bootstrap sources",
                );
            }
        }
    }

    fn validate_type(&mut self, ty: &AstTypeRef, generics: &BTreeSet<String>, path: &str) {
        self.checked_nodes += 1;
        if ty.is_ref || ty.is_optional {
            self.reject(
                CODE_ADDRESS_TYPE,
                path,
                format!(
                    "bootstrap subset v1 requires owned, non-optional types; found `{}`",
                    render_type(ty)
                ),
            );
        }
        if matches!(ty.name.as_str(), "f32" | "f64") {
            self.reject(
                CODE_FLOAT,
                path,
                format!("floating type `{}` is outside bootstrap subset v1", ty.name),
            );
        } else if !generics.contains(&ty.name) && !self.context.permits_type(&ty.name) {
            self.reject(
                CODE_TYPE,
                path,
                format!(
                    "type `{}` is not in the bootstrap v1 type allowlist",
                    ty.name
                ),
            );
        }
        for (index, arg) in ty.generic_args.iter().enumerate() {
            self.validate_type(arg, generics, &format!("{path} argument[{index}]"));
        }
    }

    fn reject(&mut self, code: &'static str, path: &str, message: impl Into<String>) {
        self.diagnostics.push(BootstrapSubsetDiagnostic {
            code,
            path: path.to_owned(),
            message: message.into(),
        });
    }
}

fn is_approved_scalar_export(item: &AstFunction, attribute: &AstAttribute) -> bool {
    if attribute.name != "export"
        || attribute.args.len() != 1
        || attribute.args[0].name.as_deref() != Some("name")
        || item.is_async
        || !item.generic_params.is_empty()
        || !item.where_bounds.is_empty()
        || !item.params.iter().all(|param| is_plain_i64(&param.ty))
        || !item.return_type.as_ref().is_some_and(is_plain_i64)
    {
        return false;
    }
    let AstAttributeValue::String(symbol) = &attribute.args[0].value else {
        return false;
    };
    BOOTSTRAP_SCALAR_EXPORTS
        .iter()
        .any(|(function, expected_symbol, parameter_count)| {
            item.name == *function
                && symbol == *expected_symbol
                && item.params.len() == *parameter_count
        })
}

fn is_plain_i64(ty: &AstTypeRef) -> bool {
    ty.name == "i64" && ty.generic_args.is_empty() && !ty.is_ref && !ty.is_optional
}

fn render_type(ty: &AstTypeRef) -> String {
    let mut rendered = String::new();
    if ty.is_ref {
        rendered.push_str("ref ");
    }
    rendered.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        rendered.push('<');
        rendered.push_str(
            &ty.generic_args
                .iter()
                .map(render_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
        rendered.push('>');
    }
    if ty.is_optional {
        rendered.push('?');
    }
    rendered
}
