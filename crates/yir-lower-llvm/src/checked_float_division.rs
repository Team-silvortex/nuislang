use super::fresh_reg;

pub(crate) fn check(rhs: &str, ty: &str, body: &mut Vec<String>, next_reg: &mut usize) {
    let zero = fresh_reg(next_reg);
    // Register-derived labels share the function's unique name supply without
    // changing loop emitters' block counters or introducing a runtime helper.
    let tag = zero.trim_start_matches('%');
    let invalid = format!("float_divisor_invalid_{tag}");
    let ready = format!("float_divisor_ready_{tag}");
    body.push(format!("  {zero} = fcmp oeq {ty} {rhs}, 0.000000e+00"));
    body.push(format!("  br i1 {zero}, label %{invalid}, label %{ready}"));
    body.push(format!("{invalid}:"));
    body.push("  call void @llvm.trap()".to_owned());
    body.push("  unreachable".to_owned());
    body.push(format!("{ready}:"));
}
