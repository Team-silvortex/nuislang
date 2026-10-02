use super::*;

fn binding(name: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: None,
        value,
    }
}

fn var(name: &str) -> NirExpr {
    NirExpr::Var(name.into())
}

#[test]
fn continuation_reads_include_generated_outputs_and_ignore_past_reads() {
    let body = [
        binding("saved", var("index")),
        binding("index", NirExpr::Int(0)),
        NirStmt::Return(Some(var("saved"))),
    ];
    let mut plan = Plan::new(&body, Some(&BTreeSet::new())).unwrap();
    assert!(!plan.after(1).unwrap().contains("index"));
    let output = BTreeSet::from(["index".into()]);
    let mut plan = Plan::new(&body, Some(&output)).unwrap();
    assert!(plan.after(1).unwrap().contains("index"));
    assert!(Plan::new(&body, None).is_none());
}

#[test]
fn continuation_reads_keep_all_branches_headers_checked_operands_and_redefinitions() {
    let body = [
        binding("index", NirExpr::Int(0)),
        NirStmt::If {
            condition: NirExpr::Bool(false),
            then_body: vec![binding(
                "unused",
                NirExpr::Call {
                    callee: "observe".into(),
                    args: vec![var("index")],
                },
            )],
            else_body: vec![NirStmt::While {
                condition: var("header"),
                body: vec![
                    binding("index", NirExpr::Int(7)),
                    NirStmt::Return(Some(var("index"))),
                ],
            }],
        },
    ];
    let mut plan = Plan::new(&body, Some(&BTreeSet::new())).unwrap();
    assert_eq!(
        plan.after(0).unwrap(),
        BTreeSet::from(["index".into(), "header".into()])
    );
}

#[test]
fn continuation_reads_bound_scan_depth_and_suffix_materialization() {
    let body = [NirStmt::Return(Some(var("index")))];
    let mut plan = Plan::new(&body, Some(&BTreeSet::new())).unwrap();
    plan.remaining = 0;
    assert!(plan.after(0).is_none());
    assert!(plan.statement(&body[0], 0, 0).is_none());
    plan.remaining = MAX_WORK;
    assert!(plan.statement(&body[0], 0, MAX_DEPTH).is_none());
    let oversized = vec![binding("index", NirExpr::Int(0)); MAX_WORK];
    assert!(Plan::new(&oversized, Some(&BTreeSet::new())).is_none());
}

#[test]
fn continuation_reads_reject_unknown_syntax_without_partial_authority() {
    let body = [
        binding("saved", var("index")),
        NirStmt::Print(NirExpr::Int(0)),
    ];
    assert!(Plan::new(&body, Some(&BTreeSet::new())).is_none());
}
