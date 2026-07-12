use std::collections::BTreeMap;

use crate::{ExecutionState, Node, Resource, Value};

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
