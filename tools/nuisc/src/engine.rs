#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuiscEngine {
    pub version: &'static str,
    pub profile: &'static str,
}

pub fn default_engine() -> NuiscEngine {
    NuiscEngine {
        version: "0.44.b-draft",
        profile: "aot",
    }
}
