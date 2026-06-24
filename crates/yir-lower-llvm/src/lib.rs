use std::collections::BTreeMap;

use yir_core::{CpuLlvmLoweringClass, EdgeKind, Node, Resource, YirModule};
use yir_verify::verify_module;

#[derive(Clone)]
enum LlvmValueRef {
    Bool { i1: String, i64: String },
    I32(String),
    I64(String),
    F32(String),
    F64(String),
    Task(TaskLlvmValueRef),
    Thread(ThreadLlvmValueRef),
    TaskResult(TaskResultLlvmValueRef),
    Mutex(MutexLlvmValueRef),
    MutexGuard(MutexGuardLlvmValueRef),
    NetworkResult(NetworkResultLlvmValueRef),
    Struct(StructLlvmValueRef),
    Ptr(String),
    TextHandle { ptr: String, handle: String },
    Void,
}

#[derive(Clone)]
struct StructLlvmValueRef {
    type_name: String,
    fields: Vec<(String, LlvmValueRef)>,
}

#[derive(Clone)]
struct NetworkResultLlvmValueRef {
    state: String,
    value: Box<LlvmValueRef>,
}

#[derive(Clone)]
struct TaskLlvmValueRef {
    value: Box<LlvmValueRef>,
}

#[derive(Clone)]
struct ThreadLlvmValueRef {
    value: Box<LlvmValueRef>,
}

#[derive(Clone)]
struct TaskResultLlvmValueRef {
    state: String,
    value: Option<Box<LlvmValueRef>>,
}

#[derive(Clone)]
struct MutexLlvmValueRef {
    value: Box<LlvmValueRef>,
}

#[derive(Clone)]
struct MutexGuardLlvmValueRef {
    value: Box<LlvmValueRef>,
}

struct LlvmLoweringState {
    body: Vec<String>,
    globals: Vec<String>,
    registers: BTreeMap<String, LlvmValueRef>,
    buffer_lengths: BTreeMap<String, String>,
    next_reg: usize,
    next_global: usize,
    next_block: usize,
    last_cpu_value: Option<String>,
    ends_with_terminal_return: bool,
}

struct EmittedCpuFunction {
    globals: Vec<String>,
    body: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CpuCallScalarKind {
    Bool,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CpuLoopScalarKind {
    I64,
    F32,
    F64,
}

struct CpuHelperSignature {
    params: Vec<CpuCallScalarKind>,
    ret: CpuCallScalarKind,
}

#[derive(Clone)]
enum LoopControlExpr {
    Cond { kind: String, rhs_name: String },
    And(Box<LoopControlExpr>, Box<LoopControlExpr>),
    Or(Box<LoopControlExpr>, Box<LoopControlExpr>),
}

#[derive(Clone)]
enum ResolvedLoopControlExpr {
    Cond { kind: String, rhs: String },
    And(Box<ResolvedLoopControlExpr>, Box<ResolvedLoopControlExpr>),
    Or(Box<ResolvedLoopControlExpr>, Box<ResolvedLoopControlExpr>),
}

#[derive(Clone)]
enum LoopFlowExpr {
    Legacy {
        condition: LoopControlExpr,
        action: String,
    },
    Terminal {
        action: String,
        condition: LoopControlExpr,
    },
    Or(Box<LoopFlowExpr>, Box<LoopFlowExpr>),
}

#[derive(Clone)]
enum ResolvedLoopFlowExpr {
    Legacy {
        condition: ResolvedLoopControlExpr,
        action: String,
    },
    Terminal {
        action: String,
        condition: ResolvedLoopControlExpr,
    },
    Or(Box<ResolvedLoopFlowExpr>, Box<ResolvedLoopFlowExpr>),
}

fn parse_loop_control_expr_for_llvm(
    args: &[String],
    start: usize,
    node_name: &str,
    instruction_name: &str,
) -> Result<(LoopControlExpr, usize), String> {
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "cpu.{instruction_name} `{}` is missing control metadata during LLVM lowering",
            node_name
        ));
    };
    if token == "and" {
        let (lhs, after_lhs) =
            parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
        let (rhs, after_rhs) =
            parse_loop_control_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
        Ok((
            LoopControlExpr::And(Box::new(lhs), Box::new(rhs)),
            after_rhs,
        ))
    } else if token == "or" {
        let (lhs, after_lhs) =
            parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
        let (rhs, after_rhs) =
            parse_loop_control_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
        Ok((LoopControlExpr::Or(Box::new(lhs), Box::new(rhs)), after_rhs))
    } else {
        let Some(rhs_name) = args.get(start + 1) else {
            return Err(format!(
                "cpu.{instruction_name} `{}` is missing control rhs during LLVM lowering",
                node_name
            ));
        };
        Ok((
            LoopControlExpr::Cond {
                kind: token.to_owned(),
                rhs_name: rhs_name.clone(),
            },
            start + 2,
        ))
    }
}

fn parse_loop_flow_expr_for_llvm(
    args: &[String],
    start: usize,
    node_name: &str,
    instruction_name: &str,
) -> Result<(LoopFlowExpr, usize), String> {
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "cpu.{instruction_name} `{}` is missing control metadata during LLVM lowering",
            node_name
        ));
    };
    match token {
        "flow_or" => {
            let (lhs, after_lhs) =
                parse_loop_flow_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
            let (rhs, after_rhs) =
                parse_loop_flow_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
            Ok((LoopFlowExpr::Or(Box::new(lhs), Box::new(rhs)), after_rhs))
        }
        "flow_break" | "flow_continue" => {
            let (condition, after_condition) =
                parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
            Ok((
                LoopFlowExpr::Terminal {
                    action: token.trim_start_matches("flow_").to_owned(),
                    condition,
                },
                after_condition,
            ))
        }
        "flow_and" => Err(format!(
            "cpu.{instruction_name} `{}` does not support `flow_and` during LLVM lowering yet",
            node_name
        )),
        _ => {
            let (condition, action_index) =
                parse_loop_control_expr_for_llvm(args, start, node_name, instruction_name)?;
            let Some(action) = args.get(action_index) else {
                return Err(format!(
                    "cpu.{instruction_name} `{}` is missing control action during LLVM lowering",
                    node_name
                ));
            };
            Ok((
                LoopFlowExpr::Legacy {
                    condition,
                    action: action.clone(),
                },
                action_index + 1,
            ))
        }
    }
}

fn resolve_loop_control_expr_for_llvm(
    expr: &LoopControlExpr,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    instruction_name: &str,
) -> Option<ResolvedLoopControlExpr> {
    match expr {
        LoopControlExpr::Cond { kind, rhs_name } => {
            let Some(value) = registers.get(rhs_name).cloned() else {
                body.push(format!("  ; deferred lowering for cpu.{instruction_name} `{}` because one or more control rhs values are outside the current CPU LLVM slice", node_name));
                return None;
            };
            let Some(rhs) = coerce_to_i64(&value, body, next_reg) else {
                body.push(format!("  ; deferred lowering for cpu.{instruction_name} `{}` because one or more control rhs values are not coercible to i64", node_name));
                return None;
            };
            Some(ResolvedLoopControlExpr::Cond {
                kind: kind.clone(),
                rhs,
            })
        }
        LoopControlExpr::And(lhs, rhs) => Some(ResolvedLoopControlExpr::And(
            Box::new(resolve_loop_control_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_control_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
        LoopControlExpr::Or(lhs, rhs) => Some(ResolvedLoopControlExpr::Or(
            Box::new(resolve_loop_control_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_control_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
    }
}

fn resolve_loop_flow_expr_for_llvm(
    expr: &LoopFlowExpr,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    instruction_name: &str,
) -> Option<ResolvedLoopFlowExpr> {
    match expr {
        LoopFlowExpr::Legacy { condition, action } => Some(ResolvedLoopFlowExpr::Legacy {
            condition: resolve_loop_control_expr_for_llvm(
                condition,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?,
            action: action.clone(),
        }),
        LoopFlowExpr::Terminal { action, condition } => Some(ResolvedLoopFlowExpr::Terminal {
            action: action.clone(),
            condition: resolve_loop_control_expr_for_llvm(
                condition,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?,
        }),
        LoopFlowExpr::Or(lhs, rhs) => Some(ResolvedLoopFlowExpr::Or(
            Box::new(resolve_loop_flow_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_flow_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
    }
}

fn collect_resolved_loop_flow_leaves<'a>(
    expr: &'a ResolvedLoopFlowExpr,
    leaves: &mut Vec<(&'a ResolvedLoopControlExpr, &'a str)>,
) {
    match expr {
        ResolvedLoopFlowExpr::Legacy { condition, action }
        | ResolvedLoopFlowExpr::Terminal { condition, action } => {
            leaves.push((condition, action.as_str()));
        }
        ResolvedLoopFlowExpr::Or(lhs, rhs) => {
            collect_resolved_loop_flow_leaves(lhs, leaves);
            collect_resolved_loop_flow_leaves(rhs, leaves);
        }
    }
}

fn canonical_loop_instruction(instruction: &str) -> &str {
    match instruction {
        "loop_while_i64_chain" | "loop_while_scalar_chain" => "loop_while_scalar_chain",
        "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
            "loop_while_scalar_async_chain"
        }
        "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
            "loop_while_scalar_async_cond_chain"
        }
        "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
            "loop_while_scalar_cond_chain"
        }
        "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain" => {
            "loop_while_scalar_flow_chain"
        }
        "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain" => {
            "loop_while_scalar_async_flow_chain"
        }
        "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain" => {
            "loop_while_scalar_flow_cond_chain"
        }
        "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain" => {
            "loop_while_scalar_async_flow_cond_chain"
        }
        "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain" => {
            "loop_while_scalar_post_flow_chain"
        }
        "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain" => {
            "loop_while_scalar_async_post_flow_chain"
        }
        "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain" => {
            "loop_while_scalar_post_flow_cond_chain"
        }
        "loop_while_i64_async_post_flow_cond_chain"
        | "loop_while_scalar_async_post_flow_cond_chain" => {
            "loop_while_scalar_async_post_flow_cond_chain"
        }
        other => other,
    }
}

fn canonical_loop_block_prefix(instruction: &str) -> String {
    canonical_loop_instruction(instruction).replace('.', "_")
}

fn lower_cpu_literal_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "text" => {
            let label = fresh_global(&mut state.next_global);
            let (bytes, len) = llvm_c_string_bytes(&node.op.args[0]);
            state.globals.push(format!(
                "{label} = private unnamed_addr constant [{len} x i8] c\"{bytes}\""
            ));
            let ptr = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {ptr} = getelementptr inbounds [{len} x i8], ptr {label}, i64 0, i64 0"
            ));
            let handle = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {handle} = call i64 @nuis_host_text_lift(ptr {ptr})"
            ));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::TextHandle { ptr, handle });
            true
        }
        "const_bool" => {
            let value = match node.op.args[0].as_str() {
                "true" => "true",
                "false" => "false",
                _ => {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.const_bool `{}` because literal `{}` is invalid",
                        node.name, node.op.args[0]
                    ));
                    return true;
                }
            };
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {value} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1: value.to_owned(),
                    i64: widened.clone(),
                },
            );
            state.last_cpu_value = Some(widened);
            true
        }
        "const_i32" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = add i32 0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = sext i32 {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "const" | "const_i64" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = add i64 0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
            state.last_cpu_value = Some(reg);
            true
        }
        "const_f32" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = fadd float 0.0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = fptosi float {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "const_f64" => {
            let reg = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {reg} = fadd double 0.0, {}", node.op.args[0]));
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = fptosi double {reg} to i64"));
            state.last_cpu_value = Some(widened);
            true
        }
        "null" => {
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Ptr("null".to_owned()));
            true
        }
        _ => false,
    }
}

fn lower_cpu_aggregate_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "struct" => {
            let mut fields = Vec::new();
            let type_name = node.op.args[0].clone();
            for entry in &node.op.args[1..] {
                let Some((field_name, value_name)) = entry.split_once('=') else {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.struct `{}` because field binding `{}` is invalid",
                        node.name, entry
                    ));
                    return true;
                };
                let Some(value_ref) = state.registers.get(value_name.trim()).cloned() else {
                    state.body.push(format!(
                        "  ; deferred lowering for cpu.struct `{}` because field `{}` comes from outside the current CPU LLVM slice",
                        node.name, field_name
                    ));
                    return true;
                };
                fields.push((field_name.trim().to_owned(), value_ref));
            }
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Struct(StructLlvmValueRef { type_name, fields }),
            );
            true
        }
        "field" => {
            let Some(struct_value) = get_struct(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.field `{}` because its source struct is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let field_name = &node.op.args[1];
            let Some((_, field_value)) = struct_value
                .fields
                .iter()
                .find(|(name, _)| name == field_name)
            else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.field `{}` because field `{}` does not exist on `{}`",
                    node.name, field_name, struct_value.type_name
                ));
                return true;
            };
            let field_value = field_value.clone();
            state
                .registers
                .insert(node.name.clone(), field_value.clone());
            if let Some(as_i64) = coerce_to_i64(&field_value, &mut state.body, &mut state.next_reg)
            {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        _ => false,
    }
}

fn lower_cpu_pointer_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "borrow" | "move_ptr" => {
            let Some(ptr) = get_ptr(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for {} `{}` because its input is outside the current CPU LLVM slice",
                    node.op.full_name(),
                    node.name
                ));
                return true;
            };
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Ptr(ptr.to_owned()));
            if let Some(len) = state.buffer_lengths.get(&node.op.args[0]).cloned() {
                state.buffer_lengths.insert(node.name.clone(), len);
            }
            true
        }
        "borrow_end" => {
            state
                .registers
                .insert(node.name.clone(), LlvmValueRef::Void);
            true
        }
        _ => false,
    }
}

