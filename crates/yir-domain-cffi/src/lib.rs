use yir_core::{ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, Value};

pub struct CffiMod;

impl RegisteredMod for CffiMod {
    fn module_name(&self) -> &'static str {
        "cffi"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_cffi_resource(node, resource)?;
        match node.op.instruction.as_str() {
            "target_config" => {
                if node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cffi.target_config <name> <resource> <arch> <abi> <vector-bits> <isa-family> <isa-features>`",
                        node.name
                    ));
                }
                node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vector width `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "extern_call_i64" | "extern_call_i32" => {
                if node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects a registered C ABI symbol",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(
                    node.op.args.iter().skip(1).cloned().collect(),
                ))
            }
            other => Err(format!(
                "unsupported cffi instruction `{other}` for node `{}`",
                node.name
            )),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        _state: &mut ExecutionState,
    ) -> Result<Value, String> {
        require_cffi_resource(node, resource)?;
        match node.op.instruction.as_str() {
            "target_config" => Ok(Value::Tuple(vec![
                Value::Symbol(node.op.args[0].clone()),
                Value::Symbol(node.op.args[1].clone()),
                Value::Int(node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vector width `{}`",
                        node.name, node.op.args[2]
                    )
                })?),
                Value::Symbol(node.op.args[3].clone()),
                Value::Symbol(node.op.args[4].clone()),
            ])),
            "extern_call_i64" | "extern_call_i32" => Err(format!(
                "cffi node `{}` requires the registered native host bridge and cannot execute in the pure YIR interpreter",
                node.name
            )),
            other => Err(format!(
                "unsupported cffi instruction `{other}` for node `{}`",
                node.name
            )),
        }
    }
}

fn require_cffi_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("cffi") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses cffi mod on non-cffi resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{Operation, ResourceKind};

    #[test]
    fn describes_registered_extern_call_dependencies() {
        let node = Node {
            name: "call".to_owned(),
            resource: "host".to_owned(),
            op: Operation {
                module: "cffi".to_owned(),
                instruction: "extern_call_i64".to_owned(),
                args: vec!["puts".to_owned(), "message".to_owned()],
            },
        };
        let resource = Resource {
            name: "host".to_owned(),
            kind: ResourceKind::parse("cffi.host"),
        };
        assert_eq!(
            CffiMod.describe(&node, &resource).unwrap(),
            InstructionSemantics::effect(vec!["message".to_owned()])
        );
    }
}
