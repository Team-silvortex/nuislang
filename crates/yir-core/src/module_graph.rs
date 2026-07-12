use std::collections::BTreeMap;

use crate::Operation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_lanes: BTreeMap<String, String>,
}

impl YirModule {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            resources: Vec::new(),
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
