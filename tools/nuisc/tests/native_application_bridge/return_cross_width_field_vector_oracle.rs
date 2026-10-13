#[allow(dead_code)]
#[path = "return_float_owner_field_vector_oracle.rs"]
mod shared;
pub use shared::Case;
pub const STAGES: [i64; 15] = shared::STAGES;
pub fn bits(ty: &str, value: f64) -> i64 {
    shared::bits(ty, value)
}
pub const SOURCE: &str = include_str!("return_cross_width_field_vector.ns");
pub fn child_type(owner: &'static str) -> &'static str {
    match owner {
        "f32" => "f64",
        "f64" => "f32",
        _ => panic!("unsupported test owner width"),
    }
}
pub fn source(owner: &'static str) -> String {
    SOURCE
        .replace("f32", "__OWNER__")
        .replace("f64", "__CHILD__")
        .replace("__OWNER__", owner)
        .replace("__CHILD__", child_type(owner))
}
pub fn cases(owner: &'static str) -> Vec<Case> {
    shared::cases_with_child(owner, child_type(owner))
}
pub fn masked_cases(owner: &'static str) -> Vec<Case> {
    shared::masked_cases_with_child(owner, child_type(owner))
}
pub fn cli_cases(owner: &'static str) -> Vec<Case> {
    shared::cli_cases_with_child(owner, child_type(owner))
}
pub fn selected(owner: &'static str) -> Vec<Case> {
    shared::selected_with_child(owner, child_type(owner))
}
#[allow(dead_code)]
pub fn native_selected(owner: &'static str) -> Vec<Case> {
    shared::native_selected_with_child(owner, child_type(owner))
}
