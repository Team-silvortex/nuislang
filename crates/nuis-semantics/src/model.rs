#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirIntent {
    pub op: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirModule {
    pub domain: String,
    pub name: String,
    pub functions: Vec<NirFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirFunction {
    pub name: String,
    pub body: Vec<NirStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirStmt {
    Let {
        name: String,
        value: NirValue,
    },
    Print(NirValue),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NirValue {
    Text(String),
    Int(i64),
    Var(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirNode {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricPrimitive {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackage {
    pub package_id: String,
    pub domain_family: String,
    pub entry_crate: String,
    pub ops: Vec<String>,
}
