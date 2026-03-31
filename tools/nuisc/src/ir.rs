use std::collections::BTreeMap;

use nuis_semantics::model::{NirModule, NirStmt, NirValue};
use yir_core::{Edge, EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuiscEngine {
    pub version: &'static str,
    pub profile: &'static str,
}

pub fn render_nir(module: &NirModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("nir module {}::{}\n", module.domain, module.name));
    for function in &module.functions {
        out.push_str(&format!("  fn {}\n", function.name));
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { name, value } => {
                    out.push_str(&format!("    let {} = {}\n", name, render_value(value)));
                }
                NirStmt::Print(value) => {
                    out.push_str(&format!("    print {}\n", render_value(value)));
                }
            }
        }
    }
    out
}

pub fn lower_nir_to_yir(module: &NirModule) -> Result<YirModule, String> {
    if module.domain != "cpu" {
        return Err(format!(
            "minimal nuisc lowering currently only supports `mod cpu`, found `{}`",
            module.domain
        ));
    }

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .ok_or_else(|| "minimal nuisc lowering expects `fn main()`".to_owned())?;

    let mut yir = YirModule::new("0.1");
    yir.resources.push(Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.arm64"),
    });

    let mut bindings = BTreeMap::<String, String>::new();

    for (index, stmt) in main.body.iter().enumerate() {
        match stmt {
            NirStmt::Let { name, value } => {
                let lowered = lower_value(value, index, &mut yir, &bindings)?;
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Print(value) => {
                let lowered = lower_value(value, index, &mut yir, &bindings)?;
                let print_name = format!("print_{index}");
                yir.nodes.push(Node {
                    name: print_name.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "print".to_owned(),
                        args: vec![lowered.clone()],
                    },
                });
                yir.edges.push(Edge {
                    kind: EdgeKind::Dep,
                    from: lowered.clone(),
                    to: print_name.clone(),
                });
                yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: lowered,
                    to: print_name,
                });
            }
        }
    }

    Ok(yir)
}

pub fn render_yir(module: &YirModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("yir {}\n\n", module.version));
    for resource in &module.resources {
        out.push_str(&format!("resource {} {}\n", resource.name, resource.kind.raw));
    }
    if !module.resources.is_empty() {
        out.push('\n');
    }
    for node in &module.nodes {
        out.push_str(&format!(
            "{}.{} {} {}",
            node.op.module, node.op.instruction, node.name, node.resource
        ));
        for arg in &node.op.args {
            if arg.chars().any(char::is_whitespace) {
                out.push_str(&format!(" \"{}\"", escape_debug(arg)));
            } else {
                out.push_str(&format!(" {}", arg));
            }
        }
        out.push('\n');
    }
    if !module.nodes.is_empty() {
        out.push('\n');
    }
    for edge in &module.edges {
        out.push_str(&format!(
            "edge {} {} {}\n",
            edge.kind.as_str(),
            edge.from,
            edge.to
        ));
    }
    out
}

fn escape_debug(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn render_value(value: &NirValue) -> String {
    match value {
        NirValue::Text(text) => format!("\"{}\"", escape_debug(text)),
        NirValue::Int(value) => value.to_string(),
        NirValue::Var(name) => name.clone(),
    }
}

fn lower_value(
    value: &NirValue,
    index: usize,
    yir: &mut YirModule,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    match value {
        NirValue::Text(text) => {
            let name = format!("text_{index}");
            yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "text".to_owned(),
                    args: vec![text.clone()],
                },
            });
            Ok(name)
        }
        NirValue::Int(value) => {
            let name = format!("int_{index}");
            yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "const".to_owned(),
                    args: vec![value.to_string()],
                },
            });
            Ok(name)
        }
        NirValue::Var(name) => bindings
            .get(name)
            .cloned()
            .ok_or_else(|| format!("minimal nuisc lowering found unbound variable `{name}`")),
    }
}