fn lower_network_observer_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "observe" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for network.observe `{}` because its input is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::NetworkResult(NetworkResultLlvmValueRef {
                    state: node.op.args[1].clone(),
                    value: Box::new(value_ref),
                }),
            );
            true
        }
        "is_config_ready" | "is_send_ready" | "is_recv_ready" | "is_connect_ready"
        | "is_accept_ready" | "is_closed" => {
            let Some(result) = get_network_result(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for network.{} `{}` because its result is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            let wanted_state = match node.op.instruction.as_str() {
                "is_config_ready" => "config_ready",
                "is_send_ready" => "send_ready",
                "is_recv_ready" => "recv_ready",
                "is_connect_ready" => "connect_ready",
                "is_accept_ready" => "accept_ready",
                "is_closed" => "closed",
                _ => unreachable!(),
            };
            let i1 = if result.state == wanted_state {
                "true".to_owned()
            } else {
                "false".to_owned()
            };
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {i1} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1,
                    i64: widened.clone(),
                },
            );
            state.last_cpu_value = Some(widened);
            true
        }
        "value" => {
            let Some(result) = get_network_result(&state.registers, &node.op.args[0]) else {
                state.body.push(format!(
                    "  ; deferred lowering for network.value `{}` because its result is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*result.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        _ => false,
    }
}

fn lower_cpu_async_resource_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    match node.op.instruction.as_str() {
        "spawn_task" => {
            let Some(value_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.spawn_task `{}` because its value is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Task(TaskLlvmValueRef {
                    value: Box::new(value_ref),
                }),
            );
            true
        }
        "spawn_thread" | "thread_spawn" => {
            let Some(value_ref) = state.registers.get(&node.op.args[1]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.{} `{}` because its value is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Thread(ThreadLlvmValueRef {
                    value: Box::new(value_ref),
                }),
            );
            true
        }
        "join_result" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.join_result `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::TaskResult(TaskResultLlvmValueRef {
                    state: "completed".to_owned(),
                    value: Some(task.value),
                }),
            );
            true
        }
        "thread_join_result" => {
            let Some(thread) = get_thread(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.thread_join_result `{}` because its thread is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::TaskResult(TaskResultLlvmValueRef {
                    state: "completed".to_owned(),
                    value: Some(thread.value),
                }),
            );
            true
        }
        "join" => {
            let Some(task) = get_task(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.join `{}` because its task is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*task.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        "thread_join" => {
            let Some(thread) = get_thread(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.thread_join `{}` because its thread is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*thread.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        "task_completed" | "task_timed_out" | "task_cancelled" => {
            let Some(result) = get_task_result(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.{} `{}` because its result is outside the current CPU LLVM slice",
                    node.op.instruction, node.name
                ));
                return true;
            };
            let i1 = match node.op.instruction.as_str() {
                "task_completed" if result.state == "completed" => "true",
                "task_timed_out" if result.state == "timed_out" => "true",
                "task_cancelled" if result.state == "cancelled" => "true",
                _ => "false",
            }
            .to_owned();
            let widened = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {widened} = zext i1 {i1} to i64"));
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Bool {
                    i1,
                    i64: widened.clone(),
                },
            );
            state.last_cpu_value = Some(widened);
            true
        }
        "task_value" => {
            let Some(result) = get_task_result(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.task_value `{}` because its result is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let Some(value_ref) = result.value.map(|value| *value) else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.task_value `{}` because its result carries no payload",
                    node.name
                ));
                return true;
            };
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        "mutex_new" => {
            let Some(value_ref) = state.registers.get(&node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_new `{}` because its value is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Mutex(MutexLlvmValueRef {
                    value: Box::new(value_ref),
                }),
            );
            true
        }
        "mutex_lock" => {
            let Some(mutex) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_lock `{}` because its mutex is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::MutexGuard(MutexGuardLlvmValueRef { value: mutex.value }),
            );
            true
        }
        "mutex_unlock" => {
            let Some(guard) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_unlock `{}` because its guard is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            state.registers.insert(
                node.name.clone(),
                LlvmValueRef::Mutex(MutexLlvmValueRef { value: guard.value }),
            );
            true
        }
        "mutex_value" => {
            let Some(guard) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
                state.body.push(format!(
                    "  ; deferred lowering for cpu.mutex_value `{}` because its guard is outside the current CPU LLVM slice",
                    node.name
                ));
                return true;
            };
            let value_ref = (*guard.value).clone();
            state.registers.insert(node.name.clone(), value_ref.clone());
            if let Some(as_i64) = coerce_to_i64(&value_ref, &mut state.body, &mut state.next_reg) {
                state.last_cpu_value = Some(as_i64);
            }
            true
        }
        _ => false,
    }
}

fn cpu_call_scalar_kind_for_instruction(instruction: &str) -> Option<CpuCallScalarKind> {
    match instruction {
        "param_bool" | "call_bool" | "return_bool" => Some(CpuCallScalarKind::Bool),
        "param_i32" | "call_i32" | "return_i32" => Some(CpuCallScalarKind::I32),
        "param_i64" | "call_i64" | "return_i64" => Some(CpuCallScalarKind::I64),
        "param_f32" | "call_f32" | "return_f32" => Some(CpuCallScalarKind::F32),
        "param_f64" | "call_f64" | "return_f64" => Some(CpuCallScalarKind::F64),
        _ => None,
    }
}

fn cpu_scalar_kind_llvm_type(kind: CpuCallScalarKind) -> &'static str {
    match kind {
        CpuCallScalarKind::Bool => "i1",
        CpuCallScalarKind::I32 => "i32",
        CpuCallScalarKind::I64 => "i64",
        CpuCallScalarKind::F32 => "float",
        CpuCallScalarKind::F64 => "double",
    }
}

fn cpu_param_binding(kind: CpuCallScalarKind, index: usize) -> LlvmValueRef {
    let arg = format!("%arg{index}");
    match kind {
        CpuCallScalarKind::Bool => {
            let widened = format!("%arg{index}_i64");
            LlvmValueRef::Bool {
                i1: arg,
                i64: widened,
            }
        }
        CpuCallScalarKind::I32 => LlvmValueRef::I32(arg),
        CpuCallScalarKind::I64 => LlvmValueRef::I64(arg),
        CpuCallScalarKind::F32 => LlvmValueRef::F32(arg),
        CpuCallScalarKind::F64 => LlvmValueRef::F64(arg),
    }
}

fn emit_typed_return_from_value(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    return_value: &LlvmValueRef,
) -> bool {
    match function_return_kind {
        CpuCallScalarKind::Bool => match return_value {
            LlvmValueRef::Bool { i1, .. } => {
                body.push(format!("  ret i1 {i1}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_bool = fresh_reg(next_reg);
                body.push(format!("  {as_bool} = icmp ne i64 {value}, 0"));
                body.push(format!("  ret i1 {as_bool}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::I32 => match return_value {
            LlvmValueRef::I32(value) => {
                body.push(format!("  ret i32 {value}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_i32 = fresh_reg(next_reg);
                body.push(format!("  {as_i32} = trunc i64 {value} to i32"));
                body.push(format!("  ret i32 {as_i32}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::I64 => {
            if let Some(returned) = coerce_to_i64(return_value, body, next_reg) {
                body.push(format!("  ret i64 {returned}"));
                true
            } else {
                false
            }
        }
        CpuCallScalarKind::F32 => match return_value {
            LlvmValueRef::F32(value) => {
                body.push(format!("  ret float {value}"));
                true
            }
            LlvmValueRef::F64(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = fptrunc double {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = sitofp i64 {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::I32(value) => {
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_f32} = sitofp i32 {value} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let as_f32 = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {as_f32} = sitofp i64 {as_i64} to float"));
                body.push(format!("  ret float {as_f32}"));
                true
            }
            _ => false,
        },
        CpuCallScalarKind::F64 => match return_value {
            LlvmValueRef::F64(value) => {
                body.push(format!("  ret double {value}"));
                true
            }
            LlvmValueRef::F32(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = fpext float {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::I64(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = sitofp i64 {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::I32(value) => {
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_f64} = sitofp i32 {value} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let as_f64 = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {as_f64} = sitofp i64 {as_i64} to double"));
                body.push(format!("  ret double {as_f64}"));
                true
            }
            _ => false,
        },
    }
}

fn can_emit_typed_return_from_value(
    function_return_kind: CpuCallScalarKind,
    return_value: &LlvmValueRef,
) -> bool {
    match function_return_kind {
        CpuCallScalarKind::Bool => {
            matches!(
                return_value,
                LlvmValueRef::Bool { .. } | LlvmValueRef::I64(_)
            )
        }
        CpuCallScalarKind::I32 => {
            matches!(return_value, LlvmValueRef::I32(_) | LlvmValueRef::I64(_))
        }
        CpuCallScalarKind::I64 => matches!(
            return_value,
            LlvmValueRef::I64(_)
                | LlvmValueRef::TextHandle { .. }
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
                | LlvmValueRef::F32(_)
                | LlvmValueRef::F64(_)
        ),
        CpuCallScalarKind::F32 => matches!(
            return_value,
            LlvmValueRef::F32(_)
                | LlvmValueRef::F64(_)
                | LlvmValueRef::I64(_)
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
        ),
        CpuCallScalarKind::F64 => matches!(
            return_value,
            LlvmValueRef::F64(_)
                | LlvmValueRef::F32(_)
                | LlvmValueRef::I64(_)
                | LlvmValueRef::I32(_)
                | LlvmValueRef::Bool { .. }
        ),
    }
}

fn emit_typed_return_from_last_value(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    function_return_kind: CpuCallScalarKind,
    last_value: &str,
) {
    match function_return_kind {
        CpuCallScalarKind::Bool => {
            let as_bool = fresh_reg(next_reg);
            body.push(format!("  {as_bool} = icmp ne i64 {last_value}, 0"));
            body.push(format!("  ret i1 {as_bool}"));
        }
        CpuCallScalarKind::I32 => {
            let as_i32 = fresh_reg(next_reg);
            body.push(format!("  {as_i32} = trunc i64 {last_value} to i32"));
            body.push(format!("  ret i32 {as_i32}"));
        }
        CpuCallScalarKind::I64 => {
            body.push(format!("  ret i64 {last_value}"));
        }
        CpuCallScalarKind::F32 => {
            let as_f32 = fresh_reg(next_reg);
            body.push(format!("  {as_f32} = sitofp i64 {last_value} to float"));
            body.push(format!("  ret float {as_f32}"));
        }
        CpuCallScalarKind::F64 => {
            let as_f64 = fresh_reg(next_reg);
            body.push(format!("  {as_f64} = sitofp i64 {last_value} to double"));
            body.push(format!("  ret double {as_f64}"));
        }
    }
}

fn emit_cpu_function(
    module: &YirModule,
    resources: &BTreeMap<String, &Resource>,
    ordered_node_names: &[String],
    param_bindings: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    function_return_kind: CpuCallScalarKind,
    global_counter: &mut usize,
) -> Result<EmittedCpuFunction, String> {
    let mut state = LlvmLoweringState {
        body: Vec::new(),
        globals: Vec::new(),
        registers: BTreeMap::new(),
        buffer_lengths: BTreeMap::new(),
        next_reg: 0,
        next_global: *global_counter,
        next_block: 0,
        last_cpu_value: None,
        ends_with_terminal_return: false,
    };

    for (node_name, value) in param_bindings {
        state.registers.insert(node_name.clone(), value.clone());
        match value {
            LlvmValueRef::Bool { i1, i64 } => {
                state.body.push(format!("  {i64} = zext i1 {i1} to i64"));
                state.last_cpu_value = Some(i64.clone());
            }
            LlvmValueRef::I32(reg) => {
                let widened = fresh_reg(&mut state.next_reg);
                state
                    .body
                    .push(format!("  {widened} = sext i32 {reg} to i64"));
                state.last_cpu_value = Some(widened);
            }
            LlvmValueRef::I64(reg) => state.last_cpu_value = Some(reg.clone()),
            _ => {}
        }
    }

    for node_name in ordered_node_names {
        let node = module
            .nodes
            .iter()
            .find(|node| &node.name == node_name)
            .ok_or_else(|| format!("lowering references unknown node `{node_name}`"))?;
        let resource = resources
            .get(&node.resource)
            .copied()
            .ok_or_else(|| format!("unknown resource `{}`", node.resource))?;

        if resource.kind.is_family("network") && lower_network_observer_node(node, &mut state) {
            continue;
        }

        if !resource.kind.is_family("cpu") {
            state.body.push(format!(
                "  ; deferred lowering for {} on {} ({})",
                node.op.full_name(),
                node.resource,
                resource.kind.raw
            ));
            continue;
        }

        if lower_cpu_async_resource_node(node, &mut state) {
            continue;
        }

        match node.op.cpu_llvm_lowering_class() {
            CpuLlvmLoweringClass::Literal => {
                if lower_cpu_literal_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Aggregate => {
                if lower_cpu_aggregate_node(node, &mut state) {
                    continue;
                }
            }
            CpuLlvmLoweringClass::Pointer => {
                if lower_cpu_pointer_node(node, &mut state) {
                    continue;
                }
            }
            _ => {}
        }

        let mut body = &mut state.body;
        let _globals = &mut state.globals;
        let registers = &mut state.registers;
        let buffer_lengths = &mut state.buffer_lengths;
        let mut next_reg = &mut state.next_reg;
        let mut next_block = &mut state.next_block;
        let _next_global = &mut state.next_global;
        let last_cpu_value = &mut state.last_cpu_value;
        state.ends_with_terminal_return = false;

        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "text")
            | ("cpu", "const_bool")
            | ("cpu", "const_i32")
            | ("cpu", "const")
            | ("cpu", "const_i64")
            | ("cpu", "const_f32")
            | ("cpu", "const_f64")
            | ("cpu", "struct")
            | ("cpu", "field")
            | ("cpu", "null")
            | ("cpu", "borrow")
            | ("cpu", "borrow_end")
            | ("cpu", "move_ptr") => unreachable!(
                "preclassified CPU LLVM lowering op `{}` should have been handled earlier",
                node.op.full_name()
            ),
            ("cpu", "param_bool") | ("cpu", "param_i32") | ("cpu", "param_i64") => {
                if let Some(input) = coerce_to_i64(
                    registers
                        .get(&node.name)
                        .expect("parameter binding should exist"),
                    &mut body,
                    &mut next_reg,
                ) {
                    *last_cpu_value = Some(input);
                }
            }
            ("cpu", "not") => {
                let Some(input) = get_i64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.not `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = xor i64 {input}, -1"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "add") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fadd double {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fadd float {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = add i64 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = add i32 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "add_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "add_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fadd float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "add_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fadd double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "eq") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp oeq double {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp oeq float {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    *last_cpu_value = Some(reg);
                } else {
                    let lhs_value = registers.get(&node.op.args[0]).cloned();
                    let rhs_value = registers.get(&node.op.args[1]).cloned();
                    if let (Some(lhs), Some(rhs)) = (
                        lhs_value
                            .as_ref()
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg)),
                        rhs_value
                            .as_ref()
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg)),
                    ) {
                        let cmp = fresh_reg(&mut next_reg);
                        body.push(format!("  {cmp} = icmp eq i64 {lhs}, {rhs}"));
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                        registers.insert(
                            node.name.clone(),
                            LlvmValueRef::Bool {
                                i1: cmp.clone(),
                                i64: reg.clone(),
                            },
                        );
                        *last_cpu_value = Some(reg);
                    } else {
                        body.push(format!(
                        "  ; deferred lowering for cpu.eq `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                        continue;
                    }
                }
            }
            ("cpu", "eq_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.eq_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp eq i32 {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "eq_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.eq_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp oeq float {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "eq_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.eq_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp oeq double {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "ne") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp one double {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp one float {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: reg.clone(),
                        },
                    );
                    *last_cpu_value = Some(reg);
                } else {
                    let lhs_value = registers.get(&node.op.args[0]).cloned();
                    let rhs_value = registers.get(&node.op.args[1]).cloned();
                    if let (Some(lhs), Some(rhs)) = (
                        lhs_value
                            .as_ref()
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg)),
                        rhs_value
                            .as_ref()
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg)),
                    ) {
                        let cmp = fresh_reg(&mut next_reg);
                        body.push(format!("  {cmp} = icmp ne i64 {lhs}, {rhs}"));
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                        registers.insert(
                            node.name.clone(),
                            LlvmValueRef::Bool {
                                i1: cmp.clone(),
                                i64: reg.clone(),
                            },
                        );
                        *last_cpu_value = Some(reg);
                    } else {
                        body.push(format!(
                        "  ; deferred lowering for cpu.ne `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                        continue;
                    }
                }
            }
            ("cpu", "lt") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp olt double {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp olt float {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp slt i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp slt i32 {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.lt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "lt_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.lt_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp slt i32 {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "lt_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.lt_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp olt float {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "lt_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.lt_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp olt double {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "gt") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp ogt double {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp ogt float {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sgt i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sgt i32 {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.gt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "gt_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.gt_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp sgt i32 {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "gt_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.gt_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp ogt float {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "gt_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.gt_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = fcmp ogt double {lhs}, {rhs}"));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1: cmp.clone(),
                        i64: widened.clone(),
                    },
                );
                *last_cpu_value = Some(widened);
            }
            ("cpu", "le") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp ole double {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp ole float {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sle i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sle i32 {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.le `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "ge") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp oge double {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = fcmp oge float {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sge i64 {lhs}, {rhs}"));
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let cmp = fresh_reg(&mut next_reg);
                    body.push(format!("  {cmp} = icmp sge i32 {lhs}, {rhs}"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {cmp} to i64"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::Bool {
                            i1: cmp.clone(),
                            i64: widened.clone(),
                        },
                    );
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.ge `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "sub") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fsub double {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fsub float {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = sub i64 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = sub i32 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "sub_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sub i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "sub_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fsub float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "sub_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fsub double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "mul") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fmul double {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fmul float {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = mul i64 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = mul i32 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "mul_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = mul i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "mul_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fmul float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "mul_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fmul double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "div") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fdiv double {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = fdiv float {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = sdiv i64 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = sdiv i32 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.div `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "div_i32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.div_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sdiv i32 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "div_f32") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f32(&registers, &node.op.args[0]),
                    get_f32(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.div_f32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fdiv float {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "div_f64") => {
                let (Some(lhs), Some(rhs)) = (
                    get_f64(&registers, &node.op.args[0]),
                    get_f64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.div_f64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fdiv double {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "rem") => {
                if let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = srem i64 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                } else if let (Some(lhs), Some(rhs)) = (
                    get_i32(&registers, &node.op.args[0]),
                    get_i32(&registers, &node.op.args[1]),
                ) {
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {reg} = srem i32 {lhs}, {rhs}"));
                    registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {reg} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.rem `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                }
            }
            ("cpu", "and") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.and `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = and i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "or") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.or `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = or i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "xor") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.xor `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = xor i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "shl") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.shl `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = shl i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "shr") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.shr `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = ashr i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "madd") => {
                let (Some(lhs), Some(rhs), Some(acc)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                    get_i64(&registers, &node.op.args[2]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.madd `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let mul = fresh_reg(&mut next_reg);
                body.push(format!("  {mul} = mul i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 {mul}, {acc}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "select") => {
                let cond_value = registers.get(&node.op.args[0]).cloned();
                let then_value = registers.get(&node.op.args[1]).cloned();
                let else_value = registers.get(&node.op.args[2]).cloned();
                let (Some(cond_value), Some(then_value), Some(else_value)) =
                    (cond_value, then_value, else_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.select `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(cond) = coerce_to_i64(&cond_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.select `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let Some(then_value) = coerce_to_i64(&then_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.select `{}` because its then-value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let Some(else_value) = coerce_to_i64(&else_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.select `{}` because its else-value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let cond_bool = fresh_reg(&mut next_reg);
                body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {reg} = select i1 {cond_bool}, i64 {then_value}, i64 {else_value}"
                ));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "cast_i32_to_i64") => {
                let Some(input) = get_i32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_i32_to_i64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sext i32 {input} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "cast_bool_to_i64") => {
                let Some(input) = get_bool(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_bool_to_i64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {input} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "cast_i64_to_bool") => {
                let Some(input) = get_i64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_i64_to_bool `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let i1 = fresh_reg(&mut next_reg);
                body.push(format!("  {i1} = icmp ne i64 {input}, 0"));
                let i64 = fresh_reg(&mut next_reg);
                body.push(format!("  {i64} = zext i1 {i1} to i64"));
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Bool {
                        i1,
                        i64: i64.clone(),
                    },
                );
                *last_cpu_value = Some(i64);
            }
            ("cpu", "cast_i64_to_i32") => {
                let Some(input) = get_i64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_i64_to_i32 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = trunc i64 {input} to i32"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "cast_i32_to_f32") => {
                let Some(input) = get_i32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_i32_to_f32 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sitofp i32 {input} to float"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "cast_i32_to_f64") => {
                let Some(input) = get_i32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_i32_to_f64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sitofp i32 {input} to double"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "cast_f32_to_f64") => {
                let Some(input) = get_f32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_f32_to_f64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fpext float {input} to double"));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "cast_f64_to_f32") => {
                let Some(input) = get_f64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.cast_f64_to_f32 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fptrunc double {input} to float"));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                *last_cpu_value = Some(widened);
            }
            ("cpu", "alloc_node") => {
                let (Some(value), Some(next_ptr)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_ptr(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.alloc_node `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let raw = fresh_reg(&mut next_reg);
                body.push(format!("  {raw} = call ptr @malloc(i64 16)"));
                let value_slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {value_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 0"
                ));
                body.push(format!("  store i64 {value}, ptr {value_slot}"));
                let next_slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_slot} = getelementptr inbounds %cpu.node, ptr {raw}, i32 0, i32 1"
                ));
                body.push(format!("  store ptr {next_ptr}, ptr {next_slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(raw));
            }
            ("cpu", "alloc_buffer") => {
                let (Some(len), Some(fill)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.alloc_buffer `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let len = len.to_owned();
                let fill = fill.to_owned();
                let bytes = fresh_reg(&mut next_reg);
                body.push(format!("  {bytes} = mul i64 {len}, 8"));
                let raw = fresh_reg(&mut next_reg);
                body.push(format!("  {raw} = call ptr @malloc(i64 {bytes})"));
                lower_buffer_fill(
                    &mut body,
                    &mut next_reg,
                    raw.as_str(),
                    len.as_str(),
                    fill.as_str(),
                )?;
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(raw.clone()));
                buffer_lengths.insert(node.name.clone(), len);
            }
            ("cpu", "load_value") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.load_value `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
                ));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = load i64, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "load_next") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.load_next `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
                ));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = load ptr, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(reg));
                if let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() {
                    buffer_lengths.insert(node.name.clone(), len);
                }
            }
            ("cpu", "buffer_len") => {
                let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.buffer_len `{}` because its input buffer length is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                registers.insert(node.name.clone(), LlvmValueRef::I64(len.clone()));
                *last_cpu_value = Some(len);
            }
            ("cpu", "load_at") => {
                let (Some(ptr), Some(index)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.load_at `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"
                ));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = load i64, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "store_value") => {
                let (Some(ptr), Some(value)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.store_value `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 0"
                ));
                body.push(format!("  store i64 {value}, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "store_next") => {
                let (Some(ptr), Some(next_ptr)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_ptr(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.store_next `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds %cpu.node, ptr {ptr}, i32 0, i32 1"
                ));
                body.push(format!("  store ptr {next_ptr}, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "store_at") => {
                let (Some(ptr), Some(index), Some(value)) = (
                    get_ptr(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                    get_i64(&registers, &node.op.args[2]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.store_at `{}` because its inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let slot = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"
                ));
                body.push(format!("  store i64 {value}, ptr {slot}"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "is_null") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.is_null `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp eq ptr {ptr}, null"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "free") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.free `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                body.push(format!("  call void @free(ptr {ptr})"));
                registers.insert(node.name.clone(), LlvmValueRef::Void);
            }
            ("cpu", "input_i64") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  ; static AOT lowering freezes cpu.input_i64 `{}` to its default value",
                    node.op.args[0]
                ));
                body.push(format!("  {reg} = add i64 0, {}", node.op.args[1]));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "extern_call_i64") => {
                let abi = &node.op.args[0];
                let symbol = &node.op.args[1];
                if abi != "nurs" && abi != "c" {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i64 `{}` because ABI `{}` is not supported by the current LLVM bridge",
                        node.name, abi
                    ));
                    continue;
                }
                let dynamic_args = !is_builtin_host_ffi_symbol(symbol);
                let lowered_args = node.op.args[2..]
                    .iter()
                    .map(|arg| {
                        registers.get(arg).and_then(|value| {
                            if dynamic_args {
                                lower_dynamic_extern_arg(value, &mut body, &mut next_reg)
                            } else {
                                lower_i64_extern_arg(value, &mut body, &mut next_reg)
                            }
                        })
                    })
                    .collect::<Option<Vec<_>>>();
                let Some(lowered_args) = lowered_args else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                let Some(call) = render_extern_call("i64", symbol, &lowered_args) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i64 `{}` because symbol `{}` has unsupported arity {}",
                        node.name,
                        symbol,
                        lowered_args.len()
                    ));
                    continue;
                };
                body.push(format!("  {reg} = {call}"));
                if symbol == "host_deserialize_text_from"
                    || symbol == "host_parse_header_line"
                    || symbol == "host_find_header_value"
                    || symbol == "host_find_status_line_reason"
                    || symbol == "host_parse_http_response_summary"
                    || symbol == "host_parse_http_request_summary"
                    || symbol == "host_parse_http_roundtrip_summary"
                {
                    let ptr = fresh_reg(&mut next_reg);
                    body.push(format!("  {ptr} = call ptr @nuis_host_text_ptr(i64 {reg})"));
                    registers.insert(
                        node.name.clone(),
                        LlvmValueRef::TextHandle {
                            ptr,
                            handle: reg.clone(),
                        },
                    );
                    *last_cpu_value = Some(reg);
                } else {
                    registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                    *last_cpu_value = Some(reg);
                }
            }
            ("cpu", "extern_call_i32") => {
                let abi = &node.op.args[0];
                let symbol = &node.op.args[1];
                if abi != "nurs" && abi != "c" {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i32 `{}` because ABI `{}` is not supported by the current LLVM bridge",
                        node.name, abi
                    ));
                    continue;
                }
                let dynamic_args = !is_builtin_host_ffi_symbol(symbol);
                let lowered_args = node.op.args[2..]
                    .iter()
                    .map(|arg| {
                        registers.get(arg).and_then(|value| {
                            if dynamic_args {
                                lower_dynamic_extern_arg(value, &mut body, &mut next_reg)
                            } else {
                                lower_i32_extern_arg(value, &mut body, &mut next_reg)
                            }
                        })
                    })
                    .collect::<Option<Vec<_>>>();
                let Some(lowered_args) = lowered_args else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i32 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                let Some(call) = render_extern_call("i32", symbol, &lowered_args) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.extern_call_i32 `{}` because symbol `{}` has unsupported arity {}",
                        node.name,
                        symbol,
                        lowered_args.len()
                    ));
                    continue;
                };
                body.push(format!("  {reg} = {call}"));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                *last_cpu_value = Some(reg);
            }
            ("cpu", "call_bool")
            | ("cpu", "call_i32")
            | ("cpu", "call_i64")
            | ("cpu", "call_f32")
            | ("cpu", "call_f64") => {
                let callee = &node.op.args[0];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{} `{}` because helper signature `{}` is unavailable",
                        node.op.instruction, node.name, callee
                    ));
                    continue;
                };
                let lowered_args = node.op.args[1..]
                    .iter()
                    .zip(signature.params.iter())
                    .map(|(arg, kind)| match kind {
                        CpuCallScalarKind::Bool => {
                            get_bool(&registers, arg).map(|value| format!("i1 {value}"))
                        }
                        CpuCallScalarKind::I32 => {
                            get_i32(&registers, arg).map(|value| format!("i32 {value}"))
                        }
                        CpuCallScalarKind::I64 => {
                            get_i64(&registers, arg).map(|value| format!("i64 {value}"))
                        }
                        CpuCallScalarKind::F32 => {
                            get_f32(&registers, arg).map(|value| format!("float {value}"))
                        }
                        CpuCallScalarKind::F64 => {
                            get_f64(&registers, arg).map(|value| format!("double {value}"))
                        }
                    })
                    .collect::<Option<Vec<_>>>();
                let Some(lowered_args) = lowered_args else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.op.instruction, node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                let symbol = format!("nuis_fn_{callee}");
                let call = match lowered_args.as_slice() {
                    [] => format!(
                        "call {} @{symbol}()",
                        cpu_scalar_kind_llvm_type(signature.ret)
                    ),
                    [a0] => format!(
                        "call {} @{symbol}({a0})",
                        cpu_scalar_kind_llvm_type(signature.ret)
                    ),
                    [a0, a1] => format!(
                        "call {} @{symbol}({a0}, {a1})",
                        cpu_scalar_kind_llvm_type(signature.ret)
                    ),
                    [a0, a1, a2] => format!(
                        "call {} @{symbol}({a0}, {a1}, {a2})",
                        cpu_scalar_kind_llvm_type(signature.ret)
                    ),
                    _ => {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{} `{}` because callee `{}` has unsupported arity {}",
                            node.op.instruction,
                            node.name,
                            callee,
                            lowered_args.len()
                        ));
                        continue;
                    }
                };
                body.push(format!("  {reg} = {call}"));
                match signature.ret {
                    CpuCallScalarKind::Bool => {
                        let widened = fresh_reg(&mut next_reg);
                        body.push(format!("  {widened} = zext i1 {reg} to i64"));
                        registers.insert(
                            node.name.clone(),
                            LlvmValueRef::Bool {
                                i1: reg.clone(),
                                i64: widened.clone(),
                            },
                        );
                        *last_cpu_value = Some(widened);
                    }
                    CpuCallScalarKind::I32 => {
                        registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                        let widened = fresh_reg(&mut next_reg);
                        body.push(format!("  {widened} = sext i32 {reg} to i64"));
                        *last_cpu_value = Some(widened);
                    }
                    CpuCallScalarKind::I64 => {
                        registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                        *last_cpu_value = Some(reg);
                    }
                    CpuCallScalarKind::F32 => {
                        registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                        let widened = fresh_reg(&mut next_reg);
                        body.push(format!("  {widened} = fpext float {reg} to double"));
                        let as_i64 = fresh_reg(&mut next_reg);
                        body.push(format!("  {as_i64} = fptosi double {widened} to i64"));
                        *last_cpu_value = Some(as_i64);
                    }
                    CpuCallScalarKind::F64 => {
                        registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                        let as_i64 = fresh_reg(&mut next_reg);
                        body.push(format!("  {as_i64} = fptosi double {reg} to i64"));
                        *last_cpu_value = Some(as_i64);
                    }
                }
            }
            ("cpu", "print") => {
                if let Some(input) = get_cstr(&registers, &node.op.args[0]) {
                    let print_reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
                    *last_cpu_value = Some("0".to_owned());
                } else if let Some(input) = get_i64(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                    *last_cpu_value = Some(input.to_owned());
                } else if let Some(input) = get_i32(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_i32(i32 {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {input} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let Some(input) = get_bool(&registers, &node.op.args[0]) {
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {input} to i32"));
                    body.push(format!("  call void @nuis_debug_print_bool(i32 {widened})"));
                    let widened64 = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened64} = zext i1 {input} to i64"));
                    *last_cpu_value = Some(widened64);
                } else if let Some(input) = get_f32(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_f32(float {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {input} to i64"));
                    *last_cpu_value = Some(widened);
                } else if let Some(input) = get_f64(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_f64(double {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {input} to i64"));
                    *last_cpu_value = Some(widened);
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.print `{}` because its input is produced outside the current CPU LLVM slice",
                        node.op.args[0]
                    ));
                }
            }
            ("cpu", "return_bool") => {
                let Some(input) = get_bool(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.return_bool `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                if let Some(LlvmValueRef::Bool { i64, .. }) = registers.get(&node.op.args[0]) {
                    *last_cpu_value = Some(i64.clone());
                }
                body.push(format!("  ret i1 {input}"));
                state.ends_with_terminal_return = true;
                break;
            }
            ("cpu", "return_i32") => {
                let Some(input) = get_i32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.return_i32 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                if let Some(as_i64) = coerce_to_i64(
                    registers
                        .get(&node.op.args[0])
                        .expect("return_i32 source should exist"),
                    &mut body,
                    &mut next_reg,
                ) {
                    *last_cpu_value = Some(as_i64);
                }
                body.push(format!("  ret i32 {input}"));
                state.ends_with_terminal_return = true;
                break;
            }
            ("cpu", "return_i64") => {
                let Some(input) = get_i64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.return_i64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                body.push(format!("  ret i64 {input}"));
                *last_cpu_value = Some(input.to_owned());
                state.ends_with_terminal_return = true;
                break;
            }
            ("cpu", "return_f32") => {
                let Some(input) = get_f32(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.return_f32 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fpext float {input} to double"));
                let as_i64 = fresh_reg(&mut next_reg);
                body.push(format!("  {as_i64} = fptosi double {widened} to i64"));
                *last_cpu_value = Some(as_i64);
                body.push(format!("  ret float {input}"));
                state.ends_with_terminal_return = true;
                break;
            }
            ("cpu", "return_f64") => {
                let Some(input) = get_f64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.return_f64 `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let as_i64 = fresh_reg(&mut next_reg);
                body.push(format!("  {as_i64} = fptosi double {input} to i64"));
                *last_cpu_value = Some(as_i64);
                body.push(format!("  ret double {input}"));
                state.ends_with_terminal_return = true;
                break;
            }
            ("cpu", "loop_while_i64") => {
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its initial value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its limit value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.loop_while_i64 `{}` because its step value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let loop_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {loop_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {loop_slot}"));
                let loop_cond = fresh_block(&mut next_block, "loop_while_i64_cond");
                let loop_body = fresh_block(&mut next_block, "loop_while_i64_body");
                let loop_exit = fresh_block(&mut next_block, "loop_while_i64_exit");
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {loop_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.loop_while_i64 `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_value = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.loop_while_i64 `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name
                        ));
                    }
                };
                body.push(format!("  store i64 {next_value}, ptr {loop_slot}"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(current.clone()));
                *last_cpu_value = Some(current);
            }
            ("cpu", "loop_while_i64_chain" | "loop_while_scalar_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let carry_payload_len = |kind: &str| -> usize {
                    let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                        fragment == "current"
                            || fragment == "prev_current"
                            || fragment
                                .strip_prefix("prev_carry")
                                .is_some_and(|index| index.parse::<usize>().is_ok())
                            || fragment
                                .strip_prefix("carry")
                                .is_some_and(|index| index.parse::<usize>().is_ok())
                    };
                    let add_state_list_payload_len = |kind: &str| -> Option<usize> {
                        let (terms_part, payload_len) =
                            if let Some(prefix) = kind.strip_prefix("add_") {
                                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                    (prefix, 1usize)
                                } else {
                                    (prefix, 0usize)
                                }
                            } else if let Some(prefix) = kind.strip_prefix("mul_") {
                                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                    (prefix, 1usize)
                                } else {
                                    (prefix, 0usize)
                                }
                            } else {
                                return None;
                            };
                        let terms = terms_part.split("_plus_").collect::<Vec<_>>();
                        if terms.len() < 2 {
                            return None;
                        }
                        if terms.iter().all(|term| carry_state_fragment_is_valid(term)) {
                            Some(payload_len)
                        } else {
                            None
                        }
                    };
                    let zero_payload_indexed_prefixes =
                        ["add_prev_carry", "mul_prev_carry", "add_carry", "mul_carry"];
                    let one_payload_zero_payload_indexed_prefixes =
                        ["add_prev_carry", "add_carry", "mul_prev_carry", "mul_carry"];
                    let one_payload_indexed_prefixes = [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ];
                    if kind.contains("_plus_factor_invariant_")
                        && kind.starts_with("mul_scaled_by_")
                    {
                        1 + usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.starts_with("mul_scaled_by_") {
                        usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.starts_with("mul_scaled_") {
                        1 + usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.ends_with("_plus_invariant")
                        && kind.starts_with("mul_")
                        && !matches!(
                            kind,
                            "mul_current_plus_invariant"
                                | "mul_prev_current_plus_invariant"
                                | "mul_invariant"
                                | "mul_source_plus_invariant"
                        )
                    {
                        1
                    } else if kind.starts_with("mul_")
                        && kind.contains("_plus_")
                        && !matches!(
                            kind,
                            "mul_current"
                                | "mul_prev_current"
                                | "mul_read_value_fixed"
                                | "mul_read_at_fixed"
                                | "mul_read_at_dynamic_current"
                                | "mul_read_at_dynamic_prev_current"
                        )
                    {
                        usize::from(kind.ends_with("_plus_invariant"))
                    } else if matches!(
                        kind,
                        "mul_current_plus_invariant"
                            | "mul_prev_current_plus_invariant"
                            | "mul_invariant"
                            | "mul_source_plus_invariant"
                    ) {
                        1
                    } else if matches!(
                        kind,
                        "keep"
                            | "keep_prev_carry"
                            | "add_current"
                            | "add_prev_current"
                            | "mul_current"
                            | "mul_prev_current"
                    ) || zero_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        0
                    } else if one_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        1
                    } else if one_payload_zero_payload_indexed_prefixes
                        .iter()
                        .any(|prefix| {
                            kind.strip_prefix(prefix).is_some_and(|suffix| {
                                suffix
                                    .strip_suffix("_plus_invariant")
                                    .is_some_and(|index| index.parse::<usize>().is_ok())
                            })
                        })
                    {
                        1
                    } else if let Some(payload_len) = add_state_list_payload_len(kind) {
                        payload_len
                    } else if matches!(
                        kind,
                        "add_read_value_fixed"
                            | "mul_read_value_fixed"
                            | "add_read_value_fixed_plus_invariant"
                            | "mul_read_value_fixed_plus_invariant"
                            | "add_invariant"
                            | "add_current_plus_invariant"
                            | "add_prev_current_plus_invariant"
                            | "mul_invariant"
                            | "mul_current_plus_invariant"
                            | "mul_prev_current_plus_invariant"
                    ) {
                        1
                    } else if matches!(
                        kind,
                        "add_read_at_fixed"
                            | "mul_read_at_fixed"
                            | "add_read_at_fixed_plus_invariant"
                            | "mul_read_at_fixed_plus_invariant"
                    ) {
                        if kind.ends_with("_plus_invariant") {
                            3
                        } else {
                            2
                        }
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current_plus_invariant"
                            | "add_read_at_dynamic_prev_current_plus_invariant"
                            | "mul_read_at_dynamic_current_plus_invariant"
                            | "mul_read_at_dynamic_prev_current_plus_invariant"
                    ) {
                        2
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current"
                            | "add_read_at_dynamic_prev_current"
                            | "mul_read_at_dynamic_current"
                            | "mul_read_at_dynamic_prev_current"
                            | "add_source_plus_invariant"
                            | "mul_source_plus_invariant"
                    ) {
                        1
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        1
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix).is_some_and(|suffix| {
                            suffix
                                .strip_suffix("_plus_invariant")
                                .is_some_and(|index| index.parse::<usize>().is_ok())
                        })
                    }) {
                        2
                    } else {
                        0
                    }
                };
                let mut carry_initial_values = Vec::new();
                let mut carry_specs_raw = Vec::new();
                let mut cursor = 5usize;
                while cursor < node.op.args.len() {
                    let carry_initial_name = &node.op.args[cursor];
                    let carry_kind = &node.op.args[cursor + 1];
                    let payload_len = carry_payload_len(carry_kind);
                    let payload_names = &node.op.args[cursor + 2..cursor + 2 + payload_len];
                    let carry_initial_value = registers.get(carry_initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        continue;
                    };
                    let mut payload_values = Vec::new();
                    let mut missing_payload = false;
                    for payload_name in payload_names {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        payload_values.push(payload_value);
                    }
                    if missing_payload {
                        continue;
                    }
                    carry_initial_values.push(carry_initial_value);
                    carry_specs_raw.push((carry_kind.clone(), payload_values));
                    cursor += 2 + payload_len;
                }
                let Some(loop_scalar_kind) = infer_loop_scalar_kind(
                    [&initial_value, &limit_value, &step_value]
                        .into_iter()
                        .chain(carry_initial_values.iter()),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its loop values are not representable as one scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_loop_scalar(
                    &initial_value,
                    loop_scalar_kind,
                    &mut body,
                    &mut next_reg,
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) =
                    coerce_to_loop_scalar(&limit_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) =
                    coerce_to_loop_scalar(&step_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let mut carry_initials = Vec::new();
                for carry_initial_value in &carry_initial_values {
                    let Some(carry_initial) = coerce_to_loop_scalar(
                        carry_initial_value,
                        loop_scalar_kind,
                        &mut body,
                        &mut next_reg,
                    ) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to the selected loop scalar kind",
                            node.name,
                        ));
                        continue;
                    };
                    carry_initials.push(carry_initial);
                }
                let mut carry_specs = Vec::new();
                for (carry_kind, payload_values) in &carry_specs_raw {
                    let mut payloads = Vec::new();
                    for payload_value in payload_values {
                        let Some(payload) = coerce_to_loop_scalar(
                            payload_value,
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        ) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are not coercible to the selected loop scalar kind",
                                node.name,
                            ));
                            continue;
                        };
                        payloads.push(payload);
                    }
                    carry_specs.push((carry_kind.clone(), payloads));
                }
                let scalar_ty = loop_scalar_llvm_type(loop_scalar_kind);
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca {scalar_ty}"));
                body.push(format!("  store {scalar_ty} {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca {scalar_ty}"));
                        body.push(format!(
                            "  store {scalar_ty} {carry_initial}, ptr {carry_slot}"
                        ));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let cmp = emit_loop_compare(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    cmp_kind,
                    &current,
                    &limit,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = emit_loop_numeric_op(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    step_kind,
                    &current,
                    &step,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {carry_before} = load {scalar_ty}, ptr {carry_slot}"
                    ));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, ((carry_kind, raw_payloads), (_, payloads))) in
                    carry_specs_raw.iter().zip(carry_specs.iter()).enumerate()
                {
                    let (source, op) = if carry_kind == "add_current" {
                        (next_current.clone(), "add")
                    } else if carry_kind == "add_prev_current" {
                        (current.clone(), "add")
                    } else if carry_kind == "mul_current" {
                        (next_current.clone(), "mul")
                    } else if carry_kind == "mul_prev_current" {
                        (current.clone(), "mul")
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_value_fixed" | "mul_read_value_fixed"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed read pointer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {ptr}"));
                        let source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_value_fixed_plus_invariant"
                            | "mul_read_value_fixed_plus_invariant"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed read pointer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let offset = payloads.last().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {ptr}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &read_source,
                            offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_fixed" | "mul_read_at_fixed"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let index_value = raw_payloads
                            .get(1)
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg))
                            .ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read index payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed indexed-read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_fixed_plus_invariant" | "mul_read_at_fixed_plus_invariant"
                    ) {
                        let ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let index_value = raw_payloads
                            .get(1)
                            .and_then(|value| coerce_to_i64(value, &mut body, &mut next_reg))
                            .ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing fixed indexed-read index payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        let offset = payloads.last().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce fixed indexed-read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &read_source,
                            offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if matches!(
                        carry_kind.as_str(),
                        "add_read_at_dynamic_current"
                            | "add_read_at_dynamic_prev_current"
                            | "mul_read_at_dynamic_current"
                            | "mul_read_at_dynamic_prev_current"
                            | "add_read_at_dynamic_current_plus_invariant"
                            | "add_read_at_dynamic_prev_current_plus_invariant"
                            | "mul_read_at_dynamic_current_plus_invariant"
                            | "mul_read_at_dynamic_prev_current_plus_invariant"
                    ) || carry_kind
                        .strip_prefix("add_read_at_dynamic_prev_carry")
                        .is_some()
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_prev_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_carry")
                            .is_some()
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_prev_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_prev_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("add_read_at_dynamic_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                        || carry_kind
                            .strip_prefix("mul_read_at_dynamic_carry")
                            .is_some_and(|suffix| suffix.ends_with("_plus_invariant"))
                    {
                        let buffer_ptr = match raw_payloads.first() {
                            Some(LlvmValueRef::Ptr(ptr)) => ptr.clone(),
                            _ => {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` is missing dynamic read buffer payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                        };
                        let dynamic_kind = carry_kind
                            .strip_suffix("_plus_invariant")
                            .unwrap_or(carry_kind.as_str());
                        let index_value = if dynamic_kind.ends_with("_prev_current") {
                            current.clone()
                        } else if dynamic_kind.ends_with("_current") {
                            next_current.clone()
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("add_read_at_dynamic_prev_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("mul_read_at_dynamic_prev_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("add_read_at_dynamic_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) =
                            dynamic_kind.strip_prefix("mul_read_at_dynamic_carry")
                        {
                            let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let slot = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {slot} = getelementptr inbounds i64, ptr {buffer_ptr}, i64 {index_value}"
                        ));
                        let loaded = fresh_reg(&mut next_reg);
                        body.push(format!("  {loaded} = load i64, ptr {slot}"));
                        let read_source = coerce_to_loop_scalar(
                            &LlvmValueRef::I64(loaded),
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        )
                        .ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` cannot coerce dynamic read source `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let source = if carry_kind.ends_with("_plus_invariant") {
                            let offset = payloads.last().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing invariant payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &read_source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            read_source
                        };
                        let op = if carry_kind.starts_with("add_") {
                            "add"
                        } else {
                            "mul"
                        };
                        (source, op)
                    } else if let Some(rest) = carry_kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(prefix) = carry_kind
                        .strip_prefix("mul_scaled_by_")
                        .and_then(|prefix| prefix.split_once("_plus_factor_invariant_"))
                    {
                        let (factor_term, terms_part) = prefix;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let factor_offset = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let factor = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "add",
                            &factor,
                            factor_offset,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            &factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if let Some(prefix) = carry_kind.strip_prefix("mul_scaled_by_") {
                        let (factor_term, terms_part) = prefix.split_once('_').ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            &factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if carry_kind.starts_with("mul_scaled_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_scaled_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let factor = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                offset,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        source = emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            "mul",
                            &source,
                            factor,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (source, "mul")
                    } else if carry_kind.starts_with("mul_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                &rhs,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        if has_invariant {
                            let payload = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            source = emit_loop_numeric_op(
                                &mut body,
                                &mut next_reg,
                                loop_scalar_kind,
                                "add",
                                &source,
                                payload,
                            )
                            .map_err(|error| {
                                format!(
                                    "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        }
                        (source, "mul")
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = emit_loop_numeric_op(
                        &mut body,
                        &mut next_reg,
                        loop_scalar_kind,
                        op,
                        &current_carries[index],
                        &source,
                    )
                    .map_err(|error| {
                        format!(
                            "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                            node.name,
                        )
                    })?;
                    next_carries.push(reg);
                }
                body.push(format!(
                    "  store {scalar_ty} {next_current}, ptr {current_slot}"
                ));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!(
                        "  store {scalar_ty} {next_carry}, ptr {carry_slot}"
                    ));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {final_current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {final_carry} = load {scalar_ty}, ptr {carry_slot}"
                        ));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    loop_scalar_value_ref(loop_scalar_kind, final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        loop_scalar_value_ref(loop_scalar_kind, final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries
                    .last()
                    .map(|carry| loop_scalar_value_ref(loop_scalar_kind, carry.clone()))
                    .or_else(|| {
                        Some(loop_scalar_value_ref(
                            loop_scalar_kind,
                            final_current.clone(),
                        ))
                    })
                    .and_then(|value| coerce_to_i64(&value, &mut body, &mut next_reg));
            }
            ("cpu", "loop_while_i64_async_chain" | "loop_while_scalar_async_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable",
                        node.name, callee
                    ));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`",
                        node.name, callee
                    ));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let carry_payload_len = |kind: &str| -> usize {
                    if kind.contains("_plus_factor_invariant_")
                        && kind.starts_with("mul_scaled_by_")
                    {
                        1 + usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.starts_with("mul_scaled_by_") {
                        usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.starts_with("mul_scaled_") {
                        1 + usize::from(kind.ends_with("_plus_invariant"))
                    } else if kind.starts_with("mul_") && kind.contains("_plus_") {
                        usize::from(kind.ends_with("_plus_invariant"))
                    } else if matches!(
                        kind,
                        "mul_current_plus_invariant"
                            | "mul_prev_current_plus_invariant"
                            | "mul_invariant"
                            | "mul_source_plus_invariant"
                    ) {
                        1
                    } else {
                        0
                    }
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut cursor = 4usize;
                while cursor < node.op.args.len() {
                    let carry_initial_name = &node.op.args[cursor];
                    let carry_kind = &node.op.args[cursor + 1];
                    let payload_len = carry_payload_len(carry_kind);
                    let payload_names = &node.op.args[cursor + 2..cursor + 2 + payload_len];
                    let carry_initial_value = registers.get(carry_initial_name).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        continue;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        continue;
                    };
                    let mut payloads = Vec::new();
                    let mut missing_payload = false;
                    for payload_name in payload_names {
                        let Some(payload_value) = registers.get(payload_name).cloned() else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        let Some(payload) = coerce_to_i64(&payload_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry payloads are not coercible to i64",
                                node.name,
                            ));
                            missing_payload = true;
                            break;
                        };
                        payloads.push(payload);
                    }
                    if missing_payload {
                        continue;
                    }
                    carry_initials.push(carry_initial);
                    carry_specs.push((carry_kind.clone(), payloads));
                    cursor += 2 + payload_len;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, (carry_kind, payloads)) in carry_specs.iter().enumerate() {
                    let (source, op) = if carry_kind == "add_current" {
                        (next_current.clone(), "add")
                    } else if carry_kind == "add_prev_current" {
                        (current.clone(), "add")
                    } else if carry_kind == "mul_current" {
                        (next_current.clone(), "mul")
                    } else if carry_kind == "mul_prev_current" {
                        (current.clone(), "mul")
                    } else if let Some(rest) = carry_kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "add",
                        )
                    } else if let Some(rest) = carry_kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?,
                            "mul",
                        )
                    } else if let Some(prefix) = carry_kind
                        .strip_prefix("mul_scaled_by_")
                        .and_then(|prefix| prefix.split_once("_plus_factor_invariant_"))
                    {
                        let (factor_term, terms_part) = prefix;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let factor_offset = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let factor_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {factor_reg} = add i64 {factor}, {factor_offset}"
                        ));
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor_reg}"));
                        (reg, "mul")
                    } else if let Some(prefix) = carry_kind.strip_prefix("mul_scaled_by_") {
                        let (factor_term, terms_part) = prefix.split_once('_').ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let (terms_part, has_invariant) =
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let factor = resolve_term(factor_term)?;
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor}"));
                        (reg, "mul")
                    } else if carry_kind.starts_with("mul_scaled_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_scaled_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let factor = payloads.first().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let offset = payloads.get(1).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {offset}"));
                            source = reg;
                        }
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = mul i64 {source}, {factor}"));
                        (reg, "mul")
                    } else if carry_kind.starts_with("mul_") {
                        let (terms_part, has_invariant) = if let Some(terms_part) =
                            carry_kind.strip_prefix("mul_")
                        {
                            if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            }
                        } else {
                            unreachable!()
                        };
                        let resolve_term = |term: &str| -> Result<String, String> {
                            match term {
                                "current" => Ok(next_current.clone()),
                                "prev_current" => Ok(current.clone()),
                                other if other.starts_with("prev_carry") => {
                                    let source_index = other[10..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    current_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                other if other.starts_with("carry") => {
                                    let source_index = other[5..].parse::<usize>().map_err(|_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })?;
                                    next_carries.get(source_index).cloned().ok_or_else(|| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                            node.name,
                                        )
                                    })
                                }
                                _ => Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )),
                            }
                        };
                        let mut terms = terms_part.split("_plus_");
                        let first = terms.next().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let mut source = resolve_term(first)?;
                        for term in terms {
                            let rhs = resolve_term(term)?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {rhs}"));
                            source = reg;
                        }
                        if has_invariant {
                            let payload = payloads.first().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` is missing carry payload for `{carry_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let reg = fresh_reg(&mut next_reg);
                            body.push(format!("  {reg} = add i64 {source}, {payload}"));
                            source = reg;
                        }
                        (source, "mul")
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = {op} i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable",
                        node.name, callee
                    ));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`",
                        node.name, callee
                    ));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[4..].chunks(5) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(&chunk[2]).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(cond_rhs) =
                            coerce_to_i64(&cond_rhs_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    };
                    carry_initials.push(carry_initial);
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => return Err(format!(
                        "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                        node.name,
                    )),
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let resolve_source = |kind: &str,
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<(String, &'static str), String> {
                    if matches!(kind, "keep" | "keep_prev_carry") {
                        return Ok((String::new(), "keep"));
                    }
                    if kind == "add_current" {
                        return Ok((next_current.clone(), "add"));
                    }
                    if kind == "add_prev_current" {
                        return Ok((current.clone(), "add"));
                    }
                    if kind == "mul_current" {
                        return Ok((next_current.clone(), "mul"));
                    }
                    if kind == "mul_prev_current" {
                        return Ok((current.clone(), "mul"));
                    }
                    if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    Err(format!(
                        "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                        node.name,
                    ))
                };
                let eval_cond = |kind: &str,
                                 rhs: &Option<String>,
                                 next_current: &String,
                                 next_carries: &Vec<String>,
                                 body: &mut Vec<String>,
                                 next_reg: &mut usize|
                 -> Result<String, String> {
                    if kind == "always" {
                        return Ok("1".to_owned());
                    }
                    let rhs = rhs.as_ref().ok_or_else(|| format!(
                        "cpu.{loop_instruction} `{}` is missing condition rhs for `{kind}` during LLVM lowering",
                        node.name,
                    ))?;
                    let (lhs, pred) = match kind {
                        "current_eq" => (next_current.clone(), "eq"),
                        "current_ne" => (next_current.clone(), "ne"),
                        "current_lt" => (next_current.clone(), "slt"),
                        "current_le" => (next_current.clone(), "sle"),
                        "current_gt" => (next_current.clone(), "sgt"),
                        "current_ge" => (next_current.clone(), "sge"),
                        "prev_current_eq" => (current.clone(), "eq"),
                        "prev_current_ne" => (current.clone(), "ne"),
                        "prev_current_lt" => (current.clone(), "slt"),
                        "prev_current_le" => (current.clone(), "sle"),
                        "prev_current_gt" => (current.clone(), "sgt"),
                        "prev_current_ge" => (current.clone(), "sge"),
                        other if other.starts_with("prev_carry") && other.ends_with("_eq") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "eq")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_ne") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "ne")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_lt") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "slt")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_le") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sle")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_gt") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sgt")
                        }
                        other if other.starts_with("prev_carry") && other.ends_with("_ge") => {
                            let idx = other[10..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (current_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sge")
                        }
                        other if other.starts_with("carry") && other.ends_with("_eq") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "eq")
                        }
                        other if other.starts_with("carry") && other.ends_with("_ne") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "ne")
                        }
                        other if other.starts_with("carry") && other.ends_with("_lt") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "slt")
                        }
                        other if other.starts_with("carry") && other.ends_with("_le") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sle")
                        }
                        other if other.starts_with("carry") && other.ends_with("_gt") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sgt")
                        }
                        other if other.starts_with("carry") && other.ends_with("_ge") => {
                            let idx = other[5..other.len()-3].parse::<usize>().map_err(|_| format!(
                                "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                                node.name,
                            ))?;
                            (next_carries.get(idx).cloned().ok_or_else(|| format!(
                                "cpu.{loop_instruction} `{}` references unavailable condition source `{other}` during LLVM lowering",
                                node.name,
                            ))?, "sge")
                        }
                        other => return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported condition kind `{other}` during LLVM lowering",
                            node.name,
                        )),
                    };
                    let reg = fresh_reg(next_reg);
                    body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                    Ok(reg)
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let cond = eval_cond(
                        cond_kind,
                        cond_rhs,
                        &next_current,
                        &next_carries,
                        &mut body,
                        &mut next_reg,
                    )?;
                    let then_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_then");
                    let else_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_else");
                    let merge_block =
                        fresh_block(&mut next_block, "loop_while_scalar_async_cond_merge");
                    body.push(format!(
                        "  br i1 {cond}, label %{then_block}, label %{else_block}"
                    ));
                    body.push(format!("{then_block}:"));
                    let (then_source, then_op) =
                        resolve_source(then_kind, &next_current, &next_carries)?;
                    let then_value = if matches!(then_op, "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = {then_op} i64 {}, {}",
                            current_carries[index], then_source
                        ));
                        reg
                    };
                    body.push(format!("  br label %{merge_block}"));
                    body.push(format!("{else_block}:"));
                    let (else_source, else_op) =
                        resolve_source(else_kind, &next_current, &next_carries)?;
                    let else_value = if matches!(else_op, "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = {else_op} i64 {}, {}",
                            current_carries[index], else_source
                        ));
                        reg
                    };
                    body.push(format!("  br label %{merge_block}"));
                    body.push(format!("{merge_block}:"));
                    let merged = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {merged} = phi i64 [{then_value}, %{then_block}], [{else_value}, %{else_block}]"
                    ));
                    next_carries.push(merged);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let mut carry_initial_values = Vec::new();
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[5..].chunks(5) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initial_values.push(carry_initial_value);
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(&chunk[2]).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs_value)
                    };
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
                }
                if deferred {
                    continue;
                }
                let Some(loop_scalar_kind) = infer_loop_scalar_kind(
                    [&initial_value, &limit_value, &step_value]
                        .into_iter()
                        .chain(carry_initial_values.iter())
                        .chain(
                            carry_specs
                                .iter()
                                .filter_map(|(_, cond_rhs, _, _)| cond_rhs.as_ref()),
                        ),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its loop values are not representable as one scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_loop_scalar(
                    &initial_value,
                    loop_scalar_kind,
                    &mut body,
                    &mut next_reg,
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) =
                    coerce_to_loop_scalar(&limit_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) =
                    coerce_to_loop_scalar(&step_value, loop_scalar_kind, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to the selected loop scalar kind",
                        node.name,
                    ));
                    continue;
                };
                let mut carry_initials = Vec::new();
                for carry_initial_value in &carry_initial_values {
                    let Some(carry_initial) = coerce_to_loop_scalar(
                        carry_initial_value,
                        loop_scalar_kind,
                        &mut body,
                        &mut next_reg,
                    ) else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to the selected loop scalar kind",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                }
                if deferred {
                    continue;
                }
                let mut lowered_carry_specs = Vec::new();
                for (cond_kind, cond_rhs, then_kind, else_kind) in carry_specs {
                    let lowered_cond_rhs = if let Some(cond_rhs) = cond_rhs {
                        let Some(cond_rhs) = coerce_to_loop_scalar(
                            &cond_rhs,
                            loop_scalar_kind,
                            &mut body,
                            &mut next_reg,
                        ) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to the selected loop scalar kind",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    } else {
                        None
                    };
                    lowered_carry_specs.push((cond_kind, lowered_cond_rhs, then_kind, else_kind));
                }
                if deferred {
                    continue;
                }
                let scalar_ty = loop_scalar_llvm_type(loop_scalar_kind);
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca {scalar_ty}"));
                body.push(format!("  store {scalar_ty} {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca {scalar_ty}"));
                        body.push(format!(
                            "  store {scalar_ty} {carry_initial}, ptr {carry_slot}"
                        ));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let cmp = emit_loop_compare(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    cmp_kind,
                    &current,
                    &limit,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = emit_loop_numeric_op(
                    &mut body,
                    &mut next_reg,
                    loop_scalar_kind,
                    step_kind,
                    &current,
                    &step,
                )
                .map_err(|error| {
                    format!(
                        "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                        node.name,
                    )
                })?;
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {carry_before} = load {scalar_ty}, ptr {carry_slot}"
                    ));
                    current_carries.push(carry_before);
                }
                let resolve_source = |kind: &str,
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<(String, &'static str), String> {
                    if matches!(kind, "keep" | "keep_prev_carry") {
                        return Ok((String::new(), "keep"));
                    }
                    if kind == "add_current" {
                        return Ok((next_current.clone(), "add"));
                    }
                    if kind == "add_prev_current" {
                        return Ok((current.clone(), "add"));
                    }
                    if kind == "mul_current" {
                        return Ok((next_current.clone(), "mul"));
                    }
                    if kind == "mul_prev_current" {
                        return Ok((current.clone(), "mul"));
                    }
                    if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return current_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "add")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    if let Some(rest) = kind.strip_prefix("mul_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return next_carries.get(source_index).cloned().map(|value| (value, "mul")).ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                            node.name,
                        ))
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    lowered_carry_specs.iter().enumerate()
                {
                    let then_value = if matches!(then_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let (source, op) = resolve_source(then_kind, &next_current, &next_carries)?;
                        emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            op,
                            &current_carries[index],
                            &source,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?
                    };
                    let else_value = if matches!(else_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let (source, op) = resolve_source(else_kind, &next_current, &next_carries)?;
                        emit_loop_numeric_op(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            op,
                            &current_carries[index],
                            &source,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let lhs = if matches!(
                            cond_kind.as_str(),
                            "current_eq"
                                | "current_ne"
                                | "current_lt"
                                | "current_le"
                                | "current_gt"
                                | "current_ge"
                        ) {
                            next_current.clone()
                        } else if matches!(
                            cond_kind.as_str(),
                            "prev_current_eq"
                                | "prev_current_ne"
                                | "prev_current_lt"
                                | "prev_current_le"
                                | "prev_current_gt"
                                | "prev_current_ge"
                        ) {
                            current.clone()
                        } else if let Some(rest) = cond_kind.strip_prefix("prev_carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) = cond_kind.strip_prefix("carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let rhs = cond_rhs.clone().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let cond_compare =
                            if cond_kind.ends_with("_eq") || cond_kind == "current_eq" {
                                "eq"
                            } else if cond_kind.ends_with("_ne") || cond_kind == "current_ne" {
                                "ne"
                            } else if cond_kind.ends_with("_lt") || cond_kind == "current_lt" {
                                "lt"
                            } else if cond_kind.ends_with("_le") || cond_kind == "current_le" {
                                "le"
                            } else if cond_kind.ends_with("_gt") || cond_kind == "current_gt" {
                                "gt"
                            } else {
                                "ge"
                            };
                        let cond_reg = emit_loop_compare(
                            &mut body,
                            &mut next_reg,
                            loop_scalar_kind,
                            cond_compare,
                            &lhs,
                            &rhs,
                        )
                        .map_err(|error| {
                            format!(
                                "cpu.{loop_instruction} `{}` {error} during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let select_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {select_reg} = select i1 {cond_reg}, {scalar_ty} {then_value}, {scalar_ty} {else_value}"
                        ));
                        select_reg
                    };
                    next_carries.push(next_carry);
                }
                body.push(format!(
                    "  store {scalar_ty} {next_current}, ptr {current_slot}"
                ));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!(
                        "  store {scalar_ty} {next_carry}, ptr {carry_slot}"
                    ));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {final_current} = load {scalar_ty}, ptr {current_slot}"
                ));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {final_carry} = load {scalar_ty}, ptr {carry_slot}"
                        ));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    loop_scalar_value_ref(loop_scalar_kind, final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        loop_scalar_value_ref(loop_scalar_kind, final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries
                    .last()
                    .map(|carry| loop_scalar_value_ref(loop_scalar_kind, carry.clone()))
                    .or_else(|| {
                        Some(loop_scalar_value_ref(
                            loop_scalar_kind,
                            final_current.clone(),
                        ))
                    })
                    .and_then(|value| coerce_to_i64(&value, &mut body, &mut next_reg));
            }
            ("cpu", "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let control_rhs_value = registers.get(&node.op.args[6]).cloned();
                let (
                    Some(initial_value),
                    Some(limit_value),
                    Some(step_value),
                    Some(control_rhs_value),
                ) = (initial_value, limit_value, step_value, control_rhs_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(control_rhs) = coerce_to_i64(&control_rhs_value, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its control rhs is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let control_kind = node.op.args[5].as_str();
                let control_action = node.op.args[7].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_kinds = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[8..].chunks(2) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                    carry_kinds.push(chunk[1].clone());
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_update =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_update"));
                let loop_action =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_action"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let (control_lhs, control_pred) = match control_kind {
                    "current_eq" => (next_current.clone(), "eq"),
                    "current_ne" => (next_current.clone(), "ne"),
                    "current_lt" => (next_current.clone(), "slt"),
                    "current_le" => (next_current.clone(), "sle"),
                    "current_gt" => (next_current.clone(), "sgt"),
                    "current_ge" => (next_current.clone(), "sge"),
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "eq")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "ne")
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "slt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sle")
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sgt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sge")
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let control_cond = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {control_cond} = icmp {control_pred} i64 {control_lhs}, {control_rhs}"
                ));
                body.push(format!(
                    "  br i1 {control_cond}, label %{loop_action}, label %{loop_update}"
                ));
                body.push(format!("{loop_action}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                match control_action {
                    "break" => body.push(format!("  br label %{loop_exit}")),
                    "continue" => body.push(format!("  br label %{loop_cond}")),
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control action `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                }
                body.push(format!("{loop_update}:"));
                let mut next_carries = Vec::new();
                for (index, carry_kind) in carry_kinds.iter().enumerate() {
                    let source = if carry_kind == "add_current" {
                        next_current.clone()
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = add i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let control_rhs_value = registers.get(&node.op.args[5]).cloned();
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable",
                        node.name, callee
                    ));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`",
                        node.name, callee
                    ));
                    continue;
                }
                let (Some(initial_value), Some(limit_value), Some(control_rhs_value)) =
                    (initial_value, limit_value, control_rhs_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(control_rhs) = coerce_to_i64(&control_rhs_value, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its control rhs is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let control_kind = node.op.args[4].as_str();
                let control_action = node.op.args[6].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_kinds = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[7..].chunks(2) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                    carry_kinds.push(chunk[1].clone());
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_update =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_update"));
                let loop_action =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_action"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let (control_lhs, control_pred) = match control_kind {
                    "current_eq" => (next_current.clone(), "eq"),
                    "current_ne" => (next_current.clone(), "ne"),
                    "current_lt" => (next_current.clone(), "slt"),
                    "current_le" => (next_current.clone(), "sle"),
                    "current_gt" => (next_current.clone(), "sgt"),
                    "current_ge" => (next_current.clone(), "sge"),
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "eq")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "ne")
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "slt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sle")
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sgt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sge")
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let control_cond = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {control_cond} = icmp {control_pred} i64 {control_lhs}, {control_rhs}"
                ));
                body.push(format!(
                    "  br i1 {control_cond}, label %{loop_action}, label %{loop_update}"
                ));
                body.push(format!("{loop_action}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                match control_action {
                    "break" => body.push(format!("  br label %{loop_exit}")),
                    "continue" => body.push(format!("  br label %{loop_cond}")),
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control action `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                }
                body.push(format!("{loop_update}:"));
                let mut next_carries = Vec::new();
                for (index, carry_kind) in carry_kinds.iter().enumerate() {
                    let source = if carry_kind == "add_current" {
                        next_current.clone()
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = add i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            (
                "cpu",
                "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain",
            ) => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                fn resolve_control_operand(
                    kind: &str,
                    next_current: &String,
                    current_carries: &Vec<String>,
                    node_name: &str,
                ) -> Result<(String, &'static str), String> {
                    match kind {
                        "current_eq" => Ok((next_current.clone(), "eq")),
                        "current_ne" => Ok((next_current.clone(), "ne")),
                        "current_lt" => Ok((next_current.clone(), "slt")),
                        "current_le" => Ok((next_current.clone(), "sle")),
                        "current_gt" => Ok((next_current.clone(), "sgt")),
                        "current_ge" => Ok((next_current.clone(), "sge")),
                        other if other.starts_with("carry") && other.ends_with("_eq") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "eq"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ne") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "ne"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_lt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "slt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_le") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sle"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_gt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sgt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ge") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sge"))
                        }
                        other => Err(format!("cpu.loop_while_scalar_async_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name)),
                    }
                }
                fn eval_control_expr(
                    expr: &ResolvedLoopControlExpr,
                    next_current: &String,
                    current_carries: &Vec<String>,
                    body: &mut Vec<String>,
                    next_reg: &mut usize,
                    node_name: &str,
                ) -> Result<String, String> {
                    match expr {
                        ResolvedLoopControlExpr::Cond { kind, rhs } => {
                            let (lhs, pred) = resolve_control_operand(
                                kind,
                                next_current,
                                current_carries,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::And(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::Or(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                current_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                    }
                }
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 4, &node.name, loop_instruction)?;
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable", node.name, callee));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`", node.name, callee));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice", node.name));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64", node.name));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64", node.name));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let Some(carry_initial_value) = registers.get(&chunk[0]).cloned() else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice", node.name));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64", node.name));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let Some(v) = registers.get(&chunk[2]).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(rhs) = coerce_to_i64(&v, &mut body, &mut next_reg) else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        Some(rhs)
                    };
                    carry_initials.push(carry_initial);
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|init| {
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!("  {s} = alloca i64"));
                        body.push(format!("  store i64 {init}, ptr {s}"));
                        s
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_update =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_update"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred=match cmp_kind { "eq"=>"eq","ne"=>"ne","lt"=>"slt","le"=>"sle","gt"=>"sgt","ge"=>"sge",other=>return Err(format!("cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering", node.name)), };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for slot in &carry_slots {
                    let r = fresh_reg(&mut next_reg);
                    body.push(format!("  {r} = load i64, ptr {slot}"));
                    current_carries.push(r);
                }
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let mut no_match_block = loop_update.clone();
                for (index, (condition, action)) in flow_leaves.iter().enumerate().rev() {
                    let leaf_block = if index == 0 {
                        None
                    } else {
                        Some(fresh_block(&mut next_block, "loop_async_flow_rhs"))
                    };
                    if let Some(block) = &leaf_block {
                        body.push(format!("{block}:"));
                    }
                    let control_cond = eval_control_expr(
                        condition,
                        &next_current,
                        &current_carries,
                        &mut body,
                        &mut next_reg,
                        &node.name,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_async_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                    if let Some(block) = leaf_block {
                        no_match_block = block;
                    }
                }
                body.push(format!("  br label %{no_match_block}"));
                body.push(format!("{loop_update}:"));
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let resolve = |kind: &str,
                                   next_carries: &Vec<String>|
                     -> Result<(String, &'static str), String> {
                        if matches!(kind, "keep" | "keep_prev_carry") {
                            return Ok((String::new(), "keep"));
                        }
                        if kind == "add_current" {
                            return Ok((next_current.clone(), "add"));
                        }
                        if kind == "add_prev_current" {
                            return Ok((current.clone(), "add"));
                        }
                        if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                            let i=rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))?;
                            return Ok((current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering", node.name))?,"add"));
                        }
                        if let Some(rest) = kind.strip_prefix("add_carry") {
                            let i=rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))?;
                            return Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering", node.name))?,"add"));
                        }
                        Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering", node.name))
                    };
                    let then_value = {
                        let (src, op) = resolve(then_kind, &next_carries)?;
                        if matches!(op, "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        }
                    };
                    let else_value = {
                        let (src, op) = resolve(else_kind, &next_carries)?;
                        if matches!(op, "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        }
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let rhs=cond_rhs.clone().ok_or_else(|| format!("cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering", node.name))?;
                        let (lhs,pred)=match cond_kind.as_str() {
                            "current_eq"=>(next_current.clone(),"eq"), "current_ne"=>(next_current.clone(),"ne"), "current_lt"=>(next_current.clone(),"slt"), "current_le"=>(next_current.clone(),"sle"), "current_gt"=>(next_current.clone(),"sgt"), "current_ge"=>(next_current.clone(),"sge"),
                            "prev_current_eq"=>(current.clone(),"eq"), "prev_current_ne"=>(current.clone(),"ne"), "prev_current_lt"=>(current.clone(),"slt"), "prev_current_le"=>(current.clone(),"sle"), "prev_current_gt"=>(current.clone(),"sgt"), "prev_current_ge"=>(current.clone(),"sge"),
                            other if other.starts_with("prev_carry") && other.ends_with("_eq") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ne") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("prev_carry") && other.ends_with("_lt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_le") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("prev_carry") && other.ends_with("_gt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ge") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other if other.starts_with("carry") && other.ends_with("_eq") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("carry") && other.ends_with("_ne") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("carry") && other.ends_with("_lt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("carry") && other.ends_with("_le") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("carry") && other.ends_with("_gt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("carry") && other.ends_with("_ge") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other => return Err(format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name)),
                        };
                        let c = fresh_reg(&mut next_reg);
                        body.push(format!("  {c} = icmp {pred} i64 {lhs}, {rhs}"));
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {s} = select i1 {c}, i64 {then_value}, i64 {else_value}"
                        ));
                        s
                    };
                    next_carries.push(next_carry);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {val}, ptr {slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|slot| {
                        let r = fresh_reg(&mut next_reg);
                        body.push(format!("  {r} = load i64, ptr {slot}"));
                        r
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, fc) in final_carries.iter().enumerate() {
                    fields.push((format!("carry{index}"), LlvmValueRef::I64(fc.clone())));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let control_rhs_value = registers.get(&node.op.args[6]).cloned();
                let (
                    Some(initial_value),
                    Some(limit_value),
                    Some(step_value),
                    Some(control_rhs_value),
                ) = (initial_value, limit_value, step_value, control_rhs_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(control_rhs) = coerce_to_i64(&control_rhs_value, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its control rhs is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let control_kind = node.op.args[5].as_str();
                let control_action = node.op.args[7].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_kinds = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[8..].chunks(2) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                    carry_kinds.push(chunk[1].clone());
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_action =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_action"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, carry_kind) in carry_kinds.iter().enumerate() {
                    let source = if carry_kind == "add_current" {
                        next_current.clone()
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = add i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                let (control_lhs, control_pred) = match control_kind {
                    "current_eq" => (next_current.clone(), "eq"),
                    "current_ne" => (next_current.clone(), "ne"),
                    "current_lt" => (next_current.clone(), "slt"),
                    "current_le" => (next_current.clone(), "sle"),
                    "current_gt" => (next_current.clone(), "sgt"),
                    "current_ge" => (next_current.clone(), "sge"),
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "eq")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "ne")
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "slt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sle")
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sgt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sge")
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let control_cond = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {control_cond} = icmp {control_pred} i64 {control_lhs}, {control_rhs}"
                ));
                body.push(format!(
                    "  br i1 {control_cond}, label %{loop_action}, label %{loop_continue}"
                ));
                body.push(format!("{loop_action}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                match control_action {
                    "break" => body.push(format!("  br label %{loop_exit}")),
                    "continue" => body.push(format!("  br label %{loop_cond}")),
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control action `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                }
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain",
            ) => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let control_rhs_value = registers.get(&node.op.args[5]).cloned();
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable",
                        node.name, callee
                    ));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`",
                        node.name, callee
                    ));
                    continue;
                }
                let (Some(initial_value), Some(limit_value), Some(control_rhs_value)) =
                    (initial_value, limit_value, control_rhs_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(control_rhs) = coerce_to_i64(&control_rhs_value, &mut body, &mut next_reg)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its control rhs is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let control_kind = node.op.args[4].as_str();
                let control_action = node.op.args[6].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_kinds = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[7..].chunks(2) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    carry_initials.push(carry_initial);
                    carry_kinds.push(chunk[1].clone());
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_action =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_action"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let mut next_carries = Vec::new();
                for (index, carry_kind) in carry_kinds.iter().enumerate() {
                    let source = if carry_kind == "add_current" {
                        next_current.clone()
                    } else if let Some(rest) = carry_kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{carry_kind}` during LLVM lowering",
                                node.name,
                            )
                        })?
                    } else {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
                            node.name,
                        ));
                    };
                    let reg = fresh_reg(&mut next_reg);
                    body.push(format!(
                        "  {reg} = add i64 {}, {}",
                        current_carries[index], source
                    ));
                    next_carries.push(reg);
                }
                let (control_lhs, control_pred) = match control_kind {
                    "current_eq" => (next_current.clone(), "eq"),
                    "current_ne" => (next_current.clone(), "ne"),
                    "current_lt" => (next_current.clone(), "slt"),
                    "current_le" => (next_current.clone(), "sle"),
                    "current_gt" => (next_current.clone(), "sgt"),
                    "current_ge" => (next_current.clone(), "sge"),
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "eq")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "ne")
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "slt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sle")
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sgt")
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                            |_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                    node.name,
                                )
                            },
                        )?;
                        let lhs = next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        (lhs, "sge")
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let control_cond = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {control_cond} = icmp {control_pred} i64 {control_lhs}, {control_rhs}"
                ));
                body.push(format!(
                    "  br i1 {control_cond}, label %{loop_action}, label %{loop_continue}"
                ));
                body.push(format!("{loop_action}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                match control_action {
                    "break" => body.push(format!("  br label %{loop_exit}")),
                    "continue" => body.push(format!("  br label %{loop_cond}")),
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported control action `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                }
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            (
                "cpu",
                "loop_while_i64_async_post_flow_cond_chain"
                | "loop_while_scalar_async_post_flow_cond_chain",
            ) => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                fn resolve_control_operand(
                    kind: &str,
                    next_current: &String,
                    next_carries: &Vec<String>,
                    node_name: &str,
                ) -> Result<(String, &'static str), String> {
                    match kind {
                        "current_eq" => Ok((next_current.clone(), "eq")),
                        "current_ne" => Ok((next_current.clone(), "ne")),
                        "current_lt" => Ok((next_current.clone(), "slt")),
                        "current_le" => Ok((next_current.clone(), "sle")),
                        "current_gt" => Ok((next_current.clone(), "sgt")),
                        "current_ge" => Ok((next_current.clone(), "sge")),
                        other if other.starts_with("carry") && other.ends_with("_eq") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "eq"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ne") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "ne"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_lt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "slt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_le") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sle"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_gt") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sgt"))
                        }
                        other if other.starts_with("carry") && other.ends_with("_ge") => {
                            let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name))?;
                            Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` references unavailable control source `{other}` during LLVM lowering", node_name))?, "sge"))
                        }
                        other => Err(format!("cpu.loop_while_scalar_async_post_flow_cond_chain `{}` has unsupported control kind `{other}` during LLVM lowering", node_name)),
                    }
                }
                fn eval_control_expr(
                    expr: &ResolvedLoopControlExpr,
                    next_current: &String,
                    next_carries: &Vec<String>,
                    body: &mut Vec<String>,
                    next_reg: &mut usize,
                    node_name: &str,
                ) -> Result<String, String> {
                    match expr {
                        ResolvedLoopControlExpr::Cond { kind, rhs } => {
                            let (lhs, pred) = resolve_control_operand(
                                kind,
                                next_current,
                                next_carries,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::And(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                next_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                next_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                        ResolvedLoopControlExpr::Or(lhs, rhs) => {
                            let lhs_reg = eval_control_expr(
                                lhs,
                                next_current,
                                next_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let rhs_reg = eval_control_expr(
                                rhs,
                                next_current,
                                next_carries,
                                body,
                                next_reg,
                                node_name,
                            )?;
                            let reg = fresh_reg(next_reg);
                            body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
                            Ok(reg)
                        }
                    }
                }
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 4, &node.name, loop_instruction)?;
                let callee = &node.op.args[2];
                let Some(signature) = helper_signatures.get(callee) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper signature `{}` is unavailable", node.name, callee));
                    continue;
                };
                if signature.params.as_slice() != [CpuCallScalarKind::I64]
                    || signature.ret != CpuCallScalarKind::I64
                {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because helper `{}` must have signature `(i64) -> i64`", node.name, callee));
                    continue;
                }
                let (Some(initial_value), Some(limit_value)) = (initial_value, limit_value) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice", node.name));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64", node.name));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64", node.name));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let carry_source_payload_len = |kind: &str| -> Option<usize> {
                    let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                        match fragment {
                            "current" | "prev_current" => true,
                            other => other
                                .strip_prefix("prev_carry")
                                .or_else(|| other.strip_prefix("carry"))
                                .is_some_and(|index| index.parse::<usize>().is_ok()),
                        }
                    };
                    let add_state_list_payload_len = |kind: &str| -> Option<usize> {
                        let (terms_part, payload_len) = if let Some(prefix) =
                            kind.strip_prefix("add_scaled_by_")
                        {
                            if let Some((lhs_group, rest)) =
                                prefix.split_once("_times_factor_group_")
                            {
                                let parse_group = |group: &str| -> Option<bool> {
                                    if let Some(group) =
                                        group.strip_suffix("_plus_factor_invariant")
                                    {
                                        let terms = group.split("_plus_").collect::<Vec<_>>();
                                        if terms.is_empty()
                                            || !terms
                                                .iter()
                                                .all(|term| carry_state_fragment_is_valid(term))
                                        {
                                            return None;
                                        }
                                        Some(true)
                                    } else {
                                        let terms = group.split("_plus_").collect::<Vec<_>>();
                                        if terms.is_empty()
                                            || !terms
                                                .iter()
                                                .all(|term| carry_state_fragment_is_valid(term))
                                        {
                                            return None;
                                        }
                                        Some(false)
                                    }
                                };
                                let lhs_offset = parse_group(lhs_group)?;
                                if let Some((rhs_group, rest)) =
                                    rest.split_once("_times_factor_invariant_times_terms_")
                                {
                                    let rhs_offset = parse_group(rhs_group)?;
                                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 2usize,
                                        )
                                    } else {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 1usize,
                                        )
                                    }
                                } else {
                                    let (rhs_group, rest) = rest.split_once("_times_terms_")?;
                                    let rhs_offset = parse_group(rhs_group)?;
                                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                        (
                                            rest,
                                            usize::from(lhs_offset)
                                                + usize::from(rhs_offset)
                                                + 1usize,
                                        )
                                    } else {
                                        (rest, usize::from(lhs_offset) + usize::from(rhs_offset))
                                    }
                                }
                            } else if let Some((factor_terms, rest)) = prefix
                                .split_once("_plus_factor_invariant_times_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 3usize)
                                } else {
                                    (rest, 2usize)
                                }
                            } else if let Some((factor_terms, rest)) =
                                prefix.split_once("_times_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.len() < 2
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some((factor_terms, rest)) =
                                prefix.split_once("_plus_factor_invariant_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some((factor_terms, rest)) = prefix.split_once("_times_")
                            {
                                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                                if factor_terms.len() < 2
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 1usize)
                                } else {
                                    (rest, 0usize)
                                }
                            } else if let Some((factor, rest)) =
                                prefix.split_once("_plus_factor_invariant_")
                            {
                                if !carry_state_fragment_is_valid(factor) {
                                    return None;
                                }
                                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                                    (rest, 2usize)
                                } else {
                                    (rest, 1usize)
                                }
                            } else if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                let (_, rest) = prefix.split_once('_')?;
                                (rest, 1usize)
                            } else {
                                let (_, rest) = prefix.split_once('_')?;
                                (rest, 0usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 2usize)
                            } else {
                                (prefix, 1usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 1usize)
                            } else {
                                (prefix, 0usize)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("mul_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (prefix, 1usize)
                            } else {
                                (prefix, 0usize)
                            }
                        } else {
                            return None;
                        };
                        let terms = terms_part.split("_plus_").collect::<Vec<_>>();
                        if terms.len() < 2 {
                            return None;
                        }
                        if terms.iter().all(|term| carry_state_fragment_is_valid(term)) {
                            Some(payload_len)
                        } else {
                            None
                        }
                    };
                    let zero_payload_indexed_prefixes =
                        ["add_prev_carry", "mul_prev_carry", "add_carry", "mul_carry"];
                    let one_payload_zero_payload_indexed_prefixes =
                        ["add_prev_carry", "add_carry", "mul_prev_carry", "mul_carry"];
                    let one_payload_indexed_prefixes = [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ];
                    if matches!(
                        kind,
                        "keep"
                            | "keep_prev_carry"
                            | "add_current"
                            | "add_prev_current"
                            | "mul_current"
                            | "mul_prev_current"
                    ) || zero_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(0)
                    } else if one_payload_indexed_prefixes.iter().any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(1)
                    } else if one_payload_zero_payload_indexed_prefixes
                        .iter()
                        .any(|prefix| {
                            kind.strip_prefix(prefix).is_some_and(|suffix| {
                                suffix
                                    .strip_suffix("_plus_invariant")
                                    .is_some_and(|index| index.parse::<usize>().is_ok())
                            })
                        })
                    {
                        Some(1)
                    } else if let Some(payload_len) = add_state_list_payload_len(kind) {
                        Some(payload_len)
                    } else if matches!(
                        kind,
                        "add_read_value_fixed"
                            | "mul_read_value_fixed"
                            | "add_read_value_fixed_plus_invariant"
                            | "mul_read_value_fixed_plus_invariant"
                            | "add_invariant"
                            | "add_current_plus_invariant"
                            | "add_prev_current_plus_invariant"
                            | "mul_invariant"
                            | "mul_current_plus_invariant"
                            | "mul_prev_current_plus_invariant"
                    ) {
                        Some(1)
                    } else if matches!(
                        kind,
                        "add_read_at_fixed"
                            | "mul_read_at_fixed"
                            | "add_read_at_fixed_plus_invariant"
                            | "mul_read_at_fixed_plus_invariant"
                    ) {
                        Some(if kind.ends_with("_plus_invariant") {
                            3
                        } else {
                            2
                        })
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current_plus_invariant"
                            | "add_read_at_dynamic_prev_current_plus_invariant"
                            | "mul_read_at_dynamic_current_plus_invariant"
                            | "mul_read_at_dynamic_prev_current_plus_invariant"
                    ) {
                        Some(2)
                    } else if matches!(
                        kind,
                        "add_read_at_dynamic_current"
                            | "add_read_at_dynamic_prev_current"
                            | "mul_read_at_dynamic_current"
                            | "mul_read_at_dynamic_prev_current"
                            | "add_source_plus_invariant"
                            | "mul_source_plus_invariant"
                    ) {
                        Some(1)
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix)
                            .is_some_and(|index| index.parse::<usize>().is_ok())
                    }) {
                        Some(1)
                    } else if [
                        "add_read_at_dynamic_prev_carry",
                        "mul_read_at_dynamic_prev_carry",
                        "add_read_at_dynamic_carry",
                        "mul_read_at_dynamic_carry",
                    ]
                    .iter()
                    .any(|prefix| {
                        kind.strip_prefix(prefix).is_some_and(|suffix| {
                            suffix
                                .strip_suffix("_plus_invariant")
                                .is_some_and(|index| index.parse::<usize>().is_ok())
                        })
                    }) {
                        Some(2)
                    } else {
                        None
                    }
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs =
                    Vec::<(String, Option<String>, Vec<String>, Vec<String>)>::new();
                let mut deferred = false;
                let mut cursor = carry_start_index;
                while cursor < node.op.args.len() {
                    let chunk0 = node.op.args.get(cursor);
                    let chunk1 = node.op.args.get(cursor + 1);
                    let chunk2 = node.op.args.get(cursor + 2);
                    let Some(initial_name) = chunk0 else {
                        break;
                    };
                    let Some(cond_kind_name) = chunk1 else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
                    };
                    let Some(cond_rhs_name) = chunk2 else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated carry spec during LLVM lowering", node.name));
                    };
                    let Some(iv) = registers.get(initial_name).cloned() else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice", node.name));
                        deferred = true;
                        break;
                    };
                    let Some(init) = coerce_to_i64(&iv, &mut body, &mut next_reg) else {
                        body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64", node.name));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if cond_kind_name == "always" {
                        None
                    } else {
                        let Some(v) = registers.get(cond_rhs_name).cloned() else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice", node.name));
                            deferred = true;
                            break;
                        };
                        let Some(rhs) = coerce_to_i64(&v, &mut body, &mut next_reg) else {
                            body.push(format!("  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64", node.name));
                            deferred = true;
                            break;
                        };
                        Some(rhs)
                    };
                    let then_start = cursor + 3;
                    let Some(then_kind) = node.op.args.get(then_start).cloned() else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated then carry source during LLVM lowering", node.name));
                    };
                    let Some(then_payload_len) = carry_source_payload_len(&then_kind) else {
                        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{then_kind}` during LLVM lowering", node.name));
                    };
                    let then_end = then_start + 1 + then_payload_len;
                    if then_end > node.op.args.len() {
                        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{then_kind}` during LLVM lowering", node.name));
                    }
                    let then_source = node.op.args[then_start..then_end].to_vec();
                    let Some(else_kind) = node.op.args.get(then_end).cloned() else {
                        return Err(format!("cpu.{loop_instruction} `{}` has truncated else carry source during LLVM lowering", node.name));
                    };
                    let Some(else_payload_len) = carry_source_payload_len(&else_kind) else {
                        return Err(format!("cpu.{loop_instruction} `{}` has unsupported carry kind `{else_kind}` during LLVM lowering", node.name));
                    };
                    let else_end = then_end + 1 + else_payload_len;
                    if else_end > node.op.args.len() {
                        return Err(format!("cpu.{loop_instruction} `{}` is missing payload for carry kind `{else_kind}` during LLVM lowering", node.name));
                    }
                    let else_source = node.op.args[then_end..else_end].to_vec();
                    carry_initials.push(init);
                    carry_specs.push((cond_kind_name.clone(), cond_rhs, then_source, else_source));
                    cursor = else_end;
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|init| {
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!("  {s} = alloca i64"));
                        body.push(format!("  store i64 {init}, ptr {s}"));
                        s
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred=match cmp_kind { "eq"=>"eq","ne"=>"ne","lt"=>"slt","le"=>"sle","gt"=>"sgt","ge"=>"sge",other=>return Err(format!("cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering", node.name)), };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {next_current} = call i64 @nuis_fn_{callee}(i64 {current})"
                ));
                let mut current_carries = Vec::new();
                for slot in &carry_slots {
                    let r = fresh_reg(&mut next_reg);
                    body.push(format!("  {r} = load i64, ptr {slot}"));
                    current_carries.push(r);
                }
                fn resolve_state_term_for_async_post_flow(
                    term: &str,
                    current: &str,
                    next_current: &str,
                    current_carries: &[String],
                    next_carries: &[String],
                    node_name: &str,
                    loop_instruction: &str,
                ) -> Result<String, String> {
                    match term {
                        "current" => Ok(next_current.to_owned()),
                        "prev_current" => Ok(current.to_owned()),
                        other if other.starts_with("prev_carry") => {
                            let i = other[10..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
                            current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
                        }
                        other if other.starts_with("carry") => {
                            let i = other[5..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
                            next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
                        }
                        other => Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering")),
                    }
                }
                fn resolve_source_for_async_post_flow(
                    source_spec: &[String],
                    current: &str,
                    next_current: &str,
                    current_carries: &[String],
                    next_carries: &[String],
                    body: &mut Vec<String>,
                    next_reg: &mut usize,
                    node_name: &str,
                    loop_instruction: &str,
                ) -> Result<String, String> {
                    let Some(kind) = source_spec.first() else {
                        return Err(format!("cpu.{loop_instruction} `{node_name}` has empty carry source during LLVM lowering"));
                    };
                    if matches!(kind.as_str(), "keep" | "keep_prev_carry") {
                        return Ok(String::new());
                    }
                    if kind == "add_current" {
                        return Ok(next_current.to_owned());
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let i = rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))?;
                        return next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry source `{kind}` during LLVM lowering"));
                    }
                    let parse_factor_group = |group: &str| -> Option<(Vec<String>, bool)> {
                        if let Some(group) = group.strip_suffix("_plus_factor_invariant") {
                            let terms =
                                group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
                            if terms.is_empty()
                                || !terms.iter().all(|term| {
                                    matches!(term.as_str(), "current" | "prev_current")
                                        || term.starts_with("prev_carry")
                                        || term.starts_with("carry")
                                })
                            {
                                return None;
                            }
                            Some((terms, true))
                        } else {
                            let terms =
                                group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
                            if terms.is_empty()
                                || !terms.iter().all(|term| {
                                    matches!(term.as_str(), "current" | "prev_current")
                                        || term.starts_with("prev_carry")
                                        || term.starts_with("carry")
                                })
                            {
                                return None;
                            }
                            Some((terms, false))
                        }
                    };
                    if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
                        if let Some((lhs_group, rest)) = prefix.split_once("_times_factor_group_") {
                            let (lhs_terms, lhs_has_offset) = parse_factor_group(lhs_group)
                                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
                            let (rhs_group, terms_part, has_factor_scale) = if let Some((
                                rhs_group,
                                terms_part,
                            )) =
                                rest.split_once("_times_factor_invariant_times_terms_")
                            {
                                (rhs_group, terms_part, true)
                            } else if let Some((rhs_group, terms_part)) =
                                rest.split_once("_times_terms_")
                            {
                                (rhs_group, terms_part, false)
                            } else {
                                return Err(format!(
                                        "cpu.{loop_instruction} `{node_name}` has malformed factor-group carry kind `{kind}` during LLVM lowering"
                                    ));
                            };
                            let (rhs_terms, rhs_has_offset) = parse_factor_group(rhs_group)
                                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
                            let (terms_part, has_invariant) = if let Some(terms_part) =
                                terms_part.strip_suffix("_plus_invariant")
                            {
                                (terms_part, true)
                            } else {
                                (terms_part, false)
                            };
                            let terms = terms_part
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>();
                            let mut payload_index = 1usize;
                            let resolve_group =
                                |group_terms: &[String],
                                 has_offset: bool,
                                 payload_index: &mut usize,
                                 body: &mut Vec<String>,
                                 next_reg: &mut usize|
                                 -> Result<String, String> {
                                    let mut acc = resolve_state_term_for_async_post_flow(
                                        &group_terms[0],
                                        current,
                                        next_current,
                                        current_carries,
                                        next_carries,
                                        node_name,
                                        loop_instruction,
                                    )?;
                                    for term in group_terms.iter().skip(1) {
                                        let rhs = resolve_state_term_for_async_post_flow(
                                            term,
                                            current,
                                            next_current,
                                            current_carries,
                                            next_carries,
                                            node_name,
                                            loop_instruction,
                                        )?;
                                        let sum = fresh_reg(next_reg);
                                        body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                                        acc = sum;
                                    }
                                    if has_offset {
                                        let offset_name = source_spec.get(*payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-group offset payload for `{kind}` during LLVM lowering"))?;
                                        *payload_index += 1;
                                        let sum = fresh_reg(next_reg);
                                        body.push(format!(
                                            "  {sum} = add i64 {acc}, {offset_name}"
                                        ));
                                        acc = sum;
                                    }
                                    Ok(acc)
                                };
                            let lhs = resolve_group(
                                &lhs_terms,
                                lhs_has_offset,
                                &mut payload_index,
                                body,
                                next_reg,
                            )?;
                            let rhs = resolve_group(
                                &rhs_terms,
                                rhs_has_offset,
                                &mut payload_index,
                                body,
                                next_reg,
                            )?;
                            let mut factor = fresh_reg(next_reg);
                            body.push(format!("  {factor} = mul i64 {lhs}, {rhs}"));
                            if has_factor_scale {
                                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-scale payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let scaled_factor = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {scaled_factor} = mul i64 {factor}, {factor_scale_name}"
                                ));
                                factor = scaled_factor;
                            }
                            let mut acc = resolve_state_term_for_async_post_flow(
                                &terms[0],
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            for term in terms.iter().skip(1) {
                                let rhs = resolve_state_term_for_async_post_flow(
                                    term,
                                    current,
                                    next_current,
                                    current_carries,
                                    next_carries,
                                    node_name,
                                    loop_instruction,
                                )?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                                acc = sum;
                            }
                            if has_invariant {
                                let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                                acc = sum;
                            }
                            let scaled = fresh_reg(next_reg);
                            body.push(format!("  {scaled} = mul i64 {acc}, {factor}"));
                            return Ok(scaled);
                        }
                    }
                    let parse_add_terms = |kind: &str| -> Option<(
                        Option<Vec<String>>,
                        bool,
                        bool,
                        bool,
                        Vec<String>,
                        bool,
                    )> {
                        let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                            matches!(fragment, "current" | "prev_current")
                                || fragment.starts_with("prev_carry")
                                || fragment.starts_with("carry")
                        };
                        let (
                            factor_term,
                            scaled_by_payload,
                            factor_invariant_payload,
                            factor_scale_payload,
                            terms_part,
                            has_invariant,
                        ) = if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
                            let (prefix, has_invariant) =
                                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                    (prefix, true)
                                } else {
                                    (prefix, false)
                                };
                            if let Some((factor_terms, terms_part)) = prefix
                                .split_once("_plus_factor_invariant_times_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    true,
                                    true,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_times_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    false,
                                    true,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_plus_factor_invariant_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    true,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            } else if let Some((factor_terms, terms_part)) =
                                prefix.split_once("_times_")
                            {
                                (
                                    Some(
                                        factor_terms
                                            .split("_plus_")
                                            .map(str::to_owned)
                                            .collect::<Vec<_>>(),
                                    ),
                                    false,
                                    false,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            } else {
                                let (factor_term, factor_invariant_payload, terms_part) =
                                    if let Some((factor_term, terms_part)) =
                                        prefix.split_once("_plus_factor_invariant_")
                                    {
                                        (Some(vec![factor_term.to_owned()]), true, terms_part)
                                    } else {
                                        let (factor_term, terms_part) = prefix.split_once('_')?;
                                        (Some(vec![factor_term.to_owned()]), false, terms_part)
                                    };
                                (
                                    factor_term,
                                    false,
                                    factor_invariant_payload,
                                    false,
                                    terms_part,
                                    has_invariant,
                                )
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (None, true, false, false, prefix, true)
                            } else {
                                (None, true, false, false, prefix, false)
                            }
                        } else if let Some(prefix) = kind.strip_prefix("add_") {
                            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                                (None, false, false, false, prefix, true)
                            } else {
                                (None, false, false, false, prefix, false)
                            }
                        } else {
                            return None;
                        };
                        let terms = terms_part
                            .split("_plus_")
                            .map(|term| term.to_owned())
                            .collect::<Vec<_>>();
                        if terms.iter().all(|term| {
                            matches!(term.as_str(), "current" | "prev_current")
                                || term.starts_with("prev_carry")
                                || term.starts_with("carry")
                        }) {
                            if let Some(factor_terms) = factor_term.as_ref() {
                                if factor_terms.is_empty()
                                    || !factor_terms
                                        .iter()
                                        .all(|term| carry_state_fragment_is_valid(term))
                                {
                                    return None;
                                }
                            }
                            Some((
                                factor_term,
                                scaled_by_payload,
                                factor_invariant_payload,
                                factor_scale_payload,
                                terms,
                                has_invariant,
                            ))
                        } else {
                            None
                        }
                    };
                    if let Some((
                        factor_term,
                        scaled_by_payload,
                        factor_invariant_payload,
                        factor_scale_payload,
                        terms,
                        has_invariant,
                    )) = parse_add_terms(kind)
                    {
                        let mut payload_index = 1usize;
                        let factor = if let Some(factor_terms) = factor_term {
                            let mut factor = resolve_state_term_for_async_post_flow(
                                &factor_terms[0],
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            for factor_term in factor_terms.iter().skip(1) {
                                let rhs = resolve_state_term_for_async_post_flow(
                                    factor_term,
                                    current,
                                    next_current,
                                    current_carries,
                                    next_carries,
                                    node_name,
                                    loop_instruction,
                                )?;
                                let sum = fresh_reg(next_reg);
                                body.push(format!("  {sum} = add i64 {factor}, {rhs}"));
                                factor = sum;
                            }
                            if factor_scale_payload {
                                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor scale payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let scaled = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {scaled} = mul i64 {factor}, {factor_scale_name}"
                                ));
                                factor = scaled;
                            }
                            if factor_invariant_payload {
                                let factor_offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor invariant payload for `{kind}` during LLVM lowering"))?;
                                payload_index += 1;
                                let sum = fresh_reg(next_reg);
                                body.push(format!(
                                    "  {sum} = add i64 {factor}, {factor_offset_name}"
                                ));
                                factor = sum;
                            }
                            Some(factor)
                        } else if scaled_by_payload {
                            let factor_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing scaled carry factor for `{kind}` during LLVM lowering"))?;
                            payload_index += 1;
                            Some(factor_name.clone())
                        } else {
                            None
                        };
                        let mut acc = resolve_state_term_for_async_post_flow(
                            &terms[0],
                            current,
                            next_current,
                            current_carries,
                            next_carries,
                            node_name,
                            loop_instruction,
                        )?;
                        for term in terms.iter().skip(1) {
                            let rhs = resolve_state_term_for_async_post_flow(
                                term,
                                current,
                                next_current,
                                current_carries,
                                next_carries,
                                node_name,
                                loop_instruction,
                            )?;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                            acc = sum;
                        }
                        if factor.is_some() && has_invariant {
                            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                            payload_index += 1;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                            acc = sum;
                        }
                        if let Some(factor) = factor {
                            let scaled_reg = fresh_reg(next_reg);
                            body.push(format!("  {scaled_reg} = mul i64 {acc}, {factor}"));
                            acc = scaled_reg;
                        } else if has_invariant {
                            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                            let sum = fresh_reg(next_reg);
                            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                            acc = sum;
                        }
                        return Ok(acc);
                    }
                    Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))
                }
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_source_spec, else_source_spec)) in
                    carry_specs.iter().enumerate()
                {
                    let then_value =
                        if matches!(then_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let src = resolve_source_for_async_post_flow(
                                then_source_spec,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &mut body,
                                &mut next_reg,
                                &node.name,
                                loop_instruction,
                            )?;
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        };
                    let else_value =
                        if matches!(else_source_spec[0].as_str(), "keep" | "keep_prev_carry") {
                            current_carries[index].clone()
                        } else {
                            let src = resolve_source_for_async_post_flow(
                                else_source_spec,
                                &current,
                                &next_current,
                                &current_carries,
                                &next_carries,
                                &mut body,
                                &mut next_reg,
                                &node.name,
                                loop_instruction,
                            )?;
                            let r = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {r} = add i64 {}, {}",
                                current_carries[index], src
                            ));
                            r
                        };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let rhs=cond_rhs.clone().ok_or_else(|| format!("cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering", node.name))?;
                        let (lhs,pred)=match cond_kind.as_str() {
                            "current_eq"=>(next_current.clone(),"eq"), "current_ne"=>(next_current.clone(),"ne"), "current_lt"=>(next_current.clone(),"slt"), "current_le"=>(next_current.clone(),"sle"), "current_gt"=>(next_current.clone(),"sgt"), "current_ge"=>(next_current.clone(),"sge"),
                            "prev_current_eq"=>(current.clone(),"eq"), "prev_current_ne"=>(current.clone(),"ne"), "prev_current_lt"=>(current.clone(),"slt"), "prev_current_le"=>(current.clone(),"sle"), "prev_current_gt"=>(current.clone(),"sgt"), "prev_current_ge"=>(current.clone(),"sge"),
                            other if other.starts_with("prev_carry") && other.ends_with("_eq") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ne") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("prev_carry") && other.ends_with("_lt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_le") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("prev_carry") && other.ends_with("_gt") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("prev_carry") && other.ends_with("_ge") => { let i=other[10..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other if other.starts_with("carry") && other.ends_with("_eq") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"eq") }
                            other if other.starts_with("carry") && other.ends_with("_ne") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"ne") }
                            other if other.starts_with("carry") && other.ends_with("_lt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"slt") }
                            other if other.starts_with("carry") && other.ends_with("_le") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sle") }
                            other if other.starts_with("carry") && other.ends_with("_gt") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sgt") }
                            other if other.starts_with("carry") && other.ends_with("_ge") => { let i=other[5..other.len()-3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name))?; (next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable conditional carry source `{other}` during LLVM lowering", node.name))?,"sge") }
                            other => return Err(format!("cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{other}` during LLVM lowering", node.name)),
                        };
                        let c = fresh_reg(&mut next_reg);
                        body.push(format!("  {c} = icmp {pred} i64 {lhs}, {rhs}"));
                        let s = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {s} = select i1 {c}, i64 {then_value}, i64 {else_value}"
                        ));
                        s
                    };
                    next_carries.push(next_carry);
                }
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let mut no_match_block = loop_continue.clone();
                for (index, (condition, action)) in flow_leaves.iter().enumerate().rev() {
                    let leaf_block = if index == 0 {
                        None
                    } else {
                        Some(fresh_block(&mut next_block, "loop_async_post_flow_rhs"))
                    };
                    if let Some(block) = &leaf_block {
                        body.push(format!("{block}:"));
                    }
                    let control_cond = eval_control_expr(
                        condition,
                        &next_current,
                        &next_carries,
                        &mut body,
                        &mut next_reg,
                        &node.name,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_async_post_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                        body.push(format!("  store i64 {val}, ptr {slot}"));
                    }
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                    if let Some(block) = leaf_block {
                        no_match_block = block;
                    }
                }
                body.push(format!("  br label %{no_match_block}"));
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (slot, val) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {val}, ptr {slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|slot| {
                        let r = fresh_reg(&mut next_reg);
                        body.push(format!("  {r} = load i64, ptr {slot}"));
                        r
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, fc) in final_carries.iter().enumerate() {
                    fields.push((format!("carry{index}"), LlvmValueRef::I64(fc.clone())));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            (
                "cpu",
                "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain",
            ) => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 5, &node.name, loop_instruction)?;
                let (Some(initial_value), Some(limit_value), Some(step_value)) =
                    (initial_value, limit_value, step_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(&chunk[2]).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(cond_rhs) =
                            coerce_to_i64(&cond_rhs_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    };
                    carry_initials.push(carry_initial);
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_continue =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_continue"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "eq" => "eq",
                    "ne" => "ne",
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let resolve_source = |kind: &str,
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<String, String> {
                    if matches!(kind, "keep" | "keep_prev_carry") {
                        return Ok(String::new());
                    }
                    if kind == "add_current" {
                        return Ok(next_current.clone());
                    }
                    if kind == "add_prev_current" {
                        return Ok(current.clone());
                    }
                    if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return next_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    Err(format!(
                        "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                        node.name,
                    ))
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let then_value = if matches!(then_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source(then_kind, &next_current, &next_carries)?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
                    };
                    let else_value = if matches!(else_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source(else_kind, &next_current, &next_carries)?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let lhs = if matches!(
                            cond_kind.as_str(),
                            "current_eq"
                                | "current_ne"
                                | "current_lt"
                                | "current_le"
                                | "current_gt"
                                | "current_ge"
                        ) {
                            next_current.clone()
                        } else if matches!(
                            cond_kind.as_str(),
                            "prev_current_eq"
                                | "prev_current_ne"
                                | "prev_current_lt"
                                | "prev_current_le"
                                | "prev_current_gt"
                                | "prev_current_ge"
                        ) {
                            current.clone()
                        } else if let Some(rest) = cond_kind.strip_prefix("prev_carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) = cond_kind.strip_prefix("carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let rhs = cond_rhs.clone().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let cond_pred = if cond_kind.ends_with("_eq") || cond_kind == "current_eq" {
                            "eq"
                        } else if cond_kind.ends_with("_ne") || cond_kind == "current_ne" {
                            "ne"
                        } else if cond_kind.ends_with("_lt") || cond_kind == "current_lt" {
                            "slt"
                        } else if cond_kind.ends_with("_le") || cond_kind == "current_le" {
                            "sle"
                        } else if cond_kind.ends_with("_gt") || cond_kind == "current_gt" {
                            "sgt"
                        } else {
                            "sge"
                        };
                        let cond_reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {cond_reg} = icmp {cond_pred} i64 {lhs}, {rhs}"));
                        let select_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {select_reg} = select i1 {cond_reg}, i64 {then_value}, i64 {else_value}"
                        ));
                        select_reg
                    };
                    next_carries.push(next_carry);
                }
                let resolve_control_operand =
                    |kind: &str, next_current: &String, next_carries: &Vec<String>| {
                        match kind {
                            "current_eq" => Ok((next_current.clone(), "eq")),
                            "current_ne" => Ok((next_current.clone(), "ne")),
                            "current_lt" => Ok((next_current.clone(), "slt")),
                            "current_le" => Ok((next_current.clone(), "sle")),
                            "current_gt" => Ok((next_current.clone(), "sgt")),
                            "current_ge" => Ok((next_current.clone(), "sge")),
                            other if other.starts_with("carry") && other.ends_with("_eq") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "eq"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_ne") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "ne"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_lt") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "slt"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_le") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "sle"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_gt") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "sgt"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_ge") => {
                                let i = other[5..other.len() - 3].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name))?;
                                Ok((next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering", node.name))?, "sge"))
                            }
                            other => Err(format!("cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering", node.name)),
                        }
                    };
                let eval_control_expr = |expr: &ResolvedLoopControlExpr,
                                         next_current: &String,
                                         next_carries: &Vec<String>,
                                         body: &mut Vec<String>,
                                         next_reg: &mut usize|
                 -> Result<String, String> {
                    fn eval(
                        expr: &ResolvedLoopControlExpr,
                        next_current: &String,
                        next_carries: &Vec<String>,
                        body: &mut Vec<String>,
                        next_reg: &mut usize,
                        resolve_control_operand: &impl Fn(
                            &str,
                            &String,
                            &Vec<String>,
                        )
                            -> Result<(String, &'static str), String>,
                    ) -> Result<String, String> {
                        match expr {
                            ResolvedLoopControlExpr::Cond { kind, rhs } => {
                                let (lhs, pred) =
                                    resolve_control_operand(kind, next_current, next_carries)?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                                Ok(reg)
                            }
                            ResolvedLoopControlExpr::And(lhs, rhs) => {
                                let lhs_reg = eval(
                                    lhs,
                                    next_current,
                                    next_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let rhs_reg = eval(
                                    rhs,
                                    next_current,
                                    next_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
                                Ok(reg)
                            }
                            ResolvedLoopControlExpr::Or(lhs, rhs) => {
                                let lhs_reg = eval(
                                    lhs,
                                    next_current,
                                    next_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let rhs_reg = eval(
                                    rhs,
                                    next_current,
                                    next_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
                                Ok(reg)
                            }
                        }
                    }
                    eval(
                        expr,
                        next_current,
                        next_carries,
                        body,
                        next_reg,
                        &resolve_control_operand,
                    )
                };
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let mut no_match_block = loop_continue.clone();
                for (index, (condition, action)) in flow_leaves.iter().enumerate().rev() {
                    let leaf_block = if index == 0 {
                        None
                    } else {
                        Some(fresh_block(&mut next_block, "loop_post_flow_rhs"))
                    };
                    if let Some(block) = &leaf_block {
                        body.push(format!("{block}:"));
                    }
                    let control_cond = eval_control_expr(
                        condition,
                        &next_current,
                        &next_carries,
                        &mut body,
                        &mut next_reg,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_post_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                        body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                    }
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported flow action `{other}` during LLVM lowering",
                                node.name,
                            ));
                        }
                    }
                    if let Some(block) = leaf_block {
                        no_match_block = block;
                    }
                }
                body.push(format!("  br label %{no_match_block}"));
                body.push(format!("{loop_continue}:"));
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain") => {
                let loop_instruction = canonical_loop_instruction(&node.op.instruction);
                let loop_block_prefix = canonical_loop_block_prefix(&node.op.instruction);
                let initial_value = registers.get(&node.op.args[0]).cloned();
                let limit_value = registers.get(&node.op.args[1]).cloned();
                let step_value = registers.get(&node.op.args[2]).cloned();
                let (flow_expr, carry_start_index) =
                    parse_loop_flow_expr_for_llvm(&node.op.args, 5, &node.name, loop_instruction)?;
                let Some(initial_value) = initial_value else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit_value) = limit_value else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(step_value) = step_value else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name,
                    ));
                    continue;
                };
                let Some(initial) = coerce_to_i64(&initial_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its initial value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(limit) = coerce_to_i64(&limit_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its limit value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(step) = coerce_to_i64(&step_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.{loop_instruction} `{}` because its step value is not coercible to i64",
                        node.name,
                    ));
                    continue;
                };
                let Some(resolved_flow_expr) = resolve_loop_flow_expr_for_llvm(
                    &flow_expr,
                    &registers,
                    &mut body,
                    &mut next_reg,
                    &node.name,
                    loop_instruction,
                ) else {
                    continue;
                };
                let cmp_kind = node.op.args[3].as_str();
                let step_kind = node.op.args[4].as_str();
                let mut carry_initials = Vec::new();
                let mut carry_specs = Vec::new();
                let mut deferred = false;
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let carry_initial_value = registers.get(&chunk[0]).cloned();
                    let Some(carry_initial_value) = carry_initial_value else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are outside the current CPU LLVM slice",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(carry_initial) =
                        coerce_to_i64(&carry_initial_value, &mut body, &mut next_reg)
                    else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more carry initials are not coercible to i64",
                            node.name,
                        ));
                        deferred = true;
                        break;
                    };
                    let cond_rhs = if chunk[1] == "always" {
                        None
                    } else {
                        let cond_rhs_value = registers.get(&chunk[2]).cloned();
                        let Some(cond_rhs_value) = cond_rhs_value else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are outside the current CPU LLVM slice",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        let Some(cond_rhs) =
                            coerce_to_i64(&cond_rhs_value, &mut body, &mut next_reg)
                        else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.{loop_instruction} `{}` because one or more condition rhs values are not coercible to i64",
                                node.name,
                            ));
                            deferred = true;
                            break;
                        };
                        Some(cond_rhs)
                    };
                    carry_initials.push(carry_initial);
                    carry_specs.push((
                        chunk[1].clone(),
                        cond_rhs,
                        chunk[3].clone(),
                        chunk[4].clone(),
                    ));
                }
                if deferred {
                    continue;
                }
                let current_slot = fresh_reg(&mut next_reg);
                body.push(format!("  {current_slot} = alloca i64"));
                body.push(format!("  store i64 {initial}, ptr {current_slot}"));
                let carry_slots = carry_initials
                    .iter()
                    .map(|carry_initial| {
                        let carry_slot = fresh_reg(&mut next_reg);
                        body.push(format!("  {carry_slot} = alloca i64"));
                        body.push(format!("  store i64 {carry_initial}, ptr {carry_slot}"));
                        carry_slot
                    })
                    .collect::<Vec<_>>();
                let loop_cond = fresh_block(&mut next_block, &format!("{loop_block_prefix}_cond"));
                let loop_body = fresh_block(&mut next_block, &format!("{loop_block_prefix}_body"));
                let loop_update =
                    fresh_block(&mut next_block, &format!("{loop_block_prefix}_update"));
                let loop_exit = fresh_block(&mut next_block, &format!("{loop_block_prefix}_exit"));
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_cond}:"));
                let current = fresh_reg(&mut next_reg);
                body.push(format!("  {current} = load i64, ptr {current_slot}"));
                let cmp = fresh_reg(&mut next_reg);
                let pred = match cmp_kind {
                    "lt" => "slt",
                    "le" => "sle",
                    "gt" => "sgt",
                    "ge" => "sge",
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported compare kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                body.push(format!("  {cmp} = icmp {pred} i64 {current}, {limit}"));
                body.push(format!(
                    "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
                ));
                body.push(format!("{loop_body}:"));
                let next_current = match step_kind {
                    "add" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = add i64 {current}, {step}"));
                        reg
                    }
                    "sub" => {
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {reg} = sub i64 {current}, {step}"));
                        reg
                    }
                    other => {
                        return Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported step kind `{other}` during LLVM lowering",
                            node.name,
                        ));
                    }
                };
                let mut current_carries = Vec::new();
                for carry_slot in &carry_slots {
                    let carry_before = fresh_reg(&mut next_reg);
                    body.push(format!("  {carry_before} = load i64, ptr {carry_slot}"));
                    current_carries.push(carry_before);
                }
                let resolve_control_operand =
                    |kind: &str, next_current: &String, current_carries: &Vec<String>| {
                        match kind {
                            "current_eq" => Ok((next_current.clone(), "eq")),
                            "current_ne" => Ok((next_current.clone(), "ne")),
                            "current_lt" => Ok((next_current.clone(), "slt")),
                            "current_le" => Ok((next_current.clone(), "sle")),
                            "current_gt" => Ok((next_current.clone(), "sgt")),
                            "current_ge" => Ok((next_current.clone(), "sge")),
                            other if other.starts_with("carry") && other.ends_with("_eq") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "eq"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_ne") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "ne"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_lt") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "slt"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_le") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "sle"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_gt") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "sgt"))
                            }
                            other if other.starts_with("carry") && other.ends_with("_ge") => {
                                let source_index = other[5..other.len() - 3].parse::<usize>().map_err(
                                    |_| {
                                        format!(
                                            "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                            node.name,
                                        )
                                    },
                                )?;
                                let lhs = current_carries.get(source_index).cloned().ok_or_else(|| {
                                    format!(
                                        "cpu.{loop_instruction} `{}` references unavailable control source `{other}` during LLVM lowering",
                                        node.name,
                                    )
                                })?;
                                Ok((lhs, "sge"))
                            }
                            other => Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported control kind `{other}` during LLVM lowering",
                                node.name,
                            )),
                        }
                    };
                let eval_control_expr = |expr: &ResolvedLoopControlExpr,
                                         next_current: &String,
                                         current_carries: &Vec<String>,
                                         body: &mut Vec<String>,
                                         next_reg: &mut usize|
                 -> Result<String, String> {
                    fn eval(
                        expr: &ResolvedLoopControlExpr,
                        next_current: &String,
                        current_carries: &Vec<String>,
                        body: &mut Vec<String>,
                        next_reg: &mut usize,
                        resolve_control_operand: &impl Fn(
                            &str,
                            &String,
                            &Vec<String>,
                        )
                            -> Result<(String, &'static str), String>,
                    ) -> Result<String, String> {
                        match expr {
                            ResolvedLoopControlExpr::Cond { kind, rhs } => {
                                let (lhs, pred) =
                                    resolve_control_operand(kind, next_current, current_carries)?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
                                Ok(reg)
                            }
                            ResolvedLoopControlExpr::And(lhs, rhs) => {
                                let lhs_reg = eval(
                                    lhs,
                                    next_current,
                                    current_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let rhs_reg = eval(
                                    rhs,
                                    next_current,
                                    current_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = and i1 {lhs_reg}, {rhs_reg}"));
                                Ok(reg)
                            }
                            ResolvedLoopControlExpr::Or(lhs, rhs) => {
                                let lhs_reg = eval(
                                    lhs,
                                    next_current,
                                    current_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let rhs_reg = eval(
                                    rhs,
                                    next_current,
                                    current_carries,
                                    body,
                                    next_reg,
                                    resolve_control_operand,
                                )?;
                                let reg = fresh_reg(next_reg);
                                body.push(format!("  {reg} = or i1 {lhs_reg}, {rhs_reg}"));
                                Ok(reg)
                            }
                        }
                    }
                    eval(
                        expr,
                        next_current,
                        current_carries,
                        body,
                        next_reg,
                        &resolve_control_operand,
                    )
                };
                let mut flow_leaves: Vec<(&ResolvedLoopControlExpr, &str)> = Vec::new();
                collect_resolved_loop_flow_leaves(&resolved_flow_expr, &mut flow_leaves);
                let mut no_match_block = loop_update.clone();
                for (index, (condition, action)) in flow_leaves.iter().enumerate().rev() {
                    let leaf_block = if index == 0 {
                        None
                    } else {
                        Some(fresh_block(&mut next_block, "loop_flow_rhs"))
                    };
                    if let Some(block) = &leaf_block {
                        body.push(format!("{block}:"));
                    }
                    let control_cond = eval_control_expr(
                        condition,
                        &next_current,
                        &current_carries,
                        &mut body,
                        &mut next_reg,
                    )?;
                    let action_block = fresh_block(&mut next_block, "loop_flow_action");
                    body.push(format!(
                        "  br i1 {control_cond}, label %{action_block}, label %{no_match_block}"
                    ));
                    body.push(format!("{action_block}:"));
                    body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                    match *action {
                        "break" => body.push(format!("  br label %{loop_exit}")),
                        "continue" => body.push(format!("  br label %{loop_cond}")),
                        other => {
                            return Err(format!(
                                "unsupported flow action `{other}` during LLVM lowering"
                            ));
                        }
                    }
                    if let Some(block) = leaf_block {
                        no_match_block = block;
                    }
                }
                body.push(format!("  br label %{no_match_block}"));
                body.push(format!("{loop_update}:"));
                let resolve_source = |kind: &str,
                                      next_current: &String,
                                      next_carries: &Vec<String>|
                 -> Result<String, String> {
                    if matches!(kind, "keep" | "keep_prev_carry") {
                        return Ok(String::new());
                    }
                    if kind == "add_current" {
                        return Ok(next_current.clone());
                    }
                    if kind == "add_prev_current" {
                        return Ok(current.clone());
                    }
                    if let Some(rest) = kind.strip_prefix("add_prev_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                            format!(
                                "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                node.name,
                            )
                        })?;
                        return current_carries.get(source_index).cloned().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                node.name,
                            )
                        });
                    }
                    if let Some(rest) = kind.strip_prefix("add_carry") {
                        let source_index = rest.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                        return next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable carry source `{kind}` during LLVM lowering",
                                    node.name,
                                )
                            });
                    }
                    Err(format!(
                            "cpu.{loop_instruction} `{}` has unsupported carry kind `{kind}` during LLVM lowering",
                            node.name,
                        ))
                };
                let mut next_carries = Vec::new();
                for (index, (cond_kind, cond_rhs, then_kind, else_kind)) in
                    carry_specs.iter().enumerate()
                {
                    let then_value = if matches!(then_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source(then_kind, &next_current, &next_carries)?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
                    };
                    let else_value = if matches!(else_kind.as_str(), "keep" | "keep_prev_carry") {
                        current_carries[index].clone()
                    } else {
                        let source = resolve_source(else_kind, &next_current, &next_carries)?;
                        let reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {reg} = add i64 {}, {}",
                            current_carries[index], source
                        ));
                        reg
                    };
                    let next_carry = if cond_kind == "always" {
                        then_value
                    } else {
                        let lhs = if matches!(
                            cond_kind.as_str(),
                            "current_eq"
                                | "current_ne"
                                | "current_lt"
                                | "current_le"
                                | "current_gt"
                                | "current_ge"
                        ) {
                            next_current.clone()
                        } else if matches!(
                            cond_kind.as_str(),
                            "prev_current_eq"
                                | "prev_current_ne"
                                | "prev_current_lt"
                                | "prev_current_le"
                                | "prev_current_gt"
                                | "prev_current_ge"
                        ) {
                            current.clone()
                        } else if let Some(rest) = cond_kind.strip_prefix("prev_carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            current_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else if let Some(rest) = cond_kind.strip_prefix("carry") {
                            let (index_text, suffix) = rest.split_once('_').ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            let source_index = index_text.parse::<usize>().map_err(|_| {
                                format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?;
                            if suffix != "eq"
                                && suffix != "ne"
                                && suffix != "lt"
                                && suffix != "le"
                                && suffix != "gt"
                                && suffix != "ge"
                            {
                                return Err(format!(
                                    "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                    node.name,
                                ));
                            }
                            next_carries.get(source_index).cloned().ok_or_else(|| {
                                format!(
                                    "cpu.{loop_instruction} `{}` references unavailable conditional carry source `{cond_kind}` during LLVM lowering",
                                    node.name,
                                )
                            })?
                        } else {
                            return Err(format!(
                                "cpu.{loop_instruction} `{}` has unsupported conditional carry kind `{cond_kind}` during LLVM lowering",
                                node.name,
                            ));
                        };
                        let rhs = cond_rhs.clone().ok_or_else(|| {
                            format!(
                                "cpu.{loop_instruction} `{}` is missing condition rhs during LLVM lowering",
                                node.name,
                            )
                        })?;
                        let cond_pred = if cond_kind.ends_with("_eq") || cond_kind == "current_eq" {
                            "eq"
                        } else if cond_kind.ends_with("_ne") || cond_kind == "current_ne" {
                            "ne"
                        } else if cond_kind.ends_with("_lt") || cond_kind == "current_lt" {
                            "slt"
                        } else if cond_kind.ends_with("_le") || cond_kind == "current_le" {
                            "sle"
                        } else if cond_kind.ends_with("_gt") || cond_kind == "current_gt" {
                            "sgt"
                        } else {
                            "sge"
                        };
                        let cond_reg = fresh_reg(&mut next_reg);
                        body.push(format!("  {cond_reg} = icmp {cond_pred} i64 {lhs}, {rhs}"));
                        let select_reg = fresh_reg(&mut next_reg);
                        body.push(format!(
                            "  {select_reg} = select i1 {cond_reg}, i64 {then_value}, i64 {else_value}"
                        ));
                        select_reg
                    };
                    next_carries.push(next_carry);
                }
                body.push(format!("  store i64 {next_current}, ptr {current_slot}"));
                for (carry_slot, next_carry) in carry_slots.iter().zip(next_carries.iter()) {
                    body.push(format!("  store i64 {next_carry}, ptr {carry_slot}"));
                }
                body.push(format!("  br label %{loop_cond}"));
                body.push(format!("{loop_exit}:"));
                let final_current = fresh_reg(&mut next_reg);
                body.push(format!("  {final_current} = load i64, ptr {current_slot}"));
                let final_carries = carry_slots
                    .iter()
                    .map(|carry_slot| {
                        let final_carry = fresh_reg(&mut next_reg);
                        body.push(format!("  {final_carry} = load i64, ptr {carry_slot}"));
                        final_carry
                    })
                    .collect::<Vec<_>>();
                let mut fields = vec![(
                    "current".to_owned(),
                    LlvmValueRef::I64(final_current.clone()),
                )];
                for (index, final_carry) in final_carries.iter().enumerate() {
                    fields.push((
                        format!("carry{index}"),
                        LlvmValueRef::I64(final_carry.clone()),
                    ));
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef {
                        type_name: "LoopChain".to_owned(),
                        fields,
                    }),
                );
                *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
            }
            ("cpu", "guard_return") => {
                let cond_value = registers.get(&node.op.args[0]).cloned();
                let return_value = registers.get(&node.op.args[1]).cloned();
                let (Some(cond_value), Some(return_value)) = (cond_value, return_value) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(cond) = coerce_to_i64(&cond_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let Some(returned) = coerce_to_i64(&return_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let cond_bool = fresh_reg(&mut next_reg);
                body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
                let then_label = fresh_block(&mut next_block, "guard_return_then");
                let cont_label = fresh_block(&mut next_block, "guard_return_cont");
                body.push(format!(
                    "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
                ));
                body.push(format!("{then_label}:"));
                match function_return_kind {
                    CpuCallScalarKind::Bool => {
                        let Some(returned_bool) = get_bool(&registers, &node.op.args[1]) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to bool",
                                node.name
                            ));
                            continue;
                        };
                        body.push(format!("  ret i1 {returned_bool}"));
                    }
                    CpuCallScalarKind::I32 => {
                        let Some(returned_i32) = get_i32(&registers, &node.op.args[1]) else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to i32",
                                node.name
                            ));
                            continue;
                        };
                        body.push(format!("  ret i32 {returned_i32}"));
                    }
                    CpuCallScalarKind::I64 => {
                        body.push(format!("  ret i64 {returned}"));
                    }
                    CpuCallScalarKind::F32 => {
                        if let Some(returned_f32) = get_f32(&registers, &node.op.args[1]) {
                            body.push(format!("  ret float {returned_f32}"));
                        } else if let Some(returned_f64) = get_f64(&registers, &node.op.args[1]) {
                            let as_f32 = fresh_reg(&mut next_reg);
                            body.push(format!(
                                "  {as_f32} = fptrunc double {returned_f64} to float"
                            ));
                            body.push(format!("  ret float {as_f32}"));
                        } else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to f32",
                                node.name
                            ));
                            continue;
                        }
                    }
                    CpuCallScalarKind::F64 => {
                        if let Some(returned_f64) = get_f64(&registers, &node.op.args[1]) {
                            body.push(format!("  ret double {returned_f64}"));
                        } else if let Some(returned_f32) = get_f32(&registers, &node.op.args[1]) {
                            let as_f64 = fresh_reg(&mut next_reg);
                            body.push(format!("  {as_f64} = fpext float {returned_f32} to double"));
                            body.push(format!("  ret double {as_f64}"));
                        } else {
                            body.push(format!(
                                "  ; deferred lowering for cpu.guard_return `{}` because its return value is not coercible to f64",
                                node.name
                            ));
                            continue;
                        }
                    }
                }
                body.push(format!("{cont_label}:"));
            }
            ("cpu", "guard_print") => {
                let cond_value = registers.get(&node.op.args[0]).cloned();
                let print_value = registers.get(&node.op.args[1]).cloned();
                let (Some(cond_value), Some(print_value)) = (cond_value, print_value) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(cond) = coerce_to_i64(&cond_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                let cond_bool = fresh_reg(&mut next_reg);
                body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
                let then_label = fresh_block(&mut next_block, "guard_print_then");
                let cont_label = fresh_block(&mut next_block, "guard_print_cont");
                body.push(format!(
                    "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
                ));
                body.push(format!("{then_label}:"));
                if let Some(input) = coerce_to_cstr(&print_value, &mut body, &mut next_reg) {
                    let print_reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
                } else if let Some(input) = coerce_to_i64(&print_value, &mut body, &mut next_reg) {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                } else {
                    body.push(format!(
                        "  ; deferred lowering inside cpu.guard_print `{}` because its print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
                }
                body.push(format!("  br label %{cont_label}"));
                body.push(format!("{cont_label}:"));
            }
            ("cpu", "guard_print_return") => {
                let cond_value = registers.get(&node.op.args[0]).cloned();
                let print_value = registers.get(&node.op.args[1]).cloned();
                let return_value = registers.get(&node.op.args[2]).cloned();
                let (Some(cond_value), Some(print_value), Some(return_value)) =
                    (cond_value, print_value, return_value)
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(cond) = coerce_to_i64(&cond_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                if !can_emit_typed_return_from_value(function_return_kind, &return_value) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                let cond_bool = fresh_reg(&mut next_reg);
                body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
                let then_label = fresh_block(&mut next_block, "guard_print_return_then");
                let cont_label = fresh_block(&mut next_block, "guard_print_return_cont");
                body.push(format!(
                    "  br i1 {cond_bool}, label %{then_label}, label %{cont_label}"
                ));
                body.push(format!("{then_label}:"));
                if let Some(input) = coerce_to_cstr(&print_value, &mut body, &mut next_reg) {
                    let print_reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
                } else if let Some(input) = coerce_to_i64(&print_value, &mut body, &mut next_reg) {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                } else {
                    body.push(format!(
                        "  ; deferred lowering inside cpu.guard_print_return `{}` because its print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
                }
                if !emit_typed_return_from_value(
                    &mut body,
                    &mut next_reg,
                    function_return_kind,
                    &return_value,
                ) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.guard_print_return `{}` because its return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                body.push(format!("{cont_label}:"));
            }
            ("cpu", "branch_print_return") => {
                let cond_value = registers.get(&node.op.args[0]).cloned();
                let then_print_value = registers.get(&node.op.args[1]).cloned();
                let then_return_value = registers.get(&node.op.args[2]).cloned();
                let else_print_value = registers.get(&node.op.args[3]).cloned();
                let else_return_value = registers.get(&node.op.args[4]).cloned();
                let (
                    Some(cond_value),
                    Some(then_print_value),
                    Some(then_return_value),
                    Some(else_print_value),
                    Some(else_return_value),
                ) = (
                    cond_value,
                    then_print_value,
                    then_return_value,
                    else_print_value,
                    else_return_value,
                )
                else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let Some(cond) = coerce_to_i64(&cond_value, &mut body, &mut next_reg) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its condition is not coercible to i64",
                        node.name
                    ));
                    continue;
                };
                if !can_emit_typed_return_from_value(function_return_kind, &then_return_value) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its then-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                if !can_emit_typed_return_from_value(function_return_kind, &else_return_value) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its else-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                let cond_bool = fresh_reg(&mut next_reg);
                body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));
                let then_label = fresh_block(&mut next_block, "branch_print_return_then");
                let else_label = fresh_block(&mut next_block, "branch_print_return_else");
                body.push(format!(
                    "  br i1 {cond_bool}, label %{then_label}, label %{else_label}"
                ));
                body.push(format!("{then_label}:"));
                if let Some(input) = coerce_to_cstr(&then_print_value, &mut body, &mut next_reg) {
                    let print_reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
                } else if let Some(input) =
                    coerce_to_i64(&then_print_value, &mut body, &mut next_reg)
                {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                } else {
                    body.push(format!(
                        "  ; deferred lowering inside cpu.branch_print_return `{}` because its then-print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
                }
                if !emit_typed_return_from_value(
                    &mut body,
                    &mut next_reg,
                    function_return_kind,
                    &then_return_value,
                ) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its then-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                body.push(format!("{else_label}:"));
                if let Some(input) = coerce_to_cstr(&else_print_value, &mut body, &mut next_reg) {
                    let print_reg = fresh_reg(&mut next_reg);
                    body.push(format!("  {print_reg} = call i32 @puts(ptr {input})"));
                } else if let Some(input) =
                    coerce_to_i64(&else_print_value, &mut body, &mut next_reg)
                {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                } else {
                    body.push(format!(
                        "  ; deferred lowering inside cpu.branch_print_return `{}` because its else-print value is not printable in the current CPU LLVM slice",
                        node.name
                    ));
                }
                if !emit_typed_return_from_value(
                    &mut body,
                    &mut next_reg,
                    function_return_kind,
                    &else_return_value,
                ) {
                    body.push(format!(
                        "  ; deferred lowering for cpu.branch_print_return `{}` because its else-return value is not coercible to {}",
                        node.name,
                        cpu_scalar_kind_llvm_type(function_return_kind)
                    ));
                    continue;
                }
                state.ends_with_terminal_return = true;
            }
            _ => {
                body.push(format!(
                    "  ; deferred lowering for {} on {} ({})",
                    node.op.full_name(),
                    node.resource,
                    resource.kind.raw
                ));
            }
        }
    }

    *global_counter = state.next_global;
    let ret = state.last_cpu_value.unwrap_or_else(|| "0".to_owned());
    let body = if state.ends_with_terminal_return {
        state.body.join("\n")
    } else {
        let mut body = state.body;
        emit_typed_return_from_last_value(
            &mut body,
            &mut state.next_reg,
            function_return_kind,
            &ret,
        );
        body.join("\n")
    };

    Ok(EmittedCpuFunction {
        globals: state.globals,
        body,
    })
}

