use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use yir_core::{Value, YirModule};
use yir_lower_llvm::native_session::{emit_registered, NativeSessionBridge, ScalarKind};
use yir_runtime_host::ApplicationSession;

#[path = "native_application_bridge/bounds.rs"]
mod bounds;
#[path = "native_application_bridge/driver.rs"]
mod driver;
#[path = "native_application_bridge/dynamic_loop_guard.rs"]
mod dynamic_loop_guard;
#[path = "native_application_bridge/dynamic_loops.rs"]
mod dynamic_loops;
#[path = "native_application_bridge/helpers.rs"]
mod helpers;
#[path = "native_application_bridge/loops.rs"]
mod loops;

const SOURCE: &str = include_str!("native_application_bridge/main.ns");
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        Self::with_source(SOURCE)
    }

    fn with_source(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "nuis-native-session-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.ns"), source).unwrap();
        fs::write(path.join("nuis.toml"), "name = \"native_session\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n").unwrap();
        Self(path)
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn state_words(state: &Value) -> Vec<u64> {
    match state {
        Value::Struct(value) => value
            .fields
            .iter()
            .flat_map(|(_, value)| state_words(value))
            .collect(),
        Value::Bool(value) => vec![u64::from(*value)],
        Value::I32(value) => vec![*value as i64 as u64],
        Value::Int(value) => vec![*value as u64],
        Value::F32(value) => vec![u64::from(value.to_bits())],
        Value::F64(value) => vec![value.to_bits()],
        _ => panic!("unexpected resource state"),
    }
}

fn reference(module: &YirModule, gain: u32, scale: u64) -> Vec<Vec<u64>> {
    let registry = yir_verify::default_registry();
    let (mut session, trace) = ApplicationSession::open_registered(
        module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(f32::from_bits(gain)),
            Value::F64(f64::from_bits(scale)),
        ],
    )
    .unwrap();
    assert!(
        trace
            .events
            .iter()
            .all(|event| !event.contains("cpu.print")),
        "must not replay main"
    );
    let mut expected = vec![state_words(session.state())];
    for (delta, paused) in [(3, false), (100, true), (-2, false)] {
        session
            .event(vec![Value::Int(delta), Value::Bool(paused)])
            .unwrap();
        expected.push(state_words(session.state()));
    }
    session.close(vec![Value::Int(5)]).unwrap();
    session.completion_status().unwrap();
    expected.push(state_words(session.state()));
    assert_eq!(expected[0][..4], [0, 10, -17_i64 as u64, 1]);
    assert_eq!(expected[4][..4], [2, 16, -17_i64 as u64, 0]);
    expected
}

#[test]
fn registered_scalar_callbacks_execute_natively_without_main_replay_or_interpreter() {
    assert_native_parity(SOURCE, false);
}

fn assert_native_parity(source: &str, helper_calls: bool) {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for reversed in [false, true] {
        let mut module = compiled.yir.clone();
        if reversed {
            module.nodes.reverse();
            module.functions.reverse();
            for function in &mut module.functions {
                function.body_nodes.reverse();
            }
        }
        let bridge = emit_registered(&module, "counter").unwrap();
        if helper_calls {
            for ty in ["i1", "i32", "i64", "float", "double"] {
                assert!(
                    bridge.llvm_ir.contains(&format!(" = call {ty} @nuis_fn_")),
                    "missing {ty} call; nodes={:?}",
                    module
                        .nodes
                        .iter()
                        .filter(|n| n.op.instruction.starts_with("call_"))
                        .collect::<Vec<_>>()
                );
            }
            let roots = bridge
                .callbacks
                .iter()
                .map(|c| c.function.as_str())
                .collect::<Vec<_>>();
            assert!(
                module.nodes.iter().any(|node| {
                    node.op.instruction == "call_i64"
                        && module.node_lanes.get(&node.name).is_some_and(|lane| {
                            lane.strip_prefix("fn:")
                                .is_some_and(|name| !roots.contains(&name))
                        })
                }),
                "fixture must retain a nested helper call after optimization"
            );
        }
        assert_eq!(
            bridge.state_fields,
            vec![
                ("frame".to_owned(), ScalarKind::I64),
                ("metrics.total".to_owned(), ScalarKind::I64),
                ("metrics.tag".to_owned(), ScalarKind::I32),
                ("metrics.active".to_owned(), ScalarKind::Bool),
                ("metrics.gain".to_owned(), ScalarKind::F32),
                ("metrics.scale".to_owned(), ScalarKind::F64),
            ]
        );
        assert!(
            !bridge.llvm_ir.contains("deferred lowering"),
            "{}",
            bridge
                .llvm_ir
                .lines()
                .filter(|line| line.contains("deferred lowering"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        // Bit-preserving transport includes finite values, signed zero and NaN payloads.
        for (case, gain, scale) in [
            (0, 1.5_f32.to_bits(), (-2.25_f64).to_bits()),
            (1, 0x8000_0000, 0x8000_0000_0000_0000),
            (2, 0x7fc0_1234, 0x7ff8_0000_0000_4321),
        ] {
            let expected = reference(&module, gain, scale);
            let (llvm, output) = driver::build(&bridge, gain, scale, &expected);
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("out-{reversed}-{case}")),
                source,
                nuisc::aot::AotCompileProgram {
                    ast: &compiled.ast,
                    nir: &compiled.nir,
                    yir: &module,
                    llvm_ir: Some(&llvm),
                },
                &nuisc::aot::host_cpu_build_target(),
            )
            .unwrap();
            let run = Command::new(artifact.binary_path).output().unwrap();
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap() as u64)
                .collect::<Vec<_>>();
            assert_eq!(actual, output, "native registered callback ABI mismatch");
        }
    }
}

