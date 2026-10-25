#[allow(dead_code)]
#[path = "alias_record_return_cases.rs"]
mod prior;

// The new pure join preserves the independent predecessor execution contract.
#[allow(unused_imports)]
pub use prior::{
    failures, healthy, native_failures, prefix, Case, ENTRY_LIMIT, ENTRY_MARKERS, LAST_STATE,
    LOOP_LIMIT, OBSERVERS,
};
pub const SOURCE: &str = include_str!("origin_record_join.ns");
