use std::collections::BTreeMap;

use yir_core::{EdgeKind, Resource, YirModule};
use yir_verify::verify_module;

#[derive(Clone)]
enum LlvmValueRef {
    Bool(String),
    I32(String),
    I64(String),
    F32(String),
    F64(String),
    Struct(StructLlvmValueRef),
    Ptr(String),
    CStr(String),
    Void,
}

#[derive(Clone)]
struct StructLlvmValueRef {
    type_name: String,
    fields: Vec<(String, LlvmValueRef)>,
}

pub fn emit_module(module: &YirModule) -> Result<String, String> {
    verify_module(module)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();

    let mut body = Vec::new();
    let mut globals = Vec::new();
    let mut registers = BTreeMap::<String, LlvmValueRef>::new();
    let mut buffer_lengths = BTreeMap::<String, String>::new();
    let mut next_reg = 0usize;
    let mut next_global = 0usize;
    let mut last_cpu_value = None::<String>;

    for node_name in topological_order(module)? {
        let node = module
            .nodes
            .iter()
            .find(|node| node.name == node_name)
            .ok_or_else(|| format!("lowering references unknown node `{node_name}`"))?;
        let resource = resources
            .get(&node.resource)
            .copied()
            .ok_or_else(|| format!("unknown resource `{}`", node.resource))?;

        if !resource.kind.is_family("cpu") {
            body.push(format!(
                "  ; deferred lowering for {} on {} ({})",
                node.op.full_name(),
                node.resource,
                resource.kind.raw
            ));
            continue;
        }

        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "text") => {
                let label = fresh_global(&mut next_global);
                let (bytes, len) = llvm_c_string_bytes(&node.op.args[0]);
                globals.push(format!(
                    "{label} = private unnamed_addr constant [{len} x i8] c\"{bytes}\""
                ));
                let ptr = fresh_reg(&mut next_reg);
                body.push(format!(
                    "  {ptr} = getelementptr inbounds [{len} x i8], ptr {label}, i64 0, i64 0"
                ));
                registers.insert(node.name.clone(), LlvmValueRef::CStr(ptr));
            }
            ("cpu", "const_bool") => {
                let value = match node.op.args[0].as_str() {
                    "true" => "true",
                    "false" => "false",
                    _ => {
                        body.push(format!(
                            "  ; deferred lowering for cpu.const_bool `{}` because literal `{}` is invalid",
                            node.name, node.op.args[0]
                        ));
                        continue;
                    }
                };
                registers.insert(node.name.clone(), LlvmValueRef::Bool(value.to_owned()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = zext i1 {value} to i64"));
                last_cpu_value = Some(widened);
            }
            ("cpu", "const_i32") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i32 0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), LlvmValueRef::I32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = sext i32 {reg} to i64"));
                last_cpu_value = Some(widened);
            }
            ("cpu", "const") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "const_i64") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "const_f32") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fadd float 0.0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), LlvmValueRef::F32(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi float {reg} to i64"));
                last_cpu_value = Some(widened);
            }
            ("cpu", "const_f64") => {
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = fadd double 0.0, {}", node.op.args[0]));
                registers.insert(node.name.clone(), LlvmValueRef::F64(reg.clone()));
                let widened = fresh_reg(&mut next_reg);
                body.push(format!("  {widened} = fptosi double {reg} to i64"));
                last_cpu_value = Some(widened);
            }
            ("cpu", "struct") => {
                let mut fields = Vec::new();
                let type_name = node.op.args[0].clone();
                let mut deferred = false;
                for entry in &node.op.args[1..] {
                    let Some((field_name, value_name)) = entry.split_once('=') else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.struct `{}` because field binding `{}` is invalid",
                            node.name, entry
                        ));
                        deferred = true;
                        break;
                    };
                    let Some(value_ref) = registers.get(value_name.trim()).cloned() else {
                        body.push(format!(
                            "  ; deferred lowering for cpu.struct `{}` because field `{}` comes from outside the current CPU LLVM slice",
                            node.name, field_name
                        ));
                        deferred = true;
                        break;
                    };
                    fields.push((field_name.trim().to_owned(), value_ref));
                }
                if deferred {
                    continue;
                }
                registers.insert(
                    node.name.clone(),
                    LlvmValueRef::Struct(StructLlvmValueRef { type_name, fields }),
                );
            }
            ("cpu", "field") => {
                let Some(struct_value) = get_struct(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.field `{}` because its source struct is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let field_name = &node.op.args[1];
                let Some((_, field_value)) = struct_value
                    .fields
                    .iter()
                    .find(|(name, _)| name == field_name) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.field `{}` because field `{}` does not exist on `{}`",
                        node.name, field_name, struct_value.type_name
                    ));
                    continue;
                };
                let field_value = field_value.clone();
                registers.insert(node.name.clone(), field_value.clone());
                if let Some(as_i64) = coerce_to_i64(&field_value, &mut body, &mut next_reg) {
                    last_cpu_value = Some(as_i64);
                }
            }
            ("cpu", "null") => {
                registers.insert(node.name.clone(), LlvmValueRef::Ptr("null".to_owned()));
            }
            ("cpu", "borrow") | ("cpu", "move_ptr") => {
                let Some(ptr) = get_ptr(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for {} `{}` because its input is outside the current CPU LLVM slice",
                        node.op.full_name(),
                        node.name
                    ));
                    continue;
                };
                registers.insert(node.name.clone(), LlvmValueRef::Ptr(ptr.to_owned()));
                if let Some(len) = buffer_lengths.get(&node.op.args[0]).cloned() {
                    buffer_lengths.insert(node.name.clone(), len);
                }
            }
            ("cpu", "neg") => {
                let Some(input) = get_i64(&registers, &node.op.args[0]) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.neg `{}` because its input is outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sub i64 0, {input}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
            }
            ("cpu", "add") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.add `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = add i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "eq") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.eq `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp eq i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "ne") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.ne `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp ne i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "lt") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.lt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp slt i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "gt") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.gt `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp sgt i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "le") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.le `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp sle i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "ge") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.ge `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let cmp = fresh_reg(&mut next_reg);
                body.push(format!("  {cmp} = icmp sge i64 {lhs}, {rhs}"));
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = zext i1 {cmp} to i64"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "sub") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.sub `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sub i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "mul") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.mul `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = mul i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "div") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.div `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = sdiv i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
            }
            ("cpu", "rem") => {
                let (Some(lhs), Some(rhs)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.rem `{}` because one or more inputs are outside the current CPU LLVM slice",
                        node.name
                    ));
                    continue;
                };
                let reg = fresh_reg(&mut next_reg);
                body.push(format!("  {reg} = srem i64 {lhs}, {rhs}"));
                registers.insert(node.name.clone(), LlvmValueRef::I64(reg.clone()));
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
            }
            ("cpu", "select") => {
                let (Some(cond), Some(then_value), Some(else_value)) = (
                    get_i64(&registers, &node.op.args[0]),
                    get_i64(&registers, &node.op.args[1]),
                    get_i64(&registers, &node.op.args[2]),
                ) else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.select `{}` because one or more inputs are outside the current CPU LLVM slice",
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
                last_cpu_value = Some(reg);
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
                lower_buffer_fill(&mut body, &mut next_reg, raw.as_str(), len.as_str(), fill.as_str())?;
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(len);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
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
                last_cpu_value = Some(reg);
            }
            ("cpu", "print") => {
                if let Some(input) = get_i64(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_i64(i64 {input})"));
                    last_cpu_value = Some(input.to_owned());
                } else if let Some(input) = get_i32(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_i32(i32 {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = sext i32 {input} to i64"));
                    last_cpu_value = Some(widened);
                } else if let Some(input) = get_bool(&registers, &node.op.args[0]) {
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = zext i1 {input} to i32"));
                    body.push(format!("  call void @nuis_debug_print_bool(i32 {widened})"));
                    let widened64 = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened64} = zext i1 {input} to i64"));
                    last_cpu_value = Some(widened64);
                } else if let Some(input) = get_f32(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_f32(float {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi float {input} to i64"));
                    last_cpu_value = Some(widened);
                } else if let Some(input) = get_f64(&registers, &node.op.args[0]) {
                    body.push(format!("  call void @nuis_debug_print_f64(double {input})"));
                    let widened = fresh_reg(&mut next_reg);
                    body.push(format!("  {widened} = fptosi double {input} to i64"));
                    last_cpu_value = Some(widened);
                } else if let Some(input) = get_cstr(&registers, &node.op.args[0]) {
                    body.push(format!("  %print_str_{next_reg} = call i32 @puts(ptr {input})"));
                    last_cpu_value = Some("0".to_owned());
                } else {
                    body.push(format!(
                        "  ; deferred lowering for cpu.print `{}` because its input is produced outside the current CPU LLVM slice",
                        node.op.args[0]
                    ));
                }
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

    let ret = last_cpu_value.unwrap_or_else(|| "0".to_owned());

    Ok(format!(
        "; yir version: {}\n\
{}\n\
%cpu.node = type {{ i64, ptr }}\n\
declare ptr @malloc(i64)\n\
declare void @free(ptr)\n\
declare i32 @puts(ptr)\n\
declare void @nuis_debug_print_bool(i32)\n\
declare void @nuis_debug_print_i32(i32)\n\
declare void @nuis_debug_print_i64(i64)\n\n\
declare void @nuis_debug_print_f32(float)\n\
declare void @nuis_debug_print_f64(double)\n\n\
define i64 @nuis_yir_entry() {{\n{}\n  ret i64 {}\n}}\n",
        module.version,
        globals.join("\n"),
        body.join("\n"),
        ret
    ))
}

fn get_i64<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I64(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_i32<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::I32(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_bool<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Bool(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_f32<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::F32(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_f64<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
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

fn coerce_to_i64(
    value: &LlvmValueRef,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) -> Option<String> {
    match value {
        LlvmValueRef::I64(value) => Some(value.clone()),
        LlvmValueRef::I32(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = sext i32 {value} to i64"));
            Some(reg)
        }
        LlvmValueRef::Bool(value) => {
            let reg = fresh_reg(next_reg);
            body.push(format!("  {reg} = zext i1 {value} to i64"));
            Some(reg)
        }
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

fn get_ptr<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::Ptr(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn get_cstr<'a>(
    registers: &'a BTreeMap<String, LlvmValueRef>,
    name: &str,
) -> Option<&'a str> {
    match registers.get(name) {
        Some(LlvmValueRef::CStr(value)) => Some(value.as_str()),
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
    body.push(format!("  br i1 {cmp}, label %{loop_body}, label %{loop_exit}"));
    body.push(format!("{loop_body}:"));
    let slot = fresh_reg(next_reg);
    body.push(format!("  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index}"));
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

    for node in &module.nodes {
        adjacency.entry(node.name.clone()).or_default();
        indegree.entry(node.name.clone()).or_insert(0);
    }

    for edge in &module.edges {
        match edge.kind {
            EdgeKind::Dep | EdgeKind::Effect | EdgeKind::Lifetime | EdgeKind::CrossDomainExchange => {
                adjacency
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                *indegree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(name, degree)| (*degree == 0).then_some(name.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut order = Vec::with_capacity(module.nodes.len());
    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(targets) = adjacency.get(&node) {
            for target in targets {
                if let Some(degree) = indegree.get_mut(target) {
                    *degree -= 1;
                    if *degree == 0 {
                        ready.push(target.clone());
                        ready.sort();
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
