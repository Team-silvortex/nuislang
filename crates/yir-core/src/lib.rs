use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl YirModule {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            resources: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
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
        Self { raw: raw.to_owned() }
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
pub struct Operation {
    pub module: String,
    pub instruction: String,
    pub args: Vec<String>,
}

impl Operation {
    pub fn parse(raw: &str, args: Vec<String>) -> Result<Self, String> {
        let (module, instruction) = raw
            .split_once('.')
            .ok_or_else(|| format!("operation `{raw}` must be written as <mod>.<instr>"))?;

        if module.is_empty() || instruction.is_empty() {
            return Err(format!("operation `{raw}` must be written as <mod>.<instr>"));
        }

        Ok(Self {
            module: module.to_owned(),
            instruction: instruction.to_owned(),
            args,
        })
    }

    pub fn full_name(&self) -> String {
        format!("{}.{}", self.module, self.instruction)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Tuple(Vec<Value>),
    Frame(FrameSurface),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSurface {
    pub width: usize,
    pub height: usize,
    pub rows: Vec<String>,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Tuple(values) => {
                write!(f, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Self::Frame(frame) => write!(f, "{frame}"),
            Self::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for FrameSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frame[{}x{}] ", self.width, self.height)?;
        for (index, row) in self.rows.iter().enumerate() {
            if index > 0 {
                write!(f, "|")?;
            }
            write!(f, "{row}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionSemantics {
    pub dependencies: Vec<String>,
    pub has_effect: bool,
}

impl InstructionSemantics {
    pub fn pure(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: false,
        }
    }

    pub fn effect(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: true,
        }
    }
}

pub trait RegisteredMod: Send + Sync {
    fn module_name(&self) -> &'static str;

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String>;

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String>;
}

#[derive(Default)]
pub struct ModRegistry {
    mods: BTreeMap<String, Box<dyn RegisteredMod>>,
}

impl ModRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M>(&mut self, module: M)
    where
        M: RegisteredMod + 'static,
    {
        self.mods
            .insert(module.module_name().to_owned(), Box::new(module));
    }

    pub fn lookup(&self, name: &str) -> Option<&dyn RegisteredMod> {
        self.mods.get(name).map(|module| module.as_ref())
    }
}

#[derive(Debug, Default)]
pub struct ExecutionState {
    pub events: Vec<String>,
    pub lane_events: BTreeMap<String, Vec<String>>,
    pub values: BTreeMap<String, Value>,
}

impl ExecutionState {
    pub fn expect_int(&self, name: &str) -> Result<i64, String> {
        match self.values.get(name) {
            Some(Value::Int(value)) => Ok(*value),
            Some(Value::Tuple(_)) => Err(format!("`{name}` is tuple, expected int")),
            Some(Value::Frame(_)) => Err(format!("`{name}` is frame, expected int")),
            Some(Value::Unit) => Err(format!("`{name}` is unit, expected int")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_value(&self, name: &str) -> Result<&Value, String> {
        self.values
            .get(name)
            .ok_or_else(|| format!("missing value for `{name}`"))
    }

    pub fn push_event(&mut self, event: impl Into<String>) {
        self.events.push(event.into());
    }

    pub fn push_resource_event(&mut self, resource: &Resource, event: impl Into<String>) {
        let event = event.into();
        self.events.push(event.clone());
        self.lane_events
            .entry(resource.kind.family().to_owned())
            .or_default()
            .push(event);
    }
}

pub struct FabricMod;

impl RegisteredMod for FabricMod {
    fn module_name(&self) -> &'static str {
        "fabric"
    }

    fn describe(&self, node: &Node, _resource: &Resource) -> Result<InstructionSemantics, String> {
        match node.op.instruction.as_str() {
            "move" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `fabric.move <name> <resource> <input> <to>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            other => Err(format!("unknown fabric instruction `{other}`")),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        match node.op.instruction.as_str() {
            "move" => {
                let input = &node.op.args[0];
                let target = &node.op.args[1];
                let value = state.expect_value(input)?.clone();
                state.push_resource_event(resource, format!(
                    "effect fabric.move @{} [{}] -> {}: {}",
                    node.resource, resource.kind.raw, target, value
                ));
                Ok(value)
            }
            other => Err(format!("unknown fabric instruction `{other}`")),
        }
    }
}
