use nuis_semantics::model::{AstExpr, NirExpr};

use super::super::{i64_type, lower_expr};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_form_control_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NovaBuiltinInput { callee, args, .. } = input;
    let expr = match callee {
        "nova_text_input_packet" => lower_text_input_packet(args, input)?,
        "nova_select_packet" => lower_select_packet(args, input)?,
        "nova_checkbox_packet" => lower_checkbox_packet(args, input)?,
        "nova_radio_packet" => lower_radio_packet(args, input)?,
        "nova_textarea_packet" => lower_textarea_packet(args, input)?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn lower_i64_arg(expr: &AstExpr, input: NovaBuiltinInput<'_>) -> Result<NirExpr, String> {
    lower_expr(
        expr,
        input.current_domain,
        input.bindings,
        input.signatures,
        input.struct_table,
        Some(&i64_type()),
    )
}

fn lower_optional_i64_arg(
    expr: Option<&AstExpr>,
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    expr.map(|expr| lower_i64_arg(expr, input)).transpose()
}

fn lower_text_input_packet(
    args: &[AstExpr],
    input: NovaBuiltinInput<'_>,
) -> Result<NirExpr, String> {
    let (echo, caret, placeholder, read_only, dirty) = match args {
        [echo, caret] => (echo, caret, None, None, None),
        [echo, caret, placeholder] => (echo, caret, Some(placeholder), None, None),
        [echo, caret, placeholder, read_only] => {
            (echo, caret, Some(placeholder), Some(read_only), None)
        }
        [echo, caret, placeholder, read_only, dirty] => {
            (echo, caret, Some(placeholder), Some(read_only), Some(dirty))
        }
        _ => {
            return Err("nova_text_input_packet(...) expects 2, 3, 4 or 5 args".to_owned());
        }
    };
    let echo = lower_i64_arg(echo, input)?;
    let caret = lower_i64_arg(caret, input)?;
    let placeholder = lower_optional_i64_arg(placeholder, input)?.unwrap_or_else(|| echo.clone());
    let read_only = lower_optional_i64_arg(read_only, input)?.unwrap_or(NirExpr::Int(0));
    let dirty = lower_optional_i64_arg(dirty, input)?.unwrap_or(NirExpr::Int(0));
    Ok(NirExpr::StructLiteral {
        type_name: "NovaTextInputPacket".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("echo".to_owned(), echo),
            ("caret".to_owned(), caret),
            ("placeholder".to_owned(), placeholder),
            ("read_only".to_owned(), read_only),
            ("dirty".to_owned(), dirty),
        ],
    })
}

fn lower_select_packet(args: &[AstExpr], input: NovaBuiltinInput<'_>) -> Result<NirExpr, String> {
    let (selected, accent, options, multiple, committed) = match args {
        [selected, accent] => (selected, accent, None, None, None),
        [selected, accent, options] => (selected, accent, Some(options), None, None),
        [selected, accent, options, multiple] => {
            (selected, accent, Some(options), Some(multiple), None)
        }
        [selected, accent, options, multiple, committed] => (
            selected,
            accent,
            Some(options),
            Some(multiple),
            Some(committed),
        ),
        _ => return Err("nova_select_packet(...) expects 2, 3, 4 or 5 args".to_owned()),
    };
    let selected = lower_i64_arg(selected, input)?;
    let accent = lower_i64_arg(accent, input)?;
    let options = lower_optional_i64_arg(options, input)?.unwrap_or(NirExpr::Int(3));
    let multiple = lower_optional_i64_arg(multiple, input)?.unwrap_or(NirExpr::Int(0));
    let committed = lower_optional_i64_arg(committed, input)?.unwrap_or(NirExpr::Int(1));
    Ok(NirExpr::StructLiteral {
        type_name: "NovaSelectPacket".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("selected".to_owned(), selected),
            ("accent".to_owned(), accent),
            ("options".to_owned(), options),
            ("multiple".to_owned(), multiple),
            ("committed".to_owned(), committed),
        ],
    })
}

fn lower_checkbox_packet(args: &[AstExpr], input: NovaBuiltinInput<'_>) -> Result<NirExpr, String> {
    let (checked, accent, disabled) = match args {
        [checked, accent] => (checked, accent, None),
        [checked, accent, disabled] => (checked, accent, Some(disabled)),
        _ => return Err("nova_checkbox_packet(...) expects 2 or 3 args".to_owned()),
    };
    let checked = lower_i64_arg(checked, input)?;
    let accent = lower_i64_arg(accent, input)?;
    let disabled = lower_optional_i64_arg(disabled, input)?.unwrap_or(NirExpr::Int(0));
    Ok(NirExpr::StructLiteral {
        type_name: "NovaCheckboxPacket".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("checked".to_owned(), checked),
            ("accent".to_owned(), accent),
            ("disabled".to_owned(), disabled),
        ],
    })
}

fn lower_radio_packet(args: &[AstExpr], input: NovaBuiltinInput<'_>) -> Result<NirExpr, String> {
    let (selected, options, accent, disabled) = match args {
        [selected, options, accent] => (selected, options, accent, None),
        [selected, options, accent, disabled] => (selected, options, accent, Some(disabled)),
        _ => return Err("nova_radio_packet(...) expects 3 or 4 args".to_owned()),
    };
    let selected = lower_i64_arg(selected, input)?;
    let options = lower_i64_arg(options, input)?;
    let accent = lower_i64_arg(accent, input)?;
    let disabled = lower_optional_i64_arg(disabled, input)?.unwrap_or(NirExpr::Int(0));
    Ok(NirExpr::StructLiteral {
        type_name: "NovaRadioPacket".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("selected".to_owned(), selected),
            ("options".to_owned(), options),
            ("accent".to_owned(), accent),
            ("disabled".to_owned(), disabled),
        ],
    })
}

fn lower_textarea_packet(args: &[AstExpr], input: NovaBuiltinInput<'_>) -> Result<NirExpr, String> {
    let (lines, scroll, placeholder, read_only, dirty) = match args {
        [lines, scroll] => (lines, scroll, None, None, None),
        [lines, scroll, placeholder] => (lines, scroll, Some(placeholder), None, None),
        [lines, scroll, placeholder, read_only] => {
            (lines, scroll, Some(placeholder), Some(read_only), None)
        }
        [lines, scroll, placeholder, read_only, dirty] => (
            lines,
            scroll,
            Some(placeholder),
            Some(read_only),
            Some(dirty),
        ),
        _ => return Err("nova_textarea_packet(...) expects 2, 3, 4 or 5 args".to_owned()),
    };
    let lines = lower_i64_arg(lines, input)?;
    let scroll = lower_i64_arg(scroll, input)?;
    let placeholder = lower_optional_i64_arg(placeholder, input)?.unwrap_or_else(|| lines.clone());
    let read_only = lower_optional_i64_arg(read_only, input)?.unwrap_or(NirExpr::Int(0));
    let dirty = lower_optional_i64_arg(dirty, input)?.unwrap_or(NirExpr::Int(0));
    Ok(NirExpr::StructLiteral {
        type_name: "NovaTextAreaPacket".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("lines".to_owned(), lines),
            ("scroll".to_owned(), scroll),
            ("placeholder".to_owned(), placeholder),
            ("read_only".to_owned(), read_only),
            ("dirty".to_owned(), dirty),
        ],
    })
}