#[test]
fn scalar_words_reject_noncanonical_encodings_without_losing_float_bits() {
    for (kind, word) in [
        (ScalarKind::Bool, 2),
        (ScalarKind::I32, u32::MAX as u64),
        (ScalarKind::F32, 1_u64 << 32),
    ] {
        assert!(kind.unpack(word).is_err());
    }
    for (kind, word) in [
        (ScalarKind::Bool, 1),
        (ScalarKind::I32, -17_i64 as u64),
        (ScalarKind::I64, u64::MAX),
        (ScalarKind::F32, 0x7fc0_1234),
        (ScalarKind::F64, 0x7ff8_0000_0000_4321),
    ] {
        assert_eq!(kind.pack(&kind.unpack(word).unwrap()).unwrap(), word);
    }
    assert!(ScalarKind::I64.pack(&Value::Bool(true)).is_err());
}

#[test]
fn native_bridge_rejects_signature_layout_lane_and_initialization_drift() {
    let project = Project::new();
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    assert!(emit_registered(&module, "not_registered").is_err());
    let mut renamed = module.clone();
    renamed.application_sessions[0].id = "counter-with-dots.v2".to_owned();
    let old = emit_registered(&module, "counter").unwrap();
    let new = emit_registered(&renamed, "counter-with-dots.v2").unwrap();
    assert!(old
        .callbacks
        .iter()
        .zip(&new.callbacks)
        .all(|(a, b)| a.function == b.function && a.symbol != b.symbol));
    let event = yir_core::registered_application_session(&module, "counter")
        .unwrap()
        .event
        .clone();
    let function = module
        .functions
        .iter()
        .find(|function| function.name == event)
        .unwrap();
    let parameter = function.parameters[0].node.clone();
    let result = function.result.as_ref().unwrap().node.clone();
    let mut partial = module.clone();
    let broken = "unmaterialized_field".to_owned();
    let resource = partial
        .nodes
        .iter()
        .find(|node| node.name == parameter)
        .unwrap()
        .resource
        .clone();
    partial.nodes.push(yir_core::Node {
        name: broken.clone(),
        resource,
        op: yir_core::Operation::parse(
            "cpu.field",
            vec![parameter.clone(), "not_a_struct".to_owned()],
        )
        .unwrap(),
    });
    partial
        .node_lanes
        .insert(broken.clone(), format!("fn:{event}"));
    partial
        .functions
        .iter_mut()
        .find(|function| function.name == event)
        .unwrap()
        .body_nodes
        .push(broken.clone());
    for (from, to) in [
        (parameter.clone(), broken.clone()),
        (broken, result.clone()),
    ] {
        partial.edges.push(yir_core::Edge {
            kind: yir_core::EdgeKind::Dep,
            from,
            to,
        });
    }
    yir_verify::verify_module(&partial).unwrap();
    assert!(emit_registered(&partial, "counter")
        .unwrap_err()
        .contains("did not materialize value"));
    for mutation in [
        "index",
        "lane",
        "layout",
        "resource-state",
        "slot-order",
        "global-input",
    ] {
        let mut drift = module.clone();
        match mutation {
            "index" => {
                drift
                    .nodes
                    .iter_mut()
                    .find(|node| node.name == parameter)
                    .unwrap()
                    .op
                    .args[0] = "999".to_owned()
            }
            "lane" => {
                drift
                    .node_lanes
                    .insert(parameter.clone(), "fn:unrelated".to_owned());
            }
            "layout" => {
                let node = drift
                    .nodes
                    .iter_mut()
                    .find(|node| node.name == result)
                    .unwrap();
                node.op.args[1] = node.op.args[1].replace("gain:f32", "gain:f64");
            }
            "resource-state" | "slot-order" => {
                for function in &drift.functions {
                    if let Some(result) = &function.result {
                        let node = drift
                            .nodes
                            .iter_mut()
                            .find(|node| node.name == result.node)
                            .unwrap();
                        if node.op.instruction == "return_owned_struct" {
                            node.op.args[1] = if mutation == "resource-state" {
                                node.op.args[1].replace("gain:f32", "gain:Bytes")
                            } else {
                                node.op.args[1]
                                    .replace("tag:i32;active:bool", "active:bool;tag:i32")
                            };
                        }
                    }
                }
            }
            "global-input" => {
                let entry = drift
                    .functions
                    .iter()
                    .find(|function| function.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                drift.edges.push(yir_core::Edge {
                    kind: yir_core::EdgeKind::Effect,
                    from: entry.result.as_ref().unwrap().node.clone(),
                    to: parameter.clone(),
                });
            }
            _ => unreachable!(),
        }
        assert_ne!(
            drift, module,
            "fault injection {mutation} must actually mutate input"
        );
        assert!(
            emit_registered(&drift, "counter").is_err(),
            "accepted {mutation}"
        );
    }
    fs::write(
        project.0.join("main.ns"),
        SOURCE.replace("if paused", "print(delta); if paused"),
    )
    .unwrap();
    let effectful = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&effectful.yir, "counter").unwrap_err();
    assert!(error.contains("does not admit cpu.print"), "{error}");
}
