pub(crate) fn fresh_reg(next: &mut usize) -> String {
    *next += 1;
    let reg = format!("%{}", *next);
    reg
}

pub(crate) fn fresh_global(next: &mut usize) -> String {
    let label = format!("@.str.{}", *next);
    *next += 1;
    label
}

pub(crate) fn fresh_block(next: &mut usize, prefix: &str) -> String {
    let label = format!("{prefix}.{}", *next);
    *next += 1;
    label
}

pub(crate) fn llvm_c_string_bytes(value: &str) -> (String, usize) {
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

pub(crate) fn lower_buffer_fill(
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

pub(crate) fn fresh_label(next: &mut usize, prefix: &str) -> String {
    *next += 1;
    format!("{prefix}_{}", *next)
}