pub fn emit_module(module: &YirModule) -> Result<String, String> {
    verify_module(module)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();
    let order = topological_order(module)?;
    let helper_lanes = module
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:"))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut global_counter = 0usize;
    let mut globals = Vec::new();
    let mut helper_defs = Vec::new();

    let mut helper_signatures = BTreeMap::<String, CpuHelperSignature>::new();
    for lane in &helper_lanes {
        let function_name = lane.trim_start_matches("fn:").to_owned();
        let mut params = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .filter_map(|node| {
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op
                    .instruction
                    .starts_with("param_")
                    .then_some((node, kind))
            })
            .map(|(node, kind)| {
                let index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "invalid {} index `{}`",
                        node.op.full_name(),
                        node.op.args[0]
                    )
                })?;
                Ok((index, node.name.clone(), kind))
            })
            .collect::<Result<Vec<_>, String>>()?;
        params.sort_by_key(|(index, _, _)| *index);
        let ret = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .find_map(|node| {
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op.instruction.starts_with("return_").then_some(kind)
            })
            .ok_or_else(|| {
                format!("helper lane `{function_name}` does not contain a typed cpu.return_*")
            })?;
        helper_signatures.insert(
            function_name,
            CpuHelperSignature {
                params: params.iter().map(|(_, _, kind)| *kind).collect(),
                ret,
            },
        );
    }

    for lane in &helper_lanes {
        let lane_nodes = order
            .iter()
            .filter(|name| module.node_lanes.get(*name) == Some(lane))
            .cloned()
            .collect::<Vec<_>>();
        let mut params = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .filter_map(|node| {
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op
                    .instruction
                    .starts_with("param_")
                    .then_some((node, kind))
            })
            .map(|(node, kind)| {
                let index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "invalid {} index `{}`",
                        node.op.full_name(),
                        node.op.args[0]
                    )
                })?;
                Ok((index, node.name.clone(), kind))
            })
            .collect::<Result<Vec<_>, String>>()?;
        params.sort_by_key(|(index, _, _)| *index);
        let param_bindings = params
            .iter()
            .map(|(index, name, kind)| (name.clone(), cpu_param_binding(*kind, *index)))
            .collect::<BTreeMap<_, _>>();
        let function_name = lane.trim_start_matches("fn:");
        let emitted = emit_cpu_function(
            module,
            &resources,
            &lane_nodes,
            &param_bindings,
            &helper_signatures,
            helper_signatures
                .get(function_name)
                .expect("helper signature should exist")
                .ret,
            &mut global_counter,
        )?;
        globals.extend(emitted.globals);
        let args_sig = params
            .iter()
            .map(|(index, _, kind)| format!("{} %arg{index}", cpu_scalar_kind_llvm_type(*kind)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_sig = cpu_scalar_kind_llvm_type(
            helper_signatures
                .get(function_name)
                .expect("helper signature should exist")
                .ret,
        );
        helper_defs.push(format!(
            "define {ret_sig} @nuis_fn_{function_name}({args_sig}) {{\n{}\n}}\n",
            emitted.body
        ));
    }

    let entry_nodes = order
        .iter()
        .filter(|name| {
            module
                .node_lanes
                .get(*name)
                .is_none_or(|lane| !lane.starts_with("fn:"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let entry_emitted = emit_cpu_function(
        module,
        &resources,
        &entry_nodes,
        &BTreeMap::new(),
        &helper_signatures,
        CpuCallScalarKind::I64,
        &mut global_counter,
    )?;
    globals.extend(entry_emitted.globals);

    let dynamic_extern_decls = render_dynamic_extern_decls(module);
    let dynamic_extern_block = if dynamic_extern_decls.is_empty() {
        String::new()
    } else {
        format!("{}\n\n", dynamic_extern_decls.join("\n"))
    };

    Ok(format!(
        "; yir version: {}\n\
{}\n\
%cpu.node = type {{ i64, ptr }}\n\
declare ptr @malloc(i64)\n\
declare void @free(ptr)\n\
declare i32 @puts(ptr)\n\
declare i64 @nuis_host_text_lift(ptr)\n\
declare ptr @nuis_host_text_ptr(i64)\n\
declare void @nuis_debug_print_bool(i32)\n\
declare void @nuis_debug_print_i32(i32)\n\
declare void @nuis_debug_print_i64(i64)\n\n\
declare void @nuis_debug_print_f32(float)\n\
declare void @nuis_debug_print_f64(double)\n\n\
declare i64 @host_color_bias(i64)\n\
declare i64 @host_speed_curve(i64)\n\
declare i64 @host_radius_curve(i64)\n\
declare i64 @host_mix_tick(i64, i64)\n\n\
declare i64 @HostRenderCurves__color_bias(i64)\n\
declare i64 @HostRenderCurves__speed_curve(i64)\n\
declare i64 @HostRenderCurves__radius_curve(i64)\n\
declare i64 @HostRenderCurves__mix_tick(i64, i64)\n\
declare i64 @HostMath__speed_curve(i64)\n\
{}\n\
{}\n\
define i64 @nuis_yir_entry() {{\n{}\n}}\n",
        module.version,
        globals.join("\n"),
        dynamic_extern_block,
        helper_defs.join("\n"),
        entry_emitted.body
    ))
}

fn render_dynamic_extern_decls(module: &YirModule) -> Vec<String> {
    let producer_types = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node_result_llvm_abi_type(node)))
        .collect::<BTreeMap<_, _>>();
    let mut declared = BTreeMap::<String, (&'static str, Vec<&'static str>)>::new();
    for node in &module.nodes {
        if node.op.module != "cpu" || !is_cpu_extern_call_instruction(&node.op.instruction) {
            continue;
        }
        if node.op.args.len() < 2 {
            continue;
        }
        let abi = node.op.args[0].as_str();
        if abi != "c" && abi != "nurs" {
            continue;
        }
        let symbol = node.op.args[1].clone();
        if is_builtin_host_ffi_symbol(&symbol) {
            continue;
        }
        let return_ty = cpu_extern_call_llvm_return_type(&node.op.instruction);
        let arg_types = node.op.args[2..]
            .iter()
            .map(|arg| producer_types.get(arg.as_str()).copied().unwrap_or("i64"))
            .collect::<Vec<_>>();
        declared.entry(symbol).or_insert((return_ty, arg_types));
    }
    declared
        .into_iter()
        .map(|(symbol, (return_ty, arg_types))| {
            let signature = if arg_types.is_empty() {
                String::new()
            } else {
                arg_types.join(", ")
            };
            format!("declare {return_ty} @{symbol}({signature})")
        })
        .collect()
}

fn get_i64<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I64(value)) => Some(value.as_str()),
        Some(LlvmValueRef::Bool { i64, .. }) => Some(i64.as_str()),
        Some(LlvmValueRef::TextHandle { handle, .. }) => Some(handle.as_str()),
        _ => None,
    }
}

