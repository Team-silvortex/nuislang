#[allow(dead_code)]
#[path = "return_private_fields_oracle.rs"]
mod base;
pub use base::Case;
#[allow(dead_code)]
pub const STAGES: [i64; 11] = base::STAGES;
pub const SOURCE: &str = include_str!("return_retained_private_fields.ns");

fn retained(c: Case) -> Case {
    Case::retained(c.ty, c.seed, c.small_seed, c.float_seed, c.first, c.second)
}
#[allow(dead_code)]
pub fn cases(ty: &'static str) -> Vec<Case> {
    base::cases(ty).into_iter().map(retained).collect()
}
pub fn cli_cases(ty: &'static str) -> Vec<Case> {
    base::cli_cases(ty).into_iter().map(retained).collect()
}
pub fn selected(ty: &'static str) -> Vec<Case> {
    base::selected(ty).into_iter().map(retained).collect()
}
#[allow(dead_code)]
pub fn native_selected(ty: &'static str) -> Vec<Case> {
    base::native_selected(ty)
        .into_iter()
        .map(retained)
        .collect()
}
