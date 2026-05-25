use yir_core::{
    ExecutionState, InstructionSemantics, NetworkFlowState, NetworkResultHandle, Node,
    RegisteredMod, Resource, Value,
};

pub struct NetworkMod;

impl RegisteredMod for NetworkMod {
    fn module_name(&self) -> &'static str {
        "network"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_network_resource(node, resource)?;

        match node.op.instruction.as_str() {
            "observe" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `network.observe <name> <resource> <input> <state>`",
                        node.name
                    ));
                }
                parse_network_flow_state(&node.op.args[1]).map_err(|error| {
                    format!(
                        "node `{}` has invalid network observe state: {error}",
                        node.name
                    )
                })?;
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "connect" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `network.connect <name> <resource> <local_port> <remote_port> <connect_timeout>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "accept" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `network.accept <name> <resource> <local_port> <read_timeout> <write_timeout>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "close" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `network.close <name> <resource> <handle>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "is_config_ready" | "is_connect_ready" | "is_accept_ready" | "is_closed" | "value" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `network.{} <name> <resource> <result>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            other => Err(format!(
                "unsupported network instruction `{other}` for node `{}`",
                node.name
            )),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        require_network_resource(node, resource)?;

        match node.op.instruction.as_str() {
            "observe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let flow = parse_network_flow_state(&node.op.args[1])?;
                Ok(Value::NetworkResult(NetworkResultHandle {
                    state: flow,
                    value: Box::new(value),
                }))
            }
            "connect" => {
                let value = state.expect_value(&node.op.args[1])?.clone();
                Ok(Value::NetworkResult(NetworkResultHandle {
                    state: NetworkFlowState::ConnectReady,
                    value: Box::new(value),
                }))
            }
            "accept" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                Ok(Value::NetworkResult(NetworkResultHandle {
                    state: NetworkFlowState::AcceptReady,
                    value: Box::new(value),
                }))
            }
            "close" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                Ok(Value::NetworkResult(NetworkResultHandle {
                    state: NetworkFlowState::Closed,
                    value: Box::new(value),
                }))
            }
            "is_config_ready" => {
                let result = state.expect_network_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(
                    result.state,
                    NetworkFlowState::ConfigReady
                )))
            }
            "is_connect_ready" => {
                let result = state.expect_network_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(
                    result.state,
                    NetworkFlowState::ConnectReady
                )))
            }
            "is_accept_ready" => {
                let result = state.expect_network_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(
                    result.state,
                    NetworkFlowState::AcceptReady
                )))
            }
            "is_closed" => {
                let result = state.expect_network_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(
                    result.state,
                    NetworkFlowState::Closed
                )))
            }
            "value" => {
                let result = state.expect_network_result(&node.op.args[0])?;
                Ok((*result.value).clone())
            }
            other => Err(format!(
                "unsupported network instruction `{other}` for node `{}`",
                node.name
            )),
        }
    }
}

fn require_network_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.raw.as_str() != "network.io" {
        return Err(format!(
            "node `{}` expects network.io resource, got `{}`",
            node.name, resource.kind.raw
        ));
    }
    Ok(())
}

fn parse_network_flow_state(raw: &str) -> Result<NetworkFlowState, String> {
    match raw {
        "config_ready" => Ok(NetworkFlowState::ConfigReady),
        "connect_ready" => Ok(NetworkFlowState::ConnectReady),
        "accept_ready" => Ok(NetworkFlowState::AcceptReady),
        "closed" => Ok(NetworkFlowState::Closed),
        other => Err(format!(
            "unknown network flow state `{other}`; expected config_ready, connect_ready, accept_ready, or closed"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkMod;
    use yir_core::{
        ExecutionState, NetworkFlowState, Node, Operation, RegisteredMod, Resource, ResourceKind,
        Value, YirResultState,
    };

    fn network_resource() -> Resource {
        Resource {
            name: "network0".to_owned(),
            kind: ResourceKind::parse("network.io"),
        }
    }

    #[test]
    fn executes_control_network_results_with_distinct_states() {
        let resource = network_resource();
        let mut state = ExecutionState::default();
        state.bind_value("local_port", Value::Int(7001));
        state.bind_value("remote_port", Value::Int(7443));
        state.bind_value("connect_timeout", Value::Int(1500));
        state.bind_value("read_timeout", Value::Int(800));
        state.bind_value("write_timeout", Value::Int(900));
        state.bind_value("handle", Value::Int(77));
        let network = NetworkMod;

        let connect = Node {
            name: "connect_result".to_owned(),
            resource: "network0".to_owned(),
            op: Operation::parse(
                "network.connect",
                vec![
                    "local_port".to_owned(),
                    "remote_port".to_owned(),
                    "connect_timeout".to_owned(),
                ],
            )
            .unwrap(),
        };
        let accept = Node {
            name: "accept_result".to_owned(),
            resource: "network0".to_owned(),
            op: Operation::parse(
                "network.accept",
                vec![
                    "local_port".to_owned(),
                    "read_timeout".to_owned(),
                    "write_timeout".to_owned(),
                ],
            )
            .unwrap(),
        };
        let close = Node {
            name: "close_result".to_owned(),
            resource: "network0".to_owned(),
            op: Operation::parse("network.close", vec!["handle".to_owned()]).unwrap(),
        };

        let connect_value = network.execute(&connect, &resource, &mut state).unwrap();
        let accept_value = network.execute(&accept, &resource, &mut state).unwrap();
        let close_value = network.execute(&close, &resource, &mut state).unwrap();

        assert_eq!(
            connect_value.result_state(),
            Some(YirResultState::Network(NetworkFlowState::ConnectReady))
        );
        assert_eq!(
            accept_value.result_state(),
            Some(YirResultState::Network(NetworkFlowState::AcceptReady))
        );
        assert_eq!(
            close_value.result_state(),
            Some(YirResultState::Network(NetworkFlowState::Closed))
        );
    }
}
