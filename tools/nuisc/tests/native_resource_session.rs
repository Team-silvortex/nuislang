const SOURCE: &str = include_str!("native_resource_session/main.ns");
#[path = "native_resource_session/boolean_captures.rs"]
mod boolean_captures;
#[cfg(unix)]
#[path = "native_resource_session/boolean_negated_short_circuit.rs"]
mod boolean_negated_short_circuit;
#[path = "native_resource_session/boolean_parameters.rs"]
mod boolean_parameters;
#[path = "native_resource_session/boolean_records.rs"]
mod boolean_records;
#[path = "native_resource_session/boolean_shadowing.rs"]
mod boolean_shadowing;
#[path = "native_resource_session/boolean_short_circuit.rs"]
mod boolean_short_circuit;
#[cfg(unix)]
#[path = "native_resource_session/boolean_short_circuit_traps.rs"]
mod boolean_short_circuit_traps;
#[path = "native_resource_session/checked.rs"]
mod checked;
#[path = "native_resource_session/continuations.rs"]
mod continuations;
#[path = "native_resource_session/early_ok.rs"]
mod early_ok;
#[path = "native_resource_session/identity.rs"]
mod identity;
#[path = "native_resource_session/inferred_shadowing.rs"]
mod inferred_shadowing;
#[path = "native_resource_session/mixed.rs"]
mod mixed;
#[cfg(unix)]
#[path = "native_resource_session/native.rs"]
mod native;
#[cfg(unix)]
#[path = "native_resource_session/trap.rs"]
mod native_trap;
#[path = "native_resource_session/nested_records.rs"]
mod nested_records;
#[cfg(unix)]
#[path = "native_resource_session/nested_resource_predicates.rs"]
mod nested_resource_predicates;
#[path = "native_resource_session/nested_returns.rs"]
mod nested_returns;
#[path = "native_resource_session/paired_continuations.rs"]
mod paired_continuations;
#[path = "native_resource_session/record_helpers.rs"]
mod record_helpers;
#[cfg(unix)]
#[path = "native_resource_session/resource_argument_predicates.rs"]
mod resource_argument_predicates;
#[cfg(unix)]
#[path = "native_resource_session/resource_buffer_lengths.rs"]
mod resource_buffer_lengths;
#[cfg(unix)]
#[path = "native_resource_session/resource_conditions.rs"]
mod resource_conditions;
#[cfg(unix)]
#[path = "native_resource_session/resource_error_codes.rs"]
mod resource_error_codes;
#[cfg(unix)]
#[path = "native_resource_session/resource_helper_predicates.rs"]
mod resource_helper_predicates;
#[cfg(unix)]
#[path = "native_resource_session/resource_i64_initializers.rs"]
mod resource_i64_initializers;
#[cfg(unix)]
#[path = "native_resource_session/resource_predicates.rs"]
mod resource_predicates;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_conditions.rs"]
mod resource_record_conditions;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_field_values.rs"]
mod resource_record_field_values;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_initializers.rs"]
mod resource_record_initializers;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_literals.rs"]
mod resource_record_literals;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_liveness.rs"]
mod resource_record_liveness;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_local_fields.rs"]
mod resource_record_local_fields;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_operands.rs"]
mod resource_record_operands;
#[cfg(unix)]
#[path = "native_resource_session/resource_record_scalar_initializers.rs"]
mod resource_record_scalar_initializers;
#[cfg(unix)]
#[path = "native_resource_session/resource_return_fields.rs"]
mod resource_return_fields;
#[cfg(unix)]
#[path = "native_resource_session/resource_wrapped_predicates.rs"]
mod resource_wrapped_predicates;
#[path = "native_resource_session/retained_bytes.rs"]
mod retained_bytes;
#[cfg(unix)]
#[path = "native_resource_session/scalar_selection.rs"]
mod scalar_selection;
#[path = "native_resource_session/scalar_shadowing.rs"]
mod scalar_shadowing;
#[path = "native_resource_session/transferred_bytes.rs"]
mod transferred_bytes;
#[cfg(unix)]
use yir_runtime_host::native_resource_session::NativeResourceSession;

fn compile(source: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_resource_session_source(source, &registration(), &Default::default())
        .unwrap()
        .emit_llvm()
        .unwrap()
}