fn get_i32<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I32(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_bool<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Bool { i1, .. }) => Some(i1.as_str()),
        _ => None,
    }
}

fn get_f32<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::F32(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_f64<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::F64(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_struct<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a StructLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Struct(value)) => Some(value),
        _ => None,
    }
}

fn get_network_result<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a NetworkResultLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::NetworkResult(result)) => Some(result),
        _ => None,
    }
}

fn get_task<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a TaskLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Task(task)) => Some(task),
        _ => None,
    }
}

fn get_thread<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a ThreadLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Thread(thread)) => Some(thread),
        _ => None,
    }
}

fn get_task_result<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a TaskResultLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::TaskResult(result)) => Some(result),
        _ => None,
    }
}

fn get_mutex<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a MutexLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::Mutex(mutex)) => Some(mutex),
        _ => None,
    }
}

fn get_mutex_guard<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a MutexGuardLlvmValueRef> {
    match registers.get(name) {
        Some(LlvmValueRef::MutexGuard(guard)) => Some(guard),
        _ => None,
    }
}

fn coerce_to_i64(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::I64(value) => Some(value.clone()),
        LlvmValueRef::TextHandle { handle, .. } => Some(handle.clone()),
        LlvmValueRef::Ptr(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = ptrtoint ptr {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::I32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sext i32 {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::Bool { i64, .. } => Some(i64.clone()),
        LlvmValueRef::F32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi float {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::F64(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi double {value} to i64"));
            Some(reg)
        }
        _ => None,
    }
}

fn coerce_to_i32(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::I32(value) => Some(value.clone()),
        LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = trunc i64 {value} to i32"));
            Some(reg)
        }
        LlvmValueRef::Bool { i1, .. } => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = zext i1 {i1} to i32"));
            Some(reg)
        }
        LlvmValueRef::F32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi float {value} to i32"));
            Some(reg)
        }
        LlvmValueRef::F64(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = fptosi double {value} to i32"));
            Some(reg)
        }
        _ => None,
    }
}

