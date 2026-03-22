#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticContext {
    pub source_language: &'static str,
    pub profile: &'static str,
}
