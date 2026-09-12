use super::{CallbackExport, ScalarKind};

pub(super) fn callback(export: &CallbackExport, slots: usize) -> String {
    let count = export.arguments.len();
    let mut lines = vec![
        format!(
            "\ndefine i32 @{}(ptr %args, i64 %argc, ptr %out, i64 %outc) {{",
            export.symbol
        ),
        "entry:".to_owned(),
        format!("  %argc_ok = icmp eq i64 %argc, {count}"),
        format!("  %outc_ok = icmp eq i64 %outc, {slots}"),
        "  %counts_ok = and i1 %argc_ok, %outc_ok".to_owned(),
        "  %out_ok = icmp ne ptr %out, null".to_owned(),
        "  %shape_ok = and i1 %counts_ok, %out_ok".to_owned(),
    ];
    if count > 0 {
        lines.extend([
            "  %args_ok = icmp ne ptr %args, null".to_owned(),
            "  %pointers_ok = and i1 %shape_ok, %args_ok".to_owned(),
            "  br i1 %pointers_ok, label %inputs, label %bad_shape".to_owned(),
        ]);
    } else {
        lines.push("  br i1 %shape_ok, label %inputs, label %bad_shape".to_owned());
    }
    lines.extend([
        "bad_shape:".to_owned(),
        "  ret i32 1".to_owned(),
        "bad_scalar:".to_owned(),
        "  ret i32 2".to_owned(),
        "inputs:".to_owned(),
    ]);
    let mut parameters = Vec::new();
    for (index, kind) in export.arguments.iter().enumerate() {
        lines.push(format!(
            "  %input_ptr{index} = getelementptr i64, ptr %args, i64 {index}"
        ));
        lines.push(format!(
            "  %word{index} = load i64, ptr %input_ptr{index}, align 1"
        ));
        let canonical = match kind {
            ScalarKind::Bool => Some(format!("icmp ule i64 %word{index}, 1")),
            ScalarKind::I32 => {
                lines.push(format!("  %narrow{index} = trunc i64 %word{index} to i32"));
                lines.push(format!("  %wide{index} = sext i32 %narrow{index} to i64"));
                Some(format!("icmp eq i64 %word{index}, %wide{index}"))
            }
            ScalarKind::F32 => Some(format!("icmp ule i64 %word{index}, 4294967295")),
            _ => None,
        };
        if let Some(check) = canonical {
            lines.push(format!("  %canonical{index} = {check}"));
            lines.push(format!(
                "  br i1 %canonical{index}, label %arg{index}_ok, label %bad_scalar"
            ));
            lines.push(format!("arg{index}_ok:"));
        }
        let value = match kind {
            ScalarKind::I64 => format!("%word{index}"),
            ScalarKind::I32 => format!("%narrow{index}"),
            ScalarKind::Bool => {
                lines.push(format!("  %value{index} = trunc i64 %word{index} to i1"));
                format!("%value{index}")
            }
            ScalarKind::F32 => {
                lines.push(format!("  %bits{index} = trunc i64 %word{index} to i32"));
                lines.push(format!(
                    "  %value{index} = bitcast i32 %bits{index} to float"
                ));
                format!("%value{index}")
            }
            ScalarKind::F64 => {
                lines.push(format!(
                    "  %value{index} = bitcast i64 %word{index} to double"
                ));
                format!("%value{index}")
            }
        };
        let ty = match kind {
            ScalarKind::Bool => "i1",
            ScalarKind::I32 => "i32",
            ScalarKind::I64 => "i64",
            ScalarKind::F32 => "float",
            ScalarKind::F64 => "double",
        };
        parameters.push(format!("{ty} {value}"));
    }
    lines.push(format!(
        "  %returned = call i64 @nuis_fn_{}({})",
        export.function,
        parameters.join(", ")
    ));
    lines.push("  %aggregate = inttoptr i64 %returned to ptr".to_owned());
    lines.push("  call void @nuis_scheduler_owned_aggregate_require_v1(ptr %aggregate)".to_owned());
    for index in 0..slots {
        lines.push(format!("  %result{index} = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr %aggregate, i64 {index})"));
    }
    lines.push("  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)".to_owned());
    for index in 0..slots {
        lines.push(format!(
            "  %output_ptr{index} = getelementptr i64, ptr %out, i64 {index}"
        ));
        lines.push(format!(
            "  store i64 %result{index}, ptr %output_ptr{index}, align 1"
        ));
    }
    lines.extend(["  ret i32 0".to_owned(), "}\n".to_owned()]);
    lines.join("\n")
}