#[cfg(unix)]
fn reference(name: &str, state: &[i64], delta: i64) -> Vec<i64> {
    let literal = |v: i64| {
        if v == i64::MIN {
            "(0 - 9223372036854775807 - 1)".to_owned()
        } else {
            v.to_string()
        }
    };
    let stores = state
        .iter()
        .enumerate()
        .map(|(i, v)| format!("store_at(buffer, {i}, {});", literal(*v)))
        .collect::<String>();
    let main = format!("fn main() -> i64 {{ let buffer: ref Buffer = alloc_buffer({}, 0); {stores} let result: Bytes = {name}(buffer, {}); let size: i64 = bytes_len(result); drop_bytes(result); free(buffer); return size; }}", state.len(), literal(delta));
    let source = SOURCE.replace("fn main() -> i64 { return 0; }", &main);
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    // Ordinary source compilation may inline the helper. Observe its actual
    // copy result rather than requiring a particular call-node shape.
    let copies = compiled
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "copy_buffer_owned")
        .filter_map(|n| trace.values.get(&n.name))
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 1);
    match copies[0] {
        yir_core::Value::OwnedBytes(words) => words.clone(),
        other => panic!("expected reference Bytes, got {other:?}"),
    }
}

fn registration() -> yir_core::native_resource_session::ResourceSessionRegistration {
    use yir_core::native_resource_session::*;
    ResourceSessionRegistration {
        contract: CONTRACT.to_owned(),
        id: "bytes".to_owned(),
        callbacks: ["start", "step", "stop"].map(str::to_owned),
        min_words: 1,
        max_words: 16,
        state_type: None,
        result_type: None,
    }
}

#[test]
fn resource_callback_source_reaches_verified_yir() {
    let compiled = compile(SOURCE);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&compiled.yir, &registration())
            .unwrap();
    assert!(!bridge.llvm_ir.contains("; deferred lowering"));
    assert!(bridge.llvm_ir.contains("call ptr @nuis_fn_step"));
    let clock = compiled
        .yir
        .nodes
        .iter()
        .find(|n| n.name == "scheduler_contract_cpu_clock_type")
        .unwrap();
    assert!(bridge.llvm_ir.contains(&clock.op.args[0]));
}

#[test]
#[cfg(unix)]
fn owned_buffer_bytes_callbacks_execute_and_release_on_every_completion() {
    let program = compile(SOURCE);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let initial = vec![0, i64::MIN, -17, 0x0123456789abcdef];
    let (mut session, completion) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.binding(&program.yir, &bridge),
        initial.clone(),
        10,
    )
    .unwrap();
    assert_eq!(completion.sequence, 1);
    assert_eq!(session.state(), reference("start", &initial, 10));
    assert_eq!(library.live_blobs(), 0);
    for (index, delta) in [3, -2, 100, -100].into_iter().cycle().take(80).enumerate() {
        let expected = session.state()[0].wrapping_add(delta);
        let reference = (index < 4).then(|| reference("step", session.state(), delta));
        let completed = session.event(delta).unwrap();
        assert_eq!(completed.sequence, index as u64 + 2);
        assert_eq!(completed.words, initial.len());
        assert_eq!(
            session.state(),
            &[expected, initial[1], initial[2], initial[3]]
        );
        if let Some(reference) = reference {
            assert_eq!(session.state(), reference);
        }
        assert_eq!(library.live_blobs(), 0);
    }
    let frame_path =
        std::env::temp_dir().join(format!("nuis-unclosed-frame-{}.ppm", std::process::id()));
    let frame = yir_runtime_host::native_resource_session::ResourceFrameExport::parse(
        "2x2",
        frame_path.to_str().unwrap(),
    )
    .unwrap();
    assert_eq!(
        frame.write_closed(&session).unwrap_err(),
        session.completion_status().unwrap_err()
    );
    assert!(!frame_path.exists());
    let expected = session.state()[0] + 5;
    let reference = reference("stop", session.state(), 5);
    session.close(5).unwrap().unwrap();
    assert_eq!(session.state()[0], expected);
    assert_eq!(session.state(), reference);
    assert_eq!(session.close(999).unwrap(), None);
    assert!(session.event(1).is_err());
    session.completion_status().unwrap();
    assert_eq!(library.live_blobs(), 0);
}