fn lower_dynamic_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::Ptr(value) | LlvmValueRef::TextHandle { ptr: value, .. } => {
            Some(format!("ptr {value}"))
        }
        LlvmValueRef::I32(_) => {
            coerce_to_i32(value, body, next_reg).map(|value| format!("i32 {value}"))
        }
        _ => coerce_to_i64(value, body, next_reg).map(|value| format!("i64 {value}")),
    }
}

fn lower_i64_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    coerce_to_i64(value, body, next_reg).map(|value| format!("i64 {value}"))
}

fn lower_i32_extern_arg(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    coerce_to_i32(value, body, next_reg).map(|value| format!("i32 {value}"))
}

fn render_extern_call(return_ty: &str, symbol: &str, lowered_args: &[String]) -> Option<String> {
    if lowered_args.len() > 6 {
        return None;
    }
    Some(format!(
        "call {return_ty} @{symbol}({})",
        lowered_args.join(", ")
    ))
}

fn is_cpu_extern_call_instruction(instruction: &str) -> bool {
    matches!(instruction, "extern_call_i64" | "extern_call_i32")
}

fn cpu_extern_call_llvm_return_type(instruction: &str) -> &'static str {
    match instruction {
        "extern_call_i32" => "i32",
        _ => "i64",
    }
}

