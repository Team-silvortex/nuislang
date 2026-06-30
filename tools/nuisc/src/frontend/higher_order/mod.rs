mod callables;
mod expansion;
mod expansion_callable_inference;
mod expansion_expected;
mod expansion_inference;
mod expansion_rewrite;
mod expansion_rewrite_expr;
mod templates;

pub(crate) use callables::is_callable_type_with_aliases;
pub(crate) use expansion::expand_higher_order_functions;
pub(crate) use expansion_rewrite::rewrite_higher_order_calls_in_function;
