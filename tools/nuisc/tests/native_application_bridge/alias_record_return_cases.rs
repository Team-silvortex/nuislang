#[allow(dead_code)]
#[path = "local_record_return_cases.rs"]
mod prior;

// Pure copies add no work: reuse the independent predecessor oracle unchanged.
#[allow(unused_imports)]
pub use prior::{
    failures, healthy, native_failures, prefix, Case, ENTRY_LIMIT, ENTRY_MARKERS, LAST_STATE,
    LOOP_LIMIT, OBSERVERS,
};
pub const SOURCE: &str = include_str!("alias_record_return.ns");
