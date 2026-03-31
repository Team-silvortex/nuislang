use nuis_semantics::model::{NirModule, NirStmt, NirValue};
use yir_core::YirModule;

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

fn render_value(value: &NirValue) -> String {
    match value {
        NirValue::Text(text) => format!("\"{}\"", escape_debug(text)),
        NirValue::Int(value) => value.to_string(),
        NirValue::Var(name) => name.clone(),
    }
}

fn escape_debug(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
