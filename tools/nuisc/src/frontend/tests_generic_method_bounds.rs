use super::{lower_project_ast_to_nir, parse_nuis_ast, parse_nuis_module};

#[path = "tests_generic_method_bounds/ambiguous_alias_diagnostics.rs"]
mod ambiguous_alias_diagnostics;
#[path = "tests_generic_method_bounds/binary_operator_failures.rs"]
mod binary_operator_failures;
#[path = "tests_generic_method_bounds/explicit_trait_calls.rs"]
mod explicit_trait_calls;
#[path = "tests_generic_method_bounds/method_calls.rs"]
mod method_calls;
#[path = "tests_generic_method_bounds/method_suggestions.rs"]
mod method_suggestions;
#[path = "tests_generic_method_bounds/operator_acceptance.rs"]
mod operator_acceptance;
#[path = "tests_generic_method_bounds/qualified_operator_calls.rs"]
mod qualified_operator_calls;
#[path = "tests_generic_method_bounds/unary_order_failures.rs"]
mod unary_order_failures;
