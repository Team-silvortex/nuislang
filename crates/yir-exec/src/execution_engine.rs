use std::collections::{BTreeMap, BTreeSet};

use yir_core::{
    parse_loop_owned_struct_carry, parse_owned_struct_layout, ExecutionState, ModRegistry, Node,
    OwnedStructFieldLayout, OwnedStructLayout, OwnedStructScalarLayout, Resource, StructValue,
    Value, YirFunction, YirFunctionRole, YirModule,
};
use yir_verify::verify_module_with_registry;

use super::{
    execute_lazy_select, first_delayed_input, is_delayable_variant_error, topological_order,
    ExecutionTrace, LazySelectOutcome,
};

const MAX_FUNCTION_CALL_DEPTH: usize = 128;
const MAX_SCOPED_LOOP_ITERATIONS: usize = 100_000;

pub(super) fn execute_module_with_registry(
    module: &YirModule,
    registry: &ModRegistry,
) -> Result<ExecutionTrace, String> {
    verify_module_with_registry(module, registry)?;
    let order = topological_order(module)?;
    let all_function_nodes = module
        .functions
        .iter()
        .flat_map(|function| function.body_nodes.iter().cloned())
        .collect::<BTreeSet<_>>();
    let entry_nodes = module
        .functions
        .iter()
        .filter(|function| function.role == YirFunctionRole::Entry)
        .flat_map(|function| function.body_nodes.iter().cloned())
        .collect::<BTreeSet<_>>();
    let function_orders = module
        .functions
        .iter()
        .map(|function| {
            let body = function.body_nodes.iter().collect::<BTreeSet<_>>();
            let body_order = order
                .iter()
                .filter(|name| body.contains(name))
                .cloned()
                .collect::<Vec<_>>();
            (function.name.clone(), body_order)
        })
        .collect();
    let mut engine = ExecutionEngine {
        module,
        registry,
        resources: module
            .resources
            .iter()
            .map(|resource| (resource.name.clone(), resource))
            .collect(),
        nodes_by_name: module
            .nodes
            .iter()
            .map(|node| (node.name.clone(), node))
            .collect(),
        function_orders,
        state: ExecutionState::default(),
        lane_steps: BTreeMap::new(),
        call_stack: Vec::new(),
    };
    let mut delayed = BTreeMap::new();
    for node_name in order {
        if all_function_nodes.contains(&node_name) && !entry_nodes.contains(&node_name) {
            continue;
        }
        engine.execute_named_node(&node_name, &mut delayed)?;
    }
    reject_remaining_delayed(&delayed)?;
    Ok(engine.into_trace())
}

struct ExecutionEngine<'a> {
    module: &'a YirModule,
    registry: &'a ModRegistry,
    resources: BTreeMap<String, &'a Resource>,
    nodes_by_name: BTreeMap<String, &'a Node>,
    function_orders: BTreeMap<String, Vec<String>>,
    state: ExecutionState,
    lane_steps: BTreeMap<String, Vec<String>>,
    call_stack: Vec<String>,
}

