#[path = "shader_source_normalize.rs"]
mod shader_source_normalize;
#[path = "shader_source_syntax.rs"]
mod shader_source_syntax;
#[cfg(test)]
#[path = "shader_source_tests.rs"]
mod shader_source_tests;

pub(crate) use shader_source_normalize::normalize_inline_wgsl_source;

use shader_source_syntax::{
    is_attribute_position, parse_binding_metadata, parse_parenthesized_args, parse_stage_metadata,
    starts_with_bare_attribute_keyword, starts_with_keyword,
};