#[test]
#[cfg(unix)]
fn native_resource_preflight_rejects_capability_shape_and_identity_before_entry() {
    use yir_core::native_resource_session::CAPABILITY_TAG;
    let program = compile(SOURCE);
    let registration = registration();
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
            .unwrap();
    let library = native::Library::build(SOURCE, &program, &bridge.llvm_ir);
    let callback = library.callbacks(&bridge)[0];
    let mut input = [71_i64; 4];
    let mut output = [83_i64; 4];
    for (tag, words, count, invalid) in [
        (0, 4, 4, 0),
        (CAPABILITY_TAG, 0, 0, 0),
        (CAPABILITY_TAG, 17, 17, 0),
        (CAPABILITY_TAG, u64::MAX, u64::MAX, 0),
        (CAPABILITY_TAG, 4, 3, 0),
        (CAPABILITY_TAG, 4, 4, 1),
        (CAPABILITY_TAG, 4, 4, 2),
        (CAPABILITY_TAG, 4, 4, 3),
    ] {
        let source = if invalid == 1 {
            std::ptr::null_mut()
        } else if invalid == 3 {
            unsafe { input.as_mut_ptr().cast::<u8>().add(1).cast::<i64>() }
        } else {
            input.as_mut_ptr()
        };
        let dest = if invalid == 2 {
            std::ptr::null_mut()
        } else {
            output.as_mut_ptr()
        };
        assert_eq!(unsafe { callback(tag, source, words, 999, dest, count) }, 1);
        assert_eq!(input, [71; 4]);
        assert_eq!(output, [83; 4]);
        assert_eq!(library.live_blobs(), 0);
    }
    let mut changed = registration.clone();
    changed.max_words += 1;
    assert!(NativeResourceSession::open(
        &program.yir,
        &changed,
        library.binding(&program.yir, &bridge),
        vec![0; 4],
        0
    )
    .is_err());
    let mut changed_module = program.yir.clone();
    changed_module.nodes.reverse();
    assert!(NativeResourceSession::open(
        &changed_module,
        &registration,
        library.binding(&program.yir, &bridge),
        vec![0; 4],
        0
    )
    .is_err());
    for count in [0, 17] {
        assert!(NativeResourceSession::open(
            &program.yir,
            &registration,
            library.binding(&program.yir, &bridge),
            vec![0; count],
            0
        )
        .is_err());
    }
}

#[test]
#[cfg(unix)]
fn native_resource_failed_completion_releases_bytes_without_publishing_or_retrying() {
    use yir_runtime_host::ApplicationSessionPhase;
    let program = compile(SOURCE);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    for role in [0, 1, 2] {
        let start = bridge
            .llvm_ir
            .find(&format!("define i32 @{}(", bridge.symbols[role]))
            .unwrap();
        let mut fault = bridge.llvm_ir[..start].to_owned();
        fault.push_str(&bridge.llvm_ir[start..].replacen(
            "%same = icmp eq i64 %length, %bytes",
            "%same = icmp eq i64 %length, -1",
            1,
        ));
        let library = native::Library::build(SOURCE, &program, &fault);
        if role == 0 {
            assert!(NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.binding(&program.yir, &bridge),
                vec![0, -19],
                10,
            )
            .is_err());
            assert_eq!(library.live_blobs(), 0);
            continue;
        }
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.binding(&program.yir, &bridge),
            vec![0, -19],
            10,
        )
        .unwrap();
        if role == 1 {
            assert!(session.event(3).unwrap_err().contains("status 3"));
            assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
            assert_eq!(session.state(), &[10, -19]);
            assert_eq!(library.live_blobs(), 0);
            assert!(session.event(100).is_err());
            session.close(5).unwrap();
            assert_eq!(session.state(), &[15, -19]);
            assert_eq!(session.close(999).unwrap(), None);
        } else {
            let error = session.close(5).unwrap_err();
            assert_eq!(session.state(), &[10, -19]);
            assert_eq!(session.close(999).unwrap_err(), error);
        }
        assert!(session.completion_status().is_err());
        assert_eq!(session.phase(), ApplicationSessionPhase::Closed);
        assert_eq!(library.live_blobs(), 0);
        let path = std::env::temp_dir().join(format!(
            "nuis-fault-frame-{}-{role}.ppm",
            std::process::id()
        ));
        let frame = yir_runtime_host::native_resource_session::ResourceFrameExport::parse(
            "2x1",
            path.to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(
            frame.write_closed(&session).unwrap_err(),
            session.completion_status().unwrap_err()
        );
        assert!(!path.exists());
    }
}