impl ExecutionEngine<'_> {
    fn execute_named_node(
        &mut self,
        node_name: &str,
        delayed: &mut BTreeMap<String, String>,
    ) -> Result<(), String> {
        let node = self
            .nodes_by_name
            .get(node_name)
            .copied()
            .cloned()
            .ok_or_else(|| format!("execution order references unknown node `{node_name}`"))?;
        let resource = self
            .resources
            .get(&node.resource)
            .copied()
            .cloned()
            .ok_or_else(|| {
                format!(
                    "node `{}` references unknown resource `{}`",
                    node.name, node.resource
                )
            })?;
        let lane_name = self
            .module
            .node_lanes
            .get(&node.name)
            .map(|lane| format!("{}@{}", node.resource, lane))
            .unwrap_or_else(|| resource.kind.family().to_owned());
        self.state.current_lane = Some(lane_name.clone());
        self.lane_steps.entry(lane_name).or_default().push(format!(
            "{} @{} -> {}",
            node.op.full_name(),
            node.resource,
            node.name
        ));

        if node.op.module == "cpu" && node.op.instruction == "select" {
            match execute_lazy_select(&node, &mut self.state, delayed, &self.nodes_by_name)? {
                LazySelectOutcome::Handled(value) => {
                    self.state.values.insert(node.name, value);
                    return Ok(());
                }
                LazySelectOutcome::UseRegisteredExecutor => {}
            }
        }
        if let Some((input, reason)) = first_delayed_input(&node, delayed) {
            delayed.insert(
                node.name.clone(),
                format!("depends on delayed `{input}`: {reason}"),
            );
            return Ok(());
        }
        if let Some(value) = self.try_execute_function_call(&node, &resource)? {
            self.state.values.insert(node.name, value);
            return Ok(());
        }
        if let Some(value) = self.try_execute_scoped_owned_struct_loop(&node, &resource)? {
            self.state.values.insert(node.name, value);
            return Ok(());
        }
        if node.op.module == "cpu"
            && node.op.instruction == "loop_owned_struct_result"
            && self.state.values.contains_key(&node.name)
        {
            return Ok(());
        }

        let module_impl = self.registry.lookup(&node.op.module).ok_or_else(|| {
            format!(
                "node `{}` references unregistered mod `{}`",
                node.name, node.op.module
            )
        })?;
        let completion_registration = module_impl.provider_completion_registration(&node);
        if completion_registration.is_some() {
            self.state.begin_registered_provider_completion(&node)?;
        }
        let executed =
            match self
                .registry
                .execute_branch_effect_node(&node, &resource, &mut self.state)
            {
                Err(error) => {
                    self.state.abort_registered_provider_completion(&node);
                    return Err(error);
                }
                Ok(Some(value)) => Ok(value),
                Ok(None) => module_impl.execute(&node, &resource, &mut self.state),
            };
        match executed {
            Ok(value) => {
                if let Some(registration) = completion_registration {
                    self.state
                        .finish_registered_provider_completion(registration, &node)?;
                }
                self.state.values.insert(node.name, value);
                Ok(())
            }
            Err(error) if is_delayable_variant_error(&node, &error) => {
                self.state.abort_registered_provider_completion(&node);
                delayed.insert(node.name, error);
                Ok(())
            }
            Err(error) => {
                self.state.abort_registered_provider_completion(&node);
                Err(error)
            }
        }
    }

    fn try_execute_function_call(
        &mut self,
        node: &Node,
        resource: &Resource,
    ) -> Result<Option<Value>, String> {
        if node.op.module != "cpu"
            || !matches!(
                node.op.instruction.as_str(),
                "call_bool"
                    | "call_i32"
                    | "call_i64"
                    | "call_f32"
                    | "call_f64"
                    | "call_owned_bytes"
                    | "call_owned_struct"
            )
        {
            return Ok(None);
        }
        let Some(callee) = node.op.args.first() else {
            return Ok(None);
        };
        if !self.function_orders.contains_key(callee) {
            return Ok(None);
        }
        let argument_offset = usize::from(node.op.instruction == "call_owned_struct") + 1;
        let arguments = node.op.args[argument_offset..]
            .iter()
            .map(|name| self.state.expect_value(name).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        let rendered_arguments = arguments
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let value = self.execute_function(callee, arguments)?;
        validate_call_result(node, &value)?;
        if node.op.instruction != "call_owned_struct" {
            self.state.push_resource_event(
                resource,
                format!(
                    "effect {} @{} [{}] {}({})",
                    node.op.full_name(),
                    node.resource,
                    resource.kind.raw,
                    callee,
                    rendered_arguments
                ),
            );
        }
        Ok(Some(value))
    }

    fn execute_function(&mut self, name: &str, arguments: Vec<Value>) -> Result<Value, String> {
        let function = self
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
            .cloned()
            .ok_or_else(|| format!("unknown YIR function `{name}`"))?;
        if self.call_stack.len() >= MAX_FUNCTION_CALL_DEPTH {
            return Err(format!(
                "YIR function call depth exceeds {MAX_FUNCTION_CALL_DEPTH}: {} -> {name}",
                self.call_stack.join(" -> ")
            ));
        }
        if arguments.len() != function.parameters.len() {
            return Err(format!(
                "YIR function `{name}` expects {} arguments, got {}",
                function.parameters.len(),
                arguments.len()
            ));
        }
        let body_order = self
            .function_orders
            .get(name)
            .cloned()
            .ok_or_else(|| format!("YIR function `{name}` has no execution order"))?;
        let saved_values = function
            .body_nodes
            .iter()
            .map(|node| (node.clone(), self.state.values.remove(node)))
            .collect::<Vec<_>>();
        for (parameter, value) in function.parameters.iter().zip(arguments) {
            self.state.values.insert(parameter.node.clone(), value);
        }
        let previous_lane = self.state.current_lane.clone();
        self.call_stack.push(name.to_owned());
        let result = self.execute_function_body(&function, &body_order);
        self.call_stack.pop();
        self.state.current_lane = previous_lane;
        for node in &function.body_nodes {
            self.state.values.remove(node);
        }
        for (node, value) in saved_values {
            if let Some(value) = value {
                self.state.values.insert(node, value);
            }
        }
        result
    }

    fn execute_function_body(
        &mut self,
        function: &YirFunction,
        body_order: &[String],
    ) -> Result<Value, String> {
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| parameter.node.as_str())
            .collect::<BTreeSet<_>>();
        let mut delayed = BTreeMap::new();
        for node_name in body_order {
            if parameters.contains(node_name.as_str()) {
                continue;
            }
            self.execute_named_node(node_name, &mut delayed)?;
            let node = self.nodes_by_name[node_name.as_str()];
            if node.op.module == "cpu" && node.op.instruction == "guard_return" {
                let taken = match self.state.expect_value(&node.op.args[0])? {
                    Value::Bool(value) => *value,
                    Value::Int(value) => *value != 0,
                    _ => {
                        return Err(format!(
                            "guard `{node_name}` requires a bool or i64 condition"
                        ))
                    }
                };
                if taken {
                    return self.state.expect_value(&node.op.args[1]).cloned();
                }
            }
        }
        reject_remaining_delayed(&delayed)?;
        let result = function
            .result
            .as_ref()
            .ok_or_else(|| format!("YIR function `{}` has no registered result", function.name))?;
        self.state.values.get(&result.node).cloned().ok_or_else(|| {
            format!(
                "YIR function `{}` did not produce result node `{}`",
                function.name, result.node
            )
        })
    }

    fn try_execute_scoped_owned_struct_loop(
        &mut self,
        node: &Node,
        resource: &Resource,
    ) -> Result<Option<Value>, String> {
        if node.op.module != "cpu"
            || node.op.instruction != "loop_while_i64_effect"
            || node.op.args.get(5).map(String::as_str) != Some("cpu")
            || node.op.args.get(6).map(String::as_str) != Some("scoped_call_owned_struct_return")
        {
            return Ok(None);
        }
        let arity = parse_usize_arg(node, 7, "loop action arity")?;
        if node.op.args.len() != 8 + arity || arity < 4 {
            return Err(format!(
                "node `{}` has an invalid scoped owned-struct loop action arity",
                node.name
            ));
        }
        let callee = required_arg(node, 8, "loop callee")?.to_owned();
        let result_node = required_arg(node, 9, "loop result node")?.to_owned();
        let layout_source = required_arg(node, 10, "loop result layout")?;
        let layout = parse_owned_struct_layout(layout_source)?;
        let mut carries = vec![None; yir_core::owned_struct_scalar_leaf_count(&layout)];
        for operand in &node.op.args[11..] {
            if let Some((index, input)) = parse_loop_owned_struct_carry(operand)? {
                let slot = carries.get_mut(index).ok_or_else(|| {
                    format!("node `{}` has out-of-range carry index {index}", node.name)
                })?;
                if slot.is_some() {
                    return Err(format!(
                        "node `{}` repeats owned-struct carry index {index}",
                        node.name
                    ));
                }
                *slot = Some(self.state.expect_value(input)?.clone());
            }
        }
        let mut carries = carries
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                value.ok_or_else(|| {
                    format!(
                        "node `{}` is missing owned-struct carry index {index}",
                        node.name
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut current = self
            .state
            .expect_int(required_arg(node, 0, "loop initial")?)?;
        let limit = self
            .state
            .expect_int(required_arg(node, 1, "loop limit")?)?;
        let step = self.state.expect_int(required_arg(node, 2, "loop step")?)?;
        let compare = required_arg(node, 3, "loop compare kind")?;
        let step_kind = required_arg(node, 4, "loop step kind")?;
        let mut result = rebuild_owned_struct(&layout, &carries)?;
        let mut iterations = 0usize;
        while loop_condition(compare, current, limit, node)? {
            if iterations >= MAX_SCOPED_LOOP_ITERATIONS {
                return Err(format!(
                    "node `{}` exceeds {MAX_SCOPED_LOOP_ITERATIONS} reference loop iterations",
                    node.name
                ));
            }
            let arguments = node.op.args[11..]
                .iter()
                .map(|operand| {
                    if operand == "$current" {
                        return Ok(Value::Int(current));
                    }
                    if let Some((index, _)) = parse_loop_owned_struct_carry(operand)? {
                        return carries.get(index).cloned().ok_or_else(|| {
                            format!("node `{}` has missing carry index {index}", node.name)
                        });
                    }
                    self.state.expect_value(operand).cloned()
                })
                .collect::<Result<Vec<_>, String>>()?;
            result = self.execute_function(&callee, arguments)?;
            carries = flatten_owned_struct(&layout, &result)?;
            current = advance_loop(step_kind, current, step, node)?;
            iterations += 1;
        }
        self.state.values.insert(result_node, result);
        self.state.push_resource_event(
            resource,
            format!(
                "effect cpu.loop_while_i64_effect @{} [{}]: iterations={} final={} action cpu.scoped_call_owned_struct_return {}",
                node.resource, resource.kind.raw, iterations, current, callee
            ),
        );
        Ok(Some(Value::Int(current)))
    }

    fn into_trace(self) -> ExecutionTrace {
        let provider_completion_witnesses = self.state.provider_completion_witnesses().clone();
        let presented_frames = self.state.presented_frames().to_vec();
        ExecutionTrace {
            events: self.state.events,
            lane_events: self.state.lane_events,
            lane_steps: self.lane_steps,
            values: self.state.values,
            presented_frames,
            provider_completion_witnesses,
        }
    }
}

fn validate_call_result(node: &Node, value: &Value) -> Result<(), String> {
    let valid = matches!(
        (node.op.instruction.as_str(), value),
        ("call_bool", Value::Bool(_))
            | ("call_i32", Value::I32(_))
            | ("call_i64", Value::Int(_))
            | ("call_f32", Value::F32(_))
            | ("call_f64", Value::F64(_))
            | ("call_owned_bytes", Value::OwnedBytes(_))
            | ("call_owned_struct", Value::Struct(_))
    );
    if valid {
        Ok(())
    } else {
        Err(format!(
            "node `{}` received incompatible result {value} from `{}`",
            node.name, node.op.args[0]
        ))
    }
}

fn loop_condition(compare: &str, current: i64, limit: i64, node: &Node) -> Result<bool, String> {
    match compare {
        "eq" => Ok(current == limit),
        "ne" => Ok(current != limit),
        "lt" => Ok(current < limit),
        "le" => Ok(current <= limit),
        "gt" => Ok(current > limit),
        "ge" => Ok(current >= limit),
        other => Err(format!(
            "node `{}` has invalid loop compare kind `{other}`",
            node.name
        )),
    }
}

fn advance_loop(step_kind: &str, current: i64, step: i64, node: &Node) -> Result<i64, String> {
    match step_kind {
        "add" => current.checked_add(step),
        "sub" => current.checked_sub(step),
        other => {
            return Err(format!(
                "node `{}` has invalid loop step kind `{other}`",
                node.name
            ))
        }
    }
    .ok_or_else(|| format!("node `{}` overflows its i64 loop induction", node.name))
}

fn flatten_owned_struct(layout: &OwnedStructLayout, value: &Value) -> Result<Vec<Value>, String> {
    let mut leaves = Vec::new();
    flatten_owned_struct_into(layout, value, &mut leaves)?;
    Ok(leaves)
}

fn flatten_owned_struct_into(
    layout: &OwnedStructLayout,
    value: &Value,
    leaves: &mut Vec<Value>,
) -> Result<(), String> {
    let Value::Struct(value) = value else {
        return Err(format!(
            "owned-struct loop expected `{}`, got {value}",
            layout.type_name
        ));
    };
    if value.type_name != layout.type_name {
        return Err(format!(
            "owned-struct loop expected `{}`, got `{}`",
            layout.type_name, value.type_name
        ));
    }
    if layout.fields.is_empty() {
        leaves.push(Value::Struct(value.clone()));
        return Ok(());
    }
    for (field_name, field_layout) in &layout.fields {
        let field = value
            .fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                format!(
                    "owned-struct loop value `{}` is missing field `{field_name}`",
                    layout.type_name
                )
            })?;
        match field_layout {
            OwnedStructFieldLayout::Scalar(kind) => {
                validate_scalar_leaf(*kind, field)?;
                leaves.push(field.clone());
            }
            OwnedStructFieldLayout::Struct(nested) => {
                flatten_owned_struct_into(nested, field, leaves)?;
            }
        }
    }
    Ok(())
}

