mod callables;
mod expansion;
mod templates;

pub(crate) use expansion::expand_higher_order_functions;
pub(crate) use expansion::rewrite_higher_order_calls_in_function;
pub(crate) use callables::is_callable_type_with_aliases;
