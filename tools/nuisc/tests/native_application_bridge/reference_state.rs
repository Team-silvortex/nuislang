use super::*;
use yir_core::ApplicationFailureKind;
use yir_runtime_host::ApplicationSessionPhase;

const SOURCE: &str = r#"
mod cpu Main {
    struct Inner { x: i64, y: i64 }
    struct State { inner: Inner, tail: i64 }
    @noinline fn stamp(value: i64) -> i64 { print(value); return value; }
    fn start() -> State {
        return State { tail: stamp(30), inner: Inner { y: stamp(20), x: stamp(10) } };
    }
    fn step(state: State) -> State {
        return State {
            tail: stamp(state.tail + 1),
            inner: Inner { y: stamp(state.inner.y + 1), x: stamp(state.inner.x + 1) }
        };
    }
    fn stop(state: State) -> State {
        return State {
            tail: stamp(state.tail + 1),
            inner: Inner { y: stamp(state.inner.y + 1), x: stamp(state.inner.x + 1) }
        };
    }
    fn main() { print(999); }
}
"#;

fn module() -> YirModule {
    nuisc::pipeline::compile_project(&Project::with_source(SOURCE).0)
        .unwrap()
        .yir
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|event| event.starts_with("effect cpu.print "))
        .map(|event| event.rsplit_once(": ").unwrap().1.parse().unwrap())
        .collect()
}

fn record_node<'a>(
    module: &'a mut YirModule,
    callback: &str,
    nominal: &str,
) -> &'a mut yir_core::Node {
    let result = &module
        .functions
        .iter()
        .find(|f| f.name == callback)
        .unwrap()
        .result
        .as_ref()
        .unwrap()
        .node;
    let root = &module
        .nodes
        .iter()
        .find(|n| &n.name == result)
        .unwrap()
        .op
        .args[0];
    let name = if nominal == "State" {
        root.clone()
    } else {
        module
            .nodes
            .iter()
            .find(|n| &n.name == root)
            .unwrap()
            .op
            .args
            .iter()
            .find_map(|arg| arg.strip_prefix("inner="))
            .unwrap()
            .to_owned()
    };
    let node = module.nodes.iter_mut().find(|n| n.name == name).unwrap();
    assert_eq!(node.op.instruction, "struct");
    assert_eq!(node.op.args[0], nominal);
    node
}

#[test]
fn reference_state_normalization_preserves_source_effect_order_for_every_callback() {
    let registry = yir_verify::default_registry();
    for source in [SOURCE.to_owned(), SOURCE.replace("@noinline ", "")] {
        let project = Project::with_source(&source);
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        for reversed in [false, true] {
            let mut module = module.clone();
            if reversed {
                module.nodes.reverse();
                module.functions.reverse();
                for function in &mut module.functions {
                    function.body_nodes.reverse();
                }
            }
            let (mut session, opened) =
                ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
            assert_eq!(printed(&opened.events), [30, 20, 10]);
            assert_eq!(state_words(session.state()), [10, 20, 30]);
            let event = session.event(vec![]).unwrap();
            assert_eq!(printed(&event.events), [31, 21, 11]);
            assert_eq!(state_words(session.state()), [11, 21, 31]);
            let closed = session.close(vec![]).unwrap().unwrap();
            assert_eq!(printed(&closed.events), [32, 22, 12]);
            assert_eq!(state_words(session.state()), [12, 22, 32]);
            assert!(session.close(vec![]).unwrap().is_none());
            session.completion_status().unwrap();
        }
    }
}

#[test]
fn field_effect_order_includes_nested_call_arguments() {
    let registry = yir_verify::default_registry();
    for inline in [false, true] {
        let mut source = SOURCE.to_owned();
        if inline {
            source = source.replace("@noinline ", "");
        }
        for expr in [
            "30",
            "20",
            "10",
            "state.tail + 1",
            "state.inner.y + 1",
            "state.inner.x + 1",
        ] {
            source = source.replace(&format!("stamp({expr})"), &format!("stamp(stamp({expr}))"));
        }
        let project = Project::with_source(&source);
        let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        module.nodes.reverse();
        for function in &mut module.functions {
            function.body_nodes.reverse();
        }
        let (mut session, opened) =
            ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
        assert_eq!(printed(&opened.events), [30, 30, 20, 20, 10, 10]);
        assert_eq!(state_words(session.state()), [10, 20, 30]);
        assert_eq!(
            printed(&session.event(vec![]).unwrap().events),
            [31, 31, 21, 21, 11, 11]
        );
        assert_eq!(
            printed(&session.close(vec![]).unwrap().unwrap().events),
            [32, 32, 22, 22, 12, 12]
        );
        session.completion_status().unwrap();
    }
}