fn rebuild_owned_struct(layout: &OwnedStructLayout, leaves: &[Value]) -> Result<Value, String> {
    let mut index = 0usize;
    let value = rebuild_owned_struct_at(layout, leaves, &mut index)?;
    if index != leaves.len() {
        return Err(format!(
            "owned-struct loop layout `{}` consumed {index} of {} carry leaves",
            layout.type_name,
            leaves.len()
        ));
    }
    Ok(Value::Struct(value))
}

fn rebuild_owned_struct_at(
    layout: &OwnedStructLayout,
    leaves: &[Value],
    index: &mut usize,
) -> Result<StructValue, String> {
    if layout.fields.is_empty() {
        let value = leaves.get(*index).ok_or_else(|| {
            format!(
                "owned-struct loop `{}` is missing a carry",
                layout.type_name
            )
        })?;
        *index += 1;
        let Value::Struct(value) = value else {
            return Err(format!(
                "owned-struct loop expected empty `{}`, got {value}",
                layout.type_name
            ));
        };
        return Ok(value.clone());
    }
    let mut fields = Vec::with_capacity(layout.fields.len());
    for (field_name, field_layout) in &layout.fields {
        let value = match field_layout {
            OwnedStructFieldLayout::Scalar(kind) => {
                let value = leaves.get(*index).cloned().ok_or_else(|| {
                    format!(
                        "owned-struct loop `{}` is missing field `{field_name}`",
                        layout.type_name
                    )
                })?;
                validate_scalar_leaf(*kind, &value)?;
                *index += 1;
                value
            }
            OwnedStructFieldLayout::Struct(nested) => {
                Value::Struct(rebuild_owned_struct_at(nested, leaves, index)?)
            }
        };
        fields.push((field_name.clone(), value));
    }
    Ok(StructValue {
        type_name: layout.type_name.clone(),
        fields,
    })
}