#[test]
fn resource_registration_and_admission_keep_scalar_and_resource_scopes_separate() {
    let program = compile(SOURCE);
    let registration = registration();
    for function in registration.bind(&program.yir).unwrap() {
        assert!(yir_core::validate_session_function(function).is_err());
    }
    for changed in [
        yir_core::native_resource_session::ResourceSessionRegistration {
            contract: "wrong".to_owned(),
            ..registration.clone()
        },
        yir_core::native_resource_session::ResourceSessionRegistration {
            max_words: usize::MAX,
            ..registration.clone()
        },
        yir_core::native_resource_session::ResourceSessionRegistration {
            min_words: 17,
            ..registration.clone()
        },
        yir_core::native_resource_session::ResourceSessionRegistration {
            callbacks: ["start".to_owned(), "start".to_owned(), "stop".to_owned()],
            ..registration.clone()
        },
    ] {
        assert!(changed.bind(&program.yir).is_err());
    }
    for source in [
        SOURCE.replace("let value: i64 =", "print(delta); let value: i64 ="),
        SOURCE.replace(
            "let value: i64 =",
            "let temp: ref Buffer = alloc_buffer(1, 0); free(temp); let value: i64 =",
        ),
    ] {
        let compiled = compile(&source);
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &compiled.yir,
            &registration
        )
        .is_err());
    }
}

#[test]
#[cfg(unix)]
fn resource_callbacks_support_empty_maximal_and_reordered_scopes_with_private_bytes() {
    let mut source = "mod cpu Main {".to_owned();
    for name in ["start", "step", "stop"] {
        source.push_str(&format!("fn {name}(buffer: ref Buffer, delta: i64) -> Bytes {{ let temporary: Bytes = copy_bytes(buffer); drop_bytes(temporary); return copy_bytes(buffer); }}"));
    }
    source.push_str("fn main() -> i64 { return 0; } }");
    let mut program = compile(&source);
    let registration = yir_core::native_resource_session::ResourceSessionRegistration {
        min_words: 0,
        max_words: yir_core::native_resource_session::MAX_BUFFER_WORDS,
        ..registration()
    };
    for reordered in [false, true] {
        if reordered {
            program.yir.nodes.reverse();
            program.yir.functions.reverse();
            for function in &mut program.yir.functions {
                function.body_nodes.reverse();
            }
        }
        let bridge =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration)
                .unwrap();
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        for words in [0, 1, registration.max_words] {
            let initial = (0..words)
                .map(|v| (v as i64).wrapping_mul(-999999999999))
                .collect::<Vec<_>>();
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration,
                library.binding(&program.yir, &bridge),
                initial.clone(),
                0,
            )
            .unwrap();
            assert_eq!(session.state(), initial);
            session.event(0).unwrap();
            assert_eq!(session.state(), initial);
            session.close(0).unwrap();
            session.completion_status().unwrap();
            assert_eq!(session.state(), initial);
            assert_eq!(library.live_blobs(), 0);
        }
    }
    let copies = (0..9)
        .map(|index| {
            format!("let temp{index}: Bytes = copy_bytes(buffer); drop_bytes(temp{index});")
        })
        .collect::<String>();
    let excessive = SOURCE.replace("let value: i64 =", &format!("{copies} let value: i64 ="));
    let compiled = compile(&excessive);
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&compiled.yir, &registration)
            .unwrap_err()
            .contains("allocation bound")
    );
}

