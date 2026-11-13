use super::*;
#[path = "state_return_eager_pair_model.rs"]
mod model;
#[path = "word_session_probe.rs"]
mod probe;

const SOURCE: &str = include_str!("state_return_eager_pair.ns");

fn execute(input: &[i64], prior: i64, op: &str) -> Result<probe::Row, Vec<i64>> {
    model::execute(input, prior, op).map(|(control, trace)| probe::Row {
        input: input.to_vec(),
        state: vec![0, control],
        trace,
    })
}

fn values(input: &[i64]) -> Vec<Value> {
    input
        .iter()
        .enumerate()
        .map(|(i, value)| {
            if i < 5 {
                Value::Bool(*value != 0)
            } else {
                Value::Int(*value)
            }
        })
        .collect()
}

fn printed(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn state_return_eager_pair_keeps_eager_order_lazy_children_and_complete_state() {
    check_source(SOURCE, 12, 0);
}

#[test]
fn state_return_direct_eager_pair_keeps_eager_order_lazy_children_and_complete_state() {
    for (kind, word) in [
        ("f32", 1.5f32.to_bits() as i64),
        ("f64", 1.5f64.to_bits() as i64),
    ] {
        let source = include_str!("state_return_direct_eager_pair.ns")
            .replace("f64", kind)
            .replace("0.0", "1.5");
        check_source(&source, 0, word);
    }
}

#[test]
fn state_return_direct_eager_pair_keeps_explicit_print_and_entry_authority() {
    use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};
    let project = Project::with_source(include_str!("state_return_direct_eager_pair.ns"));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for reverse in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reverse);
        let sites = module
            .nodes
            .iter()
            .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
            .map(|n| n.name.clone())
            .collect::<Vec<_>>();
        assert!(sites.len() >= 7);
        let full = I64PrintPolicy::new(sites.clone()).unwrap();
        let bridge = emit_registered_with_i64_prints(&module, "counter", &full, 0, 192).unwrap();
        assert_eq!(
            bridge
                .callbacks
                .iter()
                .map(|c| c.arguments.len())
                .collect::<Vec<_>>(),
            [8, 10, 2]
        );
        assert_eq!(bridge.state_fields.len(), 2);
        for missing in &sites {
            let partial =
                I64PrintPolicy::new(sites.iter().filter(|s| *s != missing).cloned()).unwrap();
            let error = emit_registered_with_i64_prints(&module, "counter", &partial, 0, 192)
                .err()
                .expect("missing print grant must fail");
            assert!(error.contains("print"), "{error}");
        }
    }
    probe::check_i64_helper_entry_limit_rejection(
        include_str!("state_return_direct_eager_pair.ns"),
        &[probe::Sequence {
            rows: vec![execute(&[1, 1, 0, 0, 1, 1, 1, 1], 0, "==").unwrap()],
        }],
        &[],
        0,
        1,
    );
}

fn check_source(source: &str, loop_limit: u64, total: i64) {
    for op in ["==", "!=", "^"] {
        for binding in ["let", "const"] {
            let source = source
                .replace(" == ", &format!(" {op} "))
                .replace("let local: bool", &format!("{binding} local: bool"));
            let start = execute(&[0, 0, 0, 0, 0, 0, 0, 0], 0, op).unwrap();
            let mut rows = vec![probe::Sequence {
                rows: vec![start.clone()],
            }];
            let mut failures = Vec::new();
            for bits in 0..16 {
                let input = vec![
                    1,
                    bits & 1,
                    (bits >> 1) & 1,
                    (bits >> 2) & 1,
                    (bits >> 3) & 1,
                    1,
                    1,
                    1,
                ];
                let accepted = execute(&input, start.state[1], op).unwrap();
                let prefix = vec![start.clone(), accepted.clone()];
                rows.push(probe::Sequence {
                    rows: prefix.clone(),
                });
                for poison in [&[5][..], &[6][..], &[7][..], &[5, 6, 7][..]] {
                    let mut bad = input.clone();
                    for index in poison {
                        bad[*index] = 0;
                    }
                    match execute(&bad, accepted.state[1], op) {
                        Ok(row) => {
                            let mut sequence = prefix.clone();
                            sequence.push(row);
                            rows.push(probe::Sequence { rows: sequence });
                        }
                        Err(before_failure) => failures.push(probe::SequenceFailure {
                            accepted: probe::Sequence {
                                rows: prefix.clone(),
                            },
                            input: bad,
                            before_failure,
                        }),
                    }
                }
            }
            assert_eq!(
                rows.len() + failures.len(),
                81,
                "no skipped poison case is discarded"
            );
            // An eager operator never uses its left result to skip its right operand.
            assert_eq!(
                execute(&[1, 1, 0, 0, 1, 1, 1, 1], 0, op).unwrap().trace,
                [10, 20, 21, 22, 23, 30]
            );
            assert_eq!(
                execute(&[1, 1, 0, 0, 1, 0, 0, 0], 0, op)
                    .err()
                    .expect("left failure"),
                [10, 20]
            );
            assert_eq!(
                execute(&[1, 0, 0, 0, 1, 0, 0, 0], 0, op)
                    .err()
                    .expect("right failure"),
                [10, 22]
            );
            for row in rows
                .iter_mut()
                .flat_map(|s| &mut s.rows)
                .chain(failures.iter_mut().flat_map(|f| &mut f.accepted.rows))
            {
                row.state[0] = total;
            }
            probe::check_sequences_with_i64_prints(&source, &rows, &failures, &[], loop_limit, 192);
            let project = Project::with_source(&source);
            let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
            let registry = yir_verify::default_registry();
            for sequence in &rows {
                let first = &sequence.rows[0];
                let (mut session, opened) = ApplicationSession::open_registered(
                    &compiled.yir,
                    &registry,
                    "counter",
                    values(&first.input),
                )
                .unwrap();
                assert_eq!(printed(&opened.events), first.trace);
                assert_eq!(
                    state_words(session.state()),
                    first.state.iter().map(|w| *w as u64).collect::<Vec<_>>()
                );
                for row in &sequence.rows[1..] {
                    let report = session.event(values(&row.input)).unwrap();
                    assert_eq!(printed(&report.events), row.trace);
                    assert_eq!(
                        state_words(session.state()),
                        row.state.iter().map(|w| *w as u64).collect::<Vec<_>>()
                    );
                }
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for failure in &failures {
                let first = &failure.accepted.rows[0];
                let (mut session, _) = ApplicationSession::open_registered(
                    &compiled.yir,
                    &registry,
                    "counter",
                    values(&first.input),
                )
                .unwrap();
                for row in &failure.accepted.rows[1..] {
                    session.event(values(&row.input)).unwrap();
                }
                let prior = state_words(session.state());
                assert!(session.event(values(&failure.input)).is_err());
                assert_eq!(
                    session.phase(),
                    yir_runtime_host::ApplicationSessionPhase::Faulted
                );
                assert_eq!(state_words(session.state()), prior);
                session.close(vec![]).unwrap();
                assert_eq!(state_words(session.state()), prior);
                assert!(session.completion_status().is_err());
            }
        }
    }
}
