use super::*;

fn function(body: &str) -> NirFunction {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
            struct Pair {{ x: i64, y: i64 }}
            fn relay(value: i64) -> i64 {{ return value; }}
            fn helper(state: Pair, flag: bool) -> i64 {{ {body} }}
        }}"
    ))
    .unwrap()
    .functions
    .into_iter()
    .find(|f| f.name == "helper")
    .unwrap()
}

fn binding(stmt: &NirStmt) -> (&str, &NirExpr) {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => (name, value),
        _ => panic!("expected binding"),
    }
}

#[test]
fn sibling_aliases_and_later_bindings_have_independent_identities() {
    let mut f = function(
        "if flag { let saved = state; relay(saved.x); }
        else { const saved: Pair = state; relay(saved.y); }
        let saved = state; return saved.x;",
    );
    normalize(&mut f);
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &f.body[0]
    else {
        panic!()
    };
    let left = binding(&then_body[0]).0;
    let right = binding(&else_body[0]).0;
    assert_ne!(left, right);
    assert_ne!(left, "saved");
    assert_ne!(right, "saved");
    assert_eq!(binding(&f.body[1]).0, "saved");
    for (body, name, field) in [(then_body, left, "x"), (else_body, right, "y")] {
        let NirStmt::Expr(NirExpr::Call { callee, args }) = &body[1] else {
            panic!()
        };
        assert_eq!(callee, "relay");
        assert_eq!(access(&args[0]), Some(vec![name.into(), field.into()]));
    }
}

#[test]
fn nested_and_loop_writes_keep_their_visible_binding_identity() {
    let mut f = function(
        "let outer = state;
        if flag {
            let local = state;
            if flag { let local = Pair { x: local.y, y: local.x }; }
            while flag { let local = Pair { x: local.y, y: local.x }; break; }
            let outer = local;
        }
        return outer.x;",
    );
    normalize(&mut f);
    let NirStmt::If { then_body, .. } = &f.body[1] else {
        panic!()
    };
    let local = binding(&then_body[0]).0;
    let NirStmt::If {
        then_body: inner, ..
    } = &then_body[1]
    else {
        panic!()
    };
    let NirStmt::While {
        body: loop_body, ..
    } = &then_body[2]
    else {
        panic!()
    };
    assert_eq!(binding(&inner[0]).0, local);
    assert_eq!(binding(&loop_body[0]).0, local);
    assert_eq!(
        binding(&then_body[3]),
        ("outer", &NirExpr::Var(local.into()))
    );
    assert_eq!(binding(&f.body[0]).0, "outer");
}

#[test]
fn private_names_reserve_later_bindings_and_do_not_rename_fields_or_callees() {
    let mut f = function(
        "if flag { let relay = state; return relay(relay.x); }
        let __nuis_capture_local_0 = 41; return __nuis_capture_local_0;",
    );
    normalize(&mut f);
    let NirStmt::If { then_body, .. } = &f.body[0] else {
        panic!()
    };
    let name = binding(&then_body[0]).0;
    assert_ne!(name, "__nuis_capture_local_0");
    let NirStmt::Return(Some(NirExpr::Call { callee, args })) = &then_body[1] else {
        panic!()
    };
    assert_eq!(callee, "relay");
    assert_eq!(access(&args[0]), Some(vec![name.into(), "x".into()]));
    assert_eq!(binding(&f.body[1]).0, "__nuis_capture_local_0");
}

#[test]
fn unsupported_expressions_leave_the_complete_candidate_unchanged() {
    let mut f = function("if flag { let saved = state; return saved.x; } return 0;");
    f.body
        .push(NirStmt::Return(Some(NirExpr::Text("unsupported".into()))));
    let before = f.body.clone();
    normalize(&mut f);
    assert_eq!(f.body, before);
}
