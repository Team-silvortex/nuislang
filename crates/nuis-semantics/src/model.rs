#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirIntent {
    pub op: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirNode {
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricPrimitive {
    pub name: &'static str,
}