#[test]
fn reference_state_rejects_callback_layout_drift_before_open() {
    let registry = yir_verify::default_registry();
    let module = module();
    for callback in ["start", "step", "stop"] {
        for replacement in [
            "State{inner:Other{x:i64;y:i64};tail:i64}",
            "State{tail:i64;inner:Inner{x:i64;y:i64}}",
            "State{inner:Inner{x:i32;y:i64};tail:i64}",
        ] {
            let mut module = module.clone();
            let result = module
                .functions
                .iter()
                .find(|f| f.name == callback)
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .node
                .clone();
            let returned = module.nodes.iter_mut().find(|n| n.name == result).unwrap();
            assert_eq!(returned.op.instruction, "return_owned_struct");
            returned.op.args[1] = replacement.to_owned();
            let error = ApplicationSession::open_registered(&module, &registry, "counter", vec![])
                .err()
                .expect("accepted layout drift");
            assert!(error.contains("application session state"), "{error}");
        }
    }
}

#[test]
fn malformed_event_state_keeps_last_accepted_state_and_allows_one_cleanup() {
    let module = module();
    let registry = yir_verify::default_registry();
    for case in 0..6 {
        let mut module = module.clone();
        let inner = record_node(&mut module, "step", "Inner");
        match case {
            0 => inner.op.args[0] = "OtherInner".to_owned(),
            1 => {
                let (_, value) = inner.op.args[2].split_once('=').unwrap();
                inner.op.args[2] = format!("y={value}");
            }
            2 => {
                inner.op.args.pop();
            }
            3 => {
                let (_, value) = inner.op.args[2].split_once('=').unwrap();
                inner.op.args.push(format!("extra={value}"));
            }
            4 => record_node(&mut module, "step", "State").op.args[0] = "OtherState".to_owned(),
            5 => {
                let outer = record_node(&mut module, "step", "State");
                let (_, value) = outer.op.args[1].split_once('=').unwrap();
                outer.op.args[2] = format!("inner={value}");
            }
            _ => unreachable!(),
        }
        let (mut session, _) =
            ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
        let accepted = session.state().clone();
        let error = session.event(vec![]).unwrap_err();
        assert!(error.contains("session"), "case {case}: {error}");
        assert_eq!(session.state(), &accepted);
        assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
        assert_eq!(session.failure_kind(), ApplicationFailureKind::Callback);
        assert!(session.event(vec![]).is_err());
        let closed = session.close(vec![]).unwrap().unwrap();
        assert_eq!(printed(&closed.events), [31, 21, 11]);
        assert_eq!(state_words(session.state()), [11, 21, 31]);
        assert!(session.close(vec![]).unwrap().is_none());
        assert!(session.completion_status().is_err());
    }
}

#[test]
fn malformed_close_state_is_terminal_and_does_not_publish_partial_state() {
    let mut module = module();
    record_node(&mut module, "stop", "Inner").op.args[0] = "Wrong".to_owned();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
    session.event(vec![]).unwrap();
    let accepted = session.state().clone();
    let error = session.close(vec![]).unwrap_err();
    assert!(error.contains("wrong state type"), "{error}");
    assert_eq!(session.state(), &accepted);
    assert_eq!(session.phase(), ApplicationSessionPhase::Closed);
    assert_eq!(session.failure_kind(), ApplicationFailureKind::Callback);
    assert_eq!(session.close(vec![]).unwrap_err(), error);
    assert!(session.completion_status().is_err());
}

#[test]
fn state_normalization_does_not_retry_fuel_exhaustion() {
    let module = module();
    let registry = yir_verify::default_registry();
    let (mut session, _) =
        ApplicationSession::open_registered(&module, &registry, "counter", vec![]).unwrap();
    let accepted = session.state().clone();
    assert!(session.event_budgeted(vec![], 1).is_err());
    assert_eq!(session.state(), &accepted);
    assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
    assert!(session.event(vec![]).is_err());
    assert_eq!(
        printed(&session.close(vec![]).unwrap().unwrap().events),
        [31, 21, 11]
    );
    assert!(session.completion_status().is_err());
}