fn validate_scalar_leaf(kind: OwnedStructScalarLayout, value: &Value) -> Result<(), String> {
    let valid = matches!(
        (kind, value),
        (OwnedStructScalarLayout::Bool, Value::Bool(_))
            | (OwnedStructScalarLayout::I32, Value::I32(_))
            | (OwnedStructScalarLayout::I64, Value::Int(_))
            | (OwnedStructScalarLayout::F32, Value::F32(_))
            | (OwnedStructScalarLayout::F64, Value::F64(_))
            | (OwnedStructScalarLayout::String, Value::Symbol(_))
            | (OwnedStructScalarLayout::Bytes, Value::OwnedBytes(_))
    );
    if valid {
        Ok(())
    } else {
        Err(format!(
            "owned-struct loop carry {value} does not match scalar layout {kind:?}"
        ))
    }
}

fn parse_usize_arg(node: &Node, index: usize, label: &str) -> Result<usize, String> {
    required_arg(node, index, label)?
        .parse::<usize>()
        .map_err(|_| format!("node `{}` has invalid {label}", node.name))
}

fn required_arg<'a>(node: &'a Node, index: usize, label: &str) -> Result<&'a str, String> {
    node.op
        .args
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("node `{}` is missing {label}", node.name))
}

fn reject_remaining_delayed(delayed: &BTreeMap<String, String>) -> Result<(), String> {
    if let Some((name, error)) = delayed.iter().next() {
        Err(format!(
            "node `{name}` was never selected by a lazy branch: {error}"
        ))
    } else {
        Ok(())
    }
}
