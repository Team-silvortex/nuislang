use super::lower_project_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;

#[path = "tests_generic_constraints/alias_use_site.rs"]
mod alias_use_site;
#[path = "tests_generic_constraints/declarations.rs"]
mod declarations;
#[path = "tests_generic_constraints/expected_type_a.rs"]
mod expected_type_a;
#[path = "tests_generic_constraints/expected_type_b.rs"]
mod expected_type_b;
#[path = "tests_generic_constraints/explicit_contexts.rs"]
mod explicit_contexts;
#[path = "tests_generic_constraints/function_arg_failures.rs"]
mod function_arg_failures;
#[path = "tests_generic_constraints/helper_trait_variants.rs"]
mod helper_trait_variants;
#[path = "tests_generic_constraints/impl_overlap.rs"]
mod impl_overlap;
#[path = "tests_generic_constraints/trait_alias_failures.rs"]
mod trait_alias_failures;