#[test]
fn resource_callback_rejects_signature_result_and_linear_ownership_drift() {
    let program = compile(SOURCE);
    for case in 0..6 {
        let mut changed = program.yir.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|f| f.name == "step")
            .unwrap();
        match case {
            0 => function.parameters[0].ownership = yir_core::YirValueOwnership::Value,
            1 => function.parameters[1].ty = "f64".to_owned(),
            2 => {
                function.result.as_mut().unwrap().ownership = yir_core::YirValueOwnership::Borrowed
            }
            3 => function.result.as_mut().unwrap().node = "missing_result".to_owned(),
            4 => function.parameters[1].node = function.parameters[0].node.clone(),
            _ => function.domain = "shader".to_owned(),
        }
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
    let mut changed = program.yir.clone();
    let parameter = registration().bind(&changed).unwrap()[1].parameters[1]
        .node
        .clone();
    let node = changed
        .nodes
        .iter_mut()
        .find(|n| n.name == parameter)
        .unwrap();
    node.op.instruction = "const_i64".to_owned();
    node.op.args = vec!["1".to_owned()];
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
            .is_err()
    );
    let source = SOURCE.replace(
        "let value: i64 =",
        "let private: Bytes = copy_bytes(buffer); drop_bytes(private); let value: i64 =",
    );
    let program = compile(&source);
    for case in 0..3 {
        let mut changed = program.yir.clone();
        let drop = changed
            .nodes
            .iter_mut()
            .find(|n| n.op.instruction == "drop_owned_bytes")
            .unwrap();
        let private = drop.op.args[0].clone();
        if case == 0 {
            drop.op.instruction = "owned_bytes_len".to_owned();
        } else if case == 1 {
            drop.op.instruction = "return_owned_bytes".to_owned();
        } else {
            let result = changed
                .functions
                .iter()
                .find(|f| f.name == "step")
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .node
                .clone();
            changed
                .nodes
                .iter_mut()
                .find(|n| n.name == result)
                .unwrap()
                .op
                .args = vec![private];
        }
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
}

#[test]
fn resource_callbacks_never_discard_external_ordering_or_lifetime_edges() {
    let program = compile(SOURCE);
    let roots = registration().bind(&program.yir).unwrap();
    let from = roots[0].result.as_ref().unwrap().node.clone();
    let to = roots[1].parameters[0].node.clone();
    for kind in [
        yir_core::EdgeKind::Dep,
        yir_core::EdgeKind::Effect,
        yir_core::EdgeKind::Lifetime,
    ] {
        let mut changed = program.yir.clone();
        changed.edges.push(yir_core::Edge {
            kind,
            from: from.clone(),
            to: to.clone(),
        });
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
    let mut changed = program.yir.clone();
    changed.node_lanes.insert(from, "fn:step".to_owned());
    assert!(
        yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
            .is_err()
    );
}

#[test]
fn resource_contract_metadata_never_grants_executable_or_lifetime_ingress() {
    let program = compile(SOURCE);
    let name = "scheduler_contract_cpu_clock_type";
    for case in 0..3 {
        let mut changed = program.yir.clone();
        match case {
            0 => {
                let node = changed.nodes.iter_mut().find(|n| n.name == name).unwrap();
                node.op.instruction = "const_i64".to_owned();
                node.op.args = vec!["1".to_owned()];
            }
            1 => {
                changed
                    .node_lanes
                    .insert(name.to_owned(), "main".to_owned());
            }
            _ => {
                changed
                    .edges
                    .iter_mut()
                    .find(|e| e.from == name)
                    .unwrap()
                    .kind = yir_core::EdgeKind::Lifetime;
            }
        }
        assert!(yir_lower_llvm::native_resource_session::emit_registered(
            &changed,
            &registration()
        )
        .is_err());
    }
}

#[test]
#[cfg(unix)]
fn resource_allocation_failure_is_nonpublishing_and_cleanup_uses_last_state() {
    let program = compile(SOURCE);
    let bridge =
        yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
            .unwrap();
    let fault = bridge.llvm_ir.replace(
        "%blob = call ptr @nuis_fn_step(ptr %buffer, i64 %words, i64 %argument)",
        "%blob = inttoptr i64 0 to ptr",
    );
    assert_ne!(fault, bridge.llvm_ir);
    let library = native::Library::build(SOURCE, &program, &fault);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.binding(&program.yir, &bridge),
        vec![0, -7],
        11,
    )
    .unwrap();
    assert!(session.event(9).unwrap_err().contains("status 2"));
    assert_eq!(session.state(), &[11, -7]);
    session.close(2).unwrap();
    assert_eq!(session.state(), &[13, -7]);
    assert!(session.completion_status().is_err());
    assert_eq!(library.live_blobs(), 0);
}
