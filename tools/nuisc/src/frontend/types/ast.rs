use nuis_semantics::model::{AstTypeRef, NirResultFamily, NirTypeRef};

use super::super::lower_type_ref;
pub(crate) fn ast_type_from_nir(ty: &NirTypeRef) -> AstTypeRef {
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty.generic_args.iter().map(ast_type_from_nir).collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

pub(crate) fn ast_named_type(name: &str) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ast_generic_named_type(name: &str, generic_args: Vec<AstTypeRef>) -> AstTypeRef {
    AstTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ast_make_result_type(family: NirResultFamily, payload: AstTypeRef) -> AstTypeRef {
    let name = match family {
        NirResultFamily::Task => "TaskResult",
        NirResultFamily::Data => "DataResult",
        NirResultFamily::Shader => "ShaderResult",
        NirResultFamily::Kernel => "KernelResult",
        NirResultFamily::Network => "NetworkResult",
    };
    ast_generic_named_type(name, vec![payload])
}

fn parent_enum_ast_type(ty: &AstTypeRef) -> Option<AstTypeRef> {
    let (parent, _variant) = ty.name.rsplit_once('.')?;
    Some(AstTypeRef {
        name: parent.to_owned(),
        generic_args: ty.generic_args.clone(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

fn impl_lookup_types(ty: &AstTypeRef) -> Vec<String> {
    let mut rendered = vec![lower_type_ref(ty).render()];
    if let Some(parent) = parent_enum_ast_type(ty) {
        rendered.push(lower_type_ref(&parent).render());
    }
    rendered
}

#[path = "ast_calls.rs"]
mod ast_calls;
#[path = "ast_calls_views.rs"]
mod ast_calls_views;
#[path = "ast_infer.rs"]
mod ast_infer;
#[path = "ast_patterns.rs"]
mod ast_patterns;

pub(crate) use ast_infer::infer_ast_expr_type;
pub(crate) use ast_patterns::infer_ast_expr_type_for_pattern;
