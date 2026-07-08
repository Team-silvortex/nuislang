use std::collections::BTreeMap;

use super::verify_module;
use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

fn node(name: &str, resource: &str, op: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: resource.to_owned(),
        op: Operation::parse(op, args.iter().map(|item| (*item).to_owned()).collect()).unwrap(),
    }
}

fn dep(from: &str, to: &str) -> Edge {
    Edge {
        kind: EdgeKind::Dep,
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn effect(from: &str, to: &str) -> Edge {
    Edge {
        kind: EdgeKind::Effect,
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn lifetime(from: &str, to: &str) -> Edge {
    Edge {
        kind: EdgeKind::Lifetime,
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

fn xfer(from: &str, to: &str) -> Edge {
    Edge {
        kind: EdgeKind::CrossDomainExchange,
        from: from.to_owned(),
        to: to.to_owned(),
    }
}

#[path = "tests/cpu_heap.rs"]
mod cpu_heap;
#[path = "tests/data_fabric.rs"]
mod data_fabric;
#[path = "tests/project_contracts.rs"]
mod project_contracts;
#[path = "tests/result_state.rs"]
mod result_state;
#[path = "tests/scheduler_contracts.rs"]
mod scheduler_contracts;