fn is_builtin_host_ffi_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "malloc"
            | "free"
            | "puts"
            | "nuis_debug_print_bool"
            | "nuis_debug_print_i32"
            | "nuis_debug_print_i64"
            | "nuis_debug_print_f32"
            | "nuis_debug_print_f64"
            | "host_color_bias"
            | "host_speed_curve"
            | "host_radius_curve"
            | "host_mix_tick"
            | "HostRenderCurves__color_bias"
            | "HostRenderCurves__speed_curve"
            | "HostRenderCurves__radius_curve"
            | "HostRenderCurves__mix_tick"
            | "HostMath__speed_curve"
    )
}

fn node_result_llvm_abi_type(node: &Node) -> &'static str {
    if node.op.module != "cpu" {
        return "i64";
    }
    match node.op.instruction.as_str() {
        "text" | "null" | "borrow" | "move_ptr" | "alloc_node" | "alloc_buffer" | "load_next" => {
            "ptr"
        }
        "const_i32" | "cast_i64_to_i32" | "extern_call_i32" | "call_i32" | "param_i32" => "i32",
        _ => "i64",
    }
}

fn infer_loop_scalar_kind<'a, I>(values: I) -> Option<CpuLoopScalarKind>
where
    I: IntoIterator<Item = &'a LlvmValueRef>,
{
    let mut saw_f32 = false;
    for value in values {
        match value {
            LlvmValueRef::F64(_) => return Some(CpuLoopScalarKind::F64),
            LlvmValueRef::F32(_) => saw_f32 = true,
            LlvmValueRef::I64(_)
            | LlvmValueRef::I32(_)
            | LlvmValueRef::Bool { .. }
            | LlvmValueRef::TextHandle { .. } => {}
            _ => return None,
        }
    }
    if saw_f32 {
        Some(CpuLoopScalarKind::F32)
    } else {
        Some(CpuLoopScalarKind::I64)
    }
}

