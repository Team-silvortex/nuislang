use super::*;

#[test]
fn ast_projection_decodes_structure_and_round_trips_without_source() {
    let source = concat!(
        "/// 模块文档\n",
        "use text Text\n",
        "ast mod cpu unit Main\n",
        "  /// 入口\n",
        "  fn main() -> i64\n",
        "    if true\n",
        "      then return 7\n",
        "      else return 9\n",
    );
    let projection = parse_compiler_structural_projection(CompilerProjectionKind::Ast, source)
        .expect("parse AST projection");

    assert_eq!(projection.module_domain, "cpu");
    assert_eq!(projection.module_unit, "Main");
    assert_eq!(projection.records.len(), 8);
    assert_eq!(
        projection.records[0].kind,
        CompilerProjectionRecordKind::ModuleDocumentation
    );
    assert_eq!(
        projection.records[5].kind,
        CompilerProjectionRecordKind::Member
    );
    assert_eq!(
        projection.records[6].kind,
        CompilerProjectionRecordKind::Nested
    );
    assert_eq!(render_compiler_structural_projection(&projection), source);
}

#[test]
fn nir_projection_decodes_structure_and_verifies_identity() {
    let source = concat!(
        "use kernel KernelUnit\n",
        "nir mod cpu unit Worker\n",
        "  fn run() -> i64\n",
        "    let result: i64 = kernel_dispatch(7)\n",
        "    return result\n",
    );
    let projection = parse_compiler_structural_projection(CompilerProjectionKind::Nir, source)
        .expect("parse NIR projection");

    verify_compiler_projection_identity(&projection, "cpu", "Worker")
        .expect("identity should match");
    let error = verify_compiler_projection_identity(&projection, "cpu", "Other")
        .expect_err("identity mismatch must fail");
    assert!(error.to_string().contains("does not match handoff module"));
    assert_eq!(render_compiler_structural_projection(&projection), source);
}

#[test]
fn malformed_projection_structure_fails_closed() {
    for (source, expected) in [
        ("ast mod cpu unit Main", "end with LF"),
        ("ast mod cpu unit Main\n   fn main()\n", "two-space levels"),
        (
            "ast mod cpu unit Main\n      return 7\n",
            "increase by at most one",
        ),
        (
            "ast mod cpu unit Main\n  fn main() \n",
            "trailing whitespace",
        ),
        (
            "ast mod cpu unit Main\nuse text Text\n",
            "structurally indented",
        ),
    ] {
        let error = parse_compiler_structural_projection(CompilerProjectionKind::Ast, source)
            .expect_err("malformed projection must fail");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn stage_specific_headers_and_documentation_fail_closed() {
    let error = parse_compiler_structural_projection(
        CompilerProjectionKind::Ast,
        "nir mod cpu unit Main\n",
    )
    .expect_err("wrong header must fail");
    assert!(error.to_string().contains("module header"));

    let error = parse_compiler_structural_projection(
        CompilerProjectionKind::Nir,
        "nir mod cpu unit Main\n  /// invalid\n",
    )
    .expect_err("NIR documentation must fail");
    assert!(error.to_string().contains("not part of the NIR"));

    let error = parse_compiler_structural_projection(
        CompilerProjectionKind::Ast,
        "use  Text\nast mod cpu unit Main\n",
    )
    .expect_err("noncanonical import must fail");
    assert!(error.to_string().contains("import domain"));
}

#[test]
fn nir_projection_round_trips_opaque_multiline_wgsl_leaf() {
    let source = concat!(
        "nir mod shader unit Surface\n",
        "  fn build() -> i64\n",
        "    let module: ShaderModule = shader_inline_wgsl(\"demo\", wgsl {\n",
        "  @compute @workgroup_size(1)\n",
        "  fn main() {\n",
        "    return;\n",
        "  \n",
        "  }\n",
        "})\n",
        "    return 7\n",
    );
    let projection = parse_compiler_structural_projection(CompilerProjectionKind::Nir, source)
        .expect("parse opaque WGSL leaf");

    assert!(projection
        .records
        .iter()
        .any(|record| record.kind == CompilerProjectionRecordKind::OpaqueBody));
    assert!(projection
        .records
        .iter()
        .any(|record| record.kind == CompilerProjectionRecordKind::OpaqueTerminator));
    assert_eq!(render_compiler_structural_projection(&projection), source);
}

#[test]
fn malformed_opaque_wgsl_leaf_fails_closed() {
    let unterminated = concat!(
        "nir mod shader unit Surface\n",
        "  fn build()\n",
        "    let module = shader_inline_wgsl(\"demo\", wgsl {\n",
        "  @compute\n",
    );
    let error = parse_compiler_structural_projection(CompilerProjectionKind::Nir, unterminated)
        .expect_err("unterminated opaque leaf must fail");
    assert!(error.to_string().contains("unterminated opaque WGSL"));

    let unframed = concat!(
        "nir mod shader unit Surface\n",
        "  fn build()\n",
        "    let module = shader_inline_wgsl(\"demo\", wgsl {\n",
        "unframed body\n",
        "})\n",
    );
    let error = parse_compiler_structural_projection(CompilerProjectionKind::Nir, unframed)
        .expect_err("unframed opaque leaf must fail");
    assert!(error.to_string().contains("two-space framing"));
}
