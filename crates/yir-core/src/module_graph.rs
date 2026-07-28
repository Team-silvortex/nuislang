use std::collections::BTreeMap;

use crate::Operation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub functions: Vec<YirFunction>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_lanes: BTreeMap<String, String>,
}

impl YirModule {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            resources: Vec::new(),
            functions: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            node_lanes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub name: String,
    pub kind: ResourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKind {
    pub raw: String,
}

impl ResourceKind {
    pub fn parse(raw: &str) -> Self {
        Self {
            raw: raw.to_owned(),
        }
    }

    pub fn family(&self) -> &str {
        self.raw.split('.').next().unwrap_or(self.raw.as_str())
    }

    pub fn is_family(&self, expected: &str) -> bool {
        self.family() == expected
    }
}

pub const YIR_FUNCTION_TABLE_CONTRACT: &str = "nuis-yir-function-table-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirFunctionRole {
    Entry,
    Helper,
    Provider,
}

impl YirFunctionRole {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "entry" => Ok(Self::Entry),
            "helper" => Ok(Self::Helper),
            "provider" => Ok(Self::Provider),
            other => Err(format!(
                "unknown YIR function role `{other}`; expected entry|helper|provider"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Helper => "helper",
            Self::Provider => "provider",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirValueOwnership {
    Value,
    Borrowed,
    Owned,
}

impl YirValueOwnership {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "value" => Ok(Self::Value),
            "borrowed" => Ok(Self::Borrowed),
            "owned" => Ok(Self::Owned),
            other => Err(format!(
                "unknown YIR value ownership `{other}`; expected value|borrowed|owned"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Borrowed => "borrowed",
            Self::Owned => "owned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirFunctionParameter {
    pub name: String,
    pub ty: String,
    pub ownership: YirValueOwnership,
    pub node: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirFunctionResult {
    pub ty: String,
    pub ownership: YirValueOwnership,
    pub node: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirFunction {
    pub name: String,
    pub domain: String,
    pub role: YirFunctionRole,
    pub parameters: Vec<YirFunctionParameter>,
    pub result: Option<YirFunctionResult>,
    pub body_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub resource: String,
    pub op: Operation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub kind: EdgeKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Dep,
    Effect,
    Lifetime,
    CrossDomainExchange,
}

impl EdgeKind {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "dep" => Ok(Self::Dep),
            "effect" => Ok(Self::Effect),
            "lifetime" => Ok(Self::Lifetime),
            "xfer" => Ok(Self::CrossDomainExchange),
            other => Err(format!(
                "unknown edge kind `{other}`; expected dep|effect|lifetime|xfer"
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dep => "dep",
            Self::Effect => "effect",
            Self::Lifetime => "lifetime",
            Self::CrossDomainExchange => "xfer",
        }
    }
}