fn loop_scalar_llvm_type(kind: CpuLoopScalarKind) -> &'static str {
    match kind {
        CpuLoopScalarKind::I64 => "i64",
        CpuLoopScalarKind::F32 => "float",
        CpuLoopScalarKind::F64 => "double",
    }
}

fn loop_scalar_value_ref(kind: CpuLoopScalarKind, value: String) -> LlvmValueRef {
    match kind {
        CpuLoopScalarKind::I64 => LlvmValueRef::I64(value),
        CpuLoopScalarKind::F32 => LlvmValueRef::F32(value),
        CpuLoopScalarKind::F64 => LlvmValueRef::F64(value),
    }
}

fn coerce_to_loop_scalar(
    value: &LlvmValueRef,
    kind: CpuLoopScalarKind,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match kind {
        CpuLoopScalarKind::I64 => coerce_to_i64(value, body, next_reg),
        CpuLoopScalarKind::F32 => match value {
            LlvmValueRef::F32(value) => Some(value.clone()),
            LlvmValueRef::F64(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fptrunc double {value} to float"));
                Some(reg)
            }
            LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i64 {value} to float"));
                Some(reg)
            }
            LlvmValueRef::I32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i32 {value} to float"));
                Some(reg)
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let reg = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {reg} = sitofp i64 {as_i64} to float"));
                Some(reg)
            }
            _ => None,
        },
        CpuLoopScalarKind::F64 => match value {
            LlvmValueRef::F64(value) => Some(value.clone()),
            LlvmValueRef::F32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = fpext float {value} to double"));
                Some(reg)
            }
            LlvmValueRef::I64(value) | LlvmValueRef::TextHandle { handle: value, .. } => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i64 {value} to double"));
                Some(reg)
            }
            LlvmValueRef::I32(value) => {
                let reg = fresh_reg(next_reg);
                body.push(format!("  {reg} = sitofp i32 {value} to double"));
                Some(reg)
            }
            LlvmValueRef::Bool { i1, .. } => {
                let as_i64 = fresh_reg(next_reg);
                let reg = fresh_reg(next_reg);
                body.push(format!("  {as_i64} = zext i1 {i1} to i64"));
                body.push(format!("  {reg} = sitofp i64 {as_i64} to double"));
                Some(reg)
            }
            _ => None,
        },
    }
}

