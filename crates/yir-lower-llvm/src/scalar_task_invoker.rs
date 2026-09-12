use super::*;

pub(super) fn render_scalar_task_invoker(
    function_name: &str,
    signature: &CpuHelperSignature,
) -> Option<String> {
    if !is_normalized_task_scalar(signature.ret)
        || signature
            .params
            .iter()
            .any(|kind| !is_normalized_task_scalar(*kind))
    {
        return None;
    }
    let mut body = Vec::new();
    let mut call_args = Vec::new();
    for (index, kind) in signature.params.iter().copied().enumerate() {
        let pointer = if index == 0 {
            "%context".to_owned()
        } else {
            let pointer = format!("%task_arg{index}_ptr");
            body.push(format!(
                "  {pointer} = getelementptr i8, ptr %context, i64 {}",
                index * 8
            ));
            pointer
        };
        let packed = if kind == CpuCallScalarKind::I64 {
            format!("%task_arg{index}")
        } else {
            format!("%task_arg{index}_packed")
        };
        body.push(format!("  {packed} = load i64, ptr {pointer}, align 8"));
        let argument = match kind {
            CpuCallScalarKind::Bool => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = trunc i64 {packed} to i1"));
                argument
            }
            CpuCallScalarKind::I32 => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = trunc i64 {packed} to i32"));
                argument
            }
            CpuCallScalarKind::I64 => packed,
            CpuCallScalarKind::F32 => {
                let bits = format!("%task_arg{index}_bits");
                body.push(format!("  {bits} = trunc i64 {packed} to i32"));
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = bitcast i32 {bits} to float"));
                argument
            }
            CpuCallScalarKind::F64 => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = bitcast i64 {packed} to double"));
                argument
            }
            CpuCallScalarKind::BorrowedBuffer => {
                unreachable!("borrowed buffers do not have task invokers")
            }
            CpuCallScalarKind::TraversalPointer => {
                unreachable!("traversal pointers do not have task invokers")
            }
            CpuCallScalarKind::OwnedBytes => {
                unreachable!("direct owned Bytes params do not have scalar task invokers")
            }
            CpuCallScalarKind::OwnedExternalBuffer => {
                unreachable!("owned external buffers do not have task invokers")
            }
        };
        call_args.push(format!("{} {argument}", cpu_scalar_kind_llvm_type(kind)));
    }
    let return_ty = cpu_scalar_kind_llvm_type(signature.ret);
    body.push(format!(
        "  %task_result = call {return_ty} @nuis_fn_{function_name}({})",
        call_args.join(", ")
    ));
    match signature.ret {
        CpuCallScalarKind::Bool => {
            body.push("  %task_result_packed = zext i1 %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::I32 => {
            body.push("  %task_result_packed = sext i32 %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::I64 => body.push("  ret i64 %task_result".to_owned()),
        CpuCallScalarKind::F32 => {
            body.push("  %task_result_bits = bitcast float %task_result to i32".to_owned());
            body.push("  %task_result_packed = zext i32 %task_result_bits to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::F64 => {
            body.push("  %task_result_packed = bitcast double %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::BorrowedBuffer => {
            unreachable!("borrowed buffers cannot return from task invokers")
        }
        CpuCallScalarKind::TraversalPointer => {
            unreachable!("traversal pointers cannot return from task invokers")
        }
        CpuCallScalarKind::OwnedBytes => {
            unreachable!("owned Bytes cannot return from scalar task invokers")
        }
        CpuCallScalarKind::OwnedExternalBuffer => {
            unreachable!("owned external buffers cannot return from scalar task invokers")
        }
    }
    Some(format!(
        "define i64 @nuis_task_invoker_{function_name}(ptr %context) {{\n{}\n}}\n",
        body.join("\n")
    ))
}

fn is_normalized_task_scalar(kind: CpuCallScalarKind) -> bool {
    matches!(
        kind,
        CpuCallScalarKind::Bool
            | CpuCallScalarKind::I32
            | CpuCallScalarKind::I64
            | CpuCallScalarKind::F32
            | CpuCallScalarKind::F64
    )
}
