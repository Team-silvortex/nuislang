use super::*;
use yir_core::{
    ExecutionState, FunctionParameterContract, InstructionSemantics, ModRegistry, RegisteredMod,
    Value, YirFunction, YirFunctionParameter, YirFunctionRole, YirValueOwnership,
};

fn module(domain: &str, instruction: &str, args: &[&str]) -> YirModule {
    let mut module = YirModule::new("0.1");
    module.resources.push(Resource {
        name: "cpu0".into(),
        kind: ResourceKind::parse("cpu.arm64"),
    });
    module.nodes.push(node(
        "input",
        "cpu0",
        &format!("{domain}.{instruction}"),
        args,
    ));
    module.functions.push(YirFunction {
        name: "helper".into(),
        domain: domain.into(),
        role: YirFunctionRole::Helper,
        parameters: vec![YirFunctionParameter {
            name: "state".into(),
            ty: "State".into(),
            ownership: YirValueOwnership::Value,
            node: "input".into(),
        }],
        result: None,
        body_nodes: vec!["input".into()],
    });
    module
}

#[test]
fn registered_value_parameter_checks_index_type_ownership_domain_and_binding() {
    let valid = module("cpu", "param_value_struct", &["0", "State{x:i64}"]);
    verify_module(&valid).unwrap();
    for mutate in [
        (|m: &mut YirModule| m.nodes[0].op.args[0] = "1".into()) as fn(&mut YirModule),
        |m| m.nodes[0].op.args[1] = "Other{x:i64}".into(),
        |m| m.functions[0].parameters[0].ty = "i64".into(),
        |m| m.functions[0].parameters[0].ownership = YirValueOwnership::Owned,
        |m| m.functions[0].domain = "data".into(),
        |m| m.functions[0].parameters.clear(),
        |m| m.functions.clear(),
    ] {
        let mut invalid = valid.clone();
        mutate(&mut invalid);
        let error = verify_module(&invalid).unwrap_err();
        assert!(
            error.contains("disagrees with its function declaration"),
            "{error}"
        );
    }
    let mut duplicate = valid;
    let mut parameter = duplicate.functions[0].parameters[0].clone();
    parameter.name = "other".into();
    duplicate.functions[0].parameters.push(parameter);
    assert!(verify_module(&duplicate)
        .unwrap_err()
        .contains("invalid parameter"));
}

struct CustomDomain;
impl RegisteredMod for CustomDomain {
    fn module_name(&self) -> &'static str {
        "custom"
    }
    fn describe(&self, _: &Node, _: &Resource) -> Result<InstructionSemantics, String> {
        Ok(InstructionSemantics::pure(vec![]))
    }
    fn function_parameter(
        &self,
        _: &Node,
        _: &Resource,
    ) -> Result<Option<FunctionParameterContract>, String> {
        Ok(Some(FunctionParameterContract {
            index: 0,
            ty: "State".into(),
            ownership: YirValueOwnership::Value,
        }))
    }
    fn execute(&self, _: &Node, _: &Resource, _: &mut ExecutionState) -> Result<Value, String> {
        unreachable!()
    }
}

#[test]
fn parameter_contract_is_domain_registered_not_a_cpu_opcode_switch() {
    let mut registry = ModRegistry::new();
    registry.register(CustomDomain);
    let mut value = module("custom", "capture", &[]);
    crate::verify_module_with_registry(&value, &registry).unwrap();
    value.functions[0].parameters[0].ty = "Wrong".into();
    assert!(crate::verify_module_with_registry(&value, &registry)
        .unwrap_err()
        .contains("disagrees with its function declaration"));
}