fn emit_loop_compare(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    kind: CpuLoopScalarKind,
    compare: &str,
    lhs: &str,
    rhs: &str,
) -> Result<String, String> {
    let reg = fresh_reg(next_reg);
    match kind {
        CpuLoopScalarKind::I64 => {
            let pred = match compare {
                "eq" => "eq",
                "ne" => "ne",
                "lt" => "slt",
                "le" => "sle",
                "gt" => "sgt",
                "ge" => "sge",
                other => return Err(format!("unsupported integer loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = icmp {pred} i64 {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F32 => {
            let pred = match compare {
                "eq" => "oeq",
                "ne" => "one",
                "lt" => "olt",
                "le" => "ole",
                "gt" => "ogt",
                "ge" => "oge",
                other => return Err(format!("unsupported float loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = fcmp {pred} float {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F64 => {
            let pred = match compare {
                "eq" => "oeq",
                "ne" => "one",
                "lt" => "olt",
                "le" => "ole",
                "gt" => "ogt",
                "ge" => "oge",
                other => return Err(format!("unsupported float loop compare kind `{other}`")),
            };
            body.push(format!("  {reg} = fcmp {pred} double {lhs}, {rhs}"));
        }
    }
    Ok(reg)
}

fn emit_loop_numeric_op(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    kind: CpuLoopScalarKind,
    op: &str,
    lhs: &str,
    rhs: &str,
) -> Result<String, String> {
    let reg = fresh_reg(next_reg);
    match kind {
        CpuLoopScalarKind::I64 => {
            let instr = match op {
                "add" => "add",
                "sub" => "sub",
                "mul" => "mul",
                "div" => "sdiv",
                other => return Err(format!("unsupported integer loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} i64 {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F32 => {
            let instr = match op {
                "add" => "fadd",
                "sub" => "fsub",
                "mul" => "fmul",
                "div" => "fdiv",
                other => return Err(format!("unsupported float loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} float {lhs}, {rhs}"));
        }
        CpuLoopScalarKind::F64 => {
            let instr = match op {
                "add" => "fadd",
                "sub" => "fsub",
                "mul" => "fmul",
                "div" => "fdiv",
                other => return Err(format!("unsupported float loop op `{other}`")),
            };
            body.push(format!("  {reg} = {instr} double {lhs}, {rhs}"));
        }
    }
    Ok(reg)
}

fn coerce_to_cstr<'a>(
    value: &'a LlvmValueRef,
    _body: &mut Vec<String>,
    _next_reg: &mut usize,
) -> Option<&'a str> {
    match value {
        LlvmValueRef::TextHandle { ptr, .. } => Some(ptr.as_str()),
        _ => None,
    }
}

fn get_ptr<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Ptr(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_cstr<'a>(registers: &'a BTreeMap<String, LlvmValueRef>, name: &str) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::TextHandle { ptr, .. }) => Some(ptr.as_str()),
        _ => None,
    }
}

fn fresh_reg(next: &mut usize) -> String {
    *next += 1;
    let reg = format!("%{}", *next);
    reg
}

fn fresh_global(next: &mut usize) -> String {
    let label = format!("@.str.{}", *next);
    *next += 1;
    label
}

fn fresh_block(next: &mut usize, prefix: &str) -> String {
    let label = format!("{prefix}.{}", *next);
    *next += 1;
    label
}

fn llvm_c_string_bytes(value: &str) -> (String, usize) {
    let mut out = String::new();
    let mut len = 0usize;
    for byte in value.as_bytes() {
        len += 1;
        match *byte {
            b'\\' => out.push_str("\\5C"),
            b'"' => out.push_str("\\22"),
            b'\n' => out.push_str("\\0A"),
            b'\r' => out.push_str("\\0D"),
            b'\t' => out.push_str("\\09"),
            0x20..=0x7E => out.push(*byte as char),
            other => out.push_str(&format!("\\{:02X}", other)),
        }
    }
    out.push_str("\\00");
    (out, len + 1)
}

fn lower_buffer_fill(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    ptr: &str,
    len: &str,
    fill: &str,
) -> Result<(), String> {
    let loop_cond = fresh_label(next_reg, "buf_fill_cond");
    let loop_body = fresh_label(next_reg, "buf_fill_body");
    let loop_exit = fresh_label(next_reg, "buf_fill_exit");
    let index_ptr = fresh_reg(next_reg);
    body.push(format!("  {index_ptr} = alloca i64"));
    body.push(format!("  store i64 0, ptr {index_ptr}"));
    body.push(format!("  br label %{loop_cond}"));
    body.push(format!("{loop_cond}:"));
    let index = fresh_reg(next_reg);
    body.push(format!("  {index} = load i64, ptr {index_ptr}"));
    let cmp = fresh_reg(next_reg);
    body.push(format!("  {cmp} = icmp slt i64 {index}, {len}"));
    body.push(format!(
        "  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"
    ));
    body.push(format!("{loop_body}:"));
    let slot = fresh_reg(next_reg);
    body.push(format!(
        "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"
    ));
    body.push(format!("  store i64 {fill}, ptr {slot}"));
    let next_index = fresh_reg(next_reg);
    body.push(format!("  {next_index} = add i64 {index}, 1"));
    body.push(format!("  store i64 {next_index}, ptr {index_ptr}"));
    body.push(format!("  br label %{loop_cond}"));
    body.push(format!("{loop_exit}:"));
    Ok(())
}

fn fresh_label(next: &mut usize, prefix: &str) -> String {
    *next += 1;
    format!("{prefix}_{}", *next)
}

fn topological_order(module: &YirModule) -> Result<Vec<String>, String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();
    let node_positions = module
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.name.clone(), index))
        .collect::<BTreeMap<_, _>>();

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep
            | EdgeKind::Effect
            | EdgeKind::Lifetime
            | EdgeKind::CrossDomainExchange => {
                adjacency
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                *indegree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_extern_on_resource = BTreeMap::<String, String>::new();
    for node in &module.nodes {
        if node.op.module == "cpu" && is_cpu_extern_call_instruction(&node.op.instruction) {
            if let Some(previous) =
                last_cpu_extern_on_resource.insert(node.resource.clone(), node.name.clone())
            {
                adjacency
                    .entry(previous)
                    .or_default()
                    .push(node.name.clone());
                *indegree.entry(node.name.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut last_cpu_node_on_lane = BTreeMap::<(String, String), String>::new();
    for node in &module.nodes {
        if node.op.module != "cpu" {
            continue;
        }
        let lane = module
            .node_lanes
            .get(&node.name)
            .cloned()
            .unwrap_or_else(|| "main".to_owned());
        if matches!(lane.as_str(), "profile" | "contract") {
            continue;
        }
        let key = (node.resource.clone(), lane);
        if let Some(previous) = last_cpu_node_on_lane.insert(key, node.name.clone()) {
            adjacency
                .entry(previous)
                .or_default()
                .push(node.name.clone());
            *indegree.entry(node.name.clone()).or_insert(0) += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort_by_key(|name| std::cmp::Reverse(node_positions[name]));

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort_by_key(|name| std::cmp::Reverse(node_positions[name]));
                    }
                }
            }
        }
    }

    if order.len() != module.nodes.len() {
        return Err("graph contains a cycle across YIR edges".to_owned());
    }

    Ok(order)
}

#[cfg(test)]
mod tests;
