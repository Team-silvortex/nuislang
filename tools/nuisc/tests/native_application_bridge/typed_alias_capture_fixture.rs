pub fn source() -> String {
    let base = include_str!("typed_sparse_captures.ns");
    let (prefix, tail) = base.split_once("    @noinline fn choose(").unwrap();
    let (_, suffix) = tail.split_once("    fn start(").unwrap();
    format!(
        "{prefix}
    @noinline fn relay(value: i64) -> i64 {{ return value; }}
    @noinline fn choose(payload: Payload) -> i64 {{
        if payload.f2 > 0 {{
            let saved = payload;
            let second: Payload = saved;
            return relay(second.f0);
        }}
        let fallback = payload;
        return fallback.f63 / fallback.f1;
    }}
    fn start({suffix}"
    )
    .replace("choose(state)", "choose(state.payload)")
}

pub fn scalar_copy_source(computed: bool) -> String {
    let copies = (0..64)
        .map(|i| format!("let copy{i}: i64 = payload.f{i}; const saved{i}: i64 = copy{i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let fields = (0..64)
        .map(|i| {
            if computed && i == 62 {
                format!("f{i}: relay(payload.f{i})")
            } else {
                format!("f{i}: saved{i}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let source = source().replace(
        "let saved = payload;",
        &format!("{copies} let saved = Payload {{ {fields} }};"),
    );
    if computed {
        source
            .replace("let second: Payload = saved;", "")
            .replace("return relay(second.f0);", "return saved.f0;")
    } else {
        source
    }
}

pub fn scalar_transport_source() -> String {
    scalar_copy_source(true).replace("return saved.f0;", "return relay(saved.f0);")
}

pub fn scalar_checked_transport_source() -> String {
    scalar_transport_source()
        .replace(
            "    @noinline fn relay",
            "    @noinline fn checked(value: i64) -> i64 { return 1 / (value - 62); }
    @noinline fn relay",
        )
        .replace("f62: relay(payload.f62)", "f62: checked(payload.f62)")
}

pub fn aggregate_transport_source() -> String {
    scalar_transport_source()
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace("return relay(saved.f0);", "return consume_snapshot(saved);")
}

pub fn evaluated_scalar_source(checked: bool) -> String {
    let fields = (0..64)
        .map(|i| {
            format!(
                "f{i}: {}",
                match i {
                    0 => "selected".into(),
                    62 => "unused".into(),
                    _ => format!("payload.f{i}"),
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let source = source().replace(
        "let saved = payload;",
        &format!(
            "let selected = relay(payload.f0); const unused: i64 = relay(payload.f62);
            let saved = Payload {{ {fields} }};"
        ),
    );
    if checked {
        source
            .replace(
                "    @noinline fn relay",
                "    @noinline fn checked(value: i64) -> i64 { return 1 / (value - 62); }
    @noinline fn relay",
            )
            .replace(
                "const unused: i64 = relay(payload.f62)",
                "const unused: i64 = checked(payload.f62)",
            )
    } else {
        source
    }
}

pub fn evaluated_scalar_transport_source() -> String {
    evaluated_scalar_source(false)
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return consume_snapshot(second);",
        )
}

pub fn aggregate_result_source(checked: bool) -> String {
    let produced = (0..64)
        .map(|i| {
            format!(
                "f{i}: {}",
                match i {
                    0 => "relay(value)".into(),
                    62 => if checked {
                        "checked(unused)"
                    } else {
                        "relay(unused)"
                    }
                    .into(),
                    _ => i.to_string(),
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let copied = (0..64)
        .map(|i| {
            format!(
                "f{i}: {}",
                match i {
                    0 => "first.f0".into(),
                    62 => "later.f62".into(),
                    _ => format!("payload.f{i}"),
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let source = source()
        .replace(
            "    @noinline fn relay",
            &format!(
                "    @noinline fn checked(value: i64) -> i64 {{ return 1 / (value - 62); }}
    @noinline fn produce(value: i64, unused: i64) -> Payload {{ return Payload {{ {produced} }}; }}
    @noinline fn relay"
            ),
        )
        .replace(
            "let saved = payload;",
            &format!(
                "let first = produce(payload.f0, payload.f62);
            const later: Payload = produce(payload.f0 + 30, payload.f62);
            let saved = Payload {{ {copied} }};"
            ),
        );
    if checked {
        // The first result completes; only the later unused field traps.
        source.replace(
            "let first = produce(payload.f0, payload.f62);",
            "let first = produce(payload.f0, payload.f62 + 1);",
        )
    } else {
        source
    }
}

pub fn aggregate_result_transport_source() -> String {
    aggregate_result_source(false)
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return consume_snapshot(second);",
        )
}

pub fn inline_record_argument_source(checked: bool) -> String {
    let source = aggregate_result_source(false)
        .replace(
            "    @noinline fn produce(value: i64, unused: i64)",
            "    struct CallInput { used: i64, unused: i64 }
    @noinline fn produce(input: CallInput)",
        )
        .replace("f0: relay(value)", "f0: input.used")
        .replace("f62: relay(unused)", "f62: input.unused")
        .replace(
            "let first = produce(payload.f0, payload.f62);",
            "let first = produce(CallInput { used: relay(payload.f0), unused: relay(payload.f62) });",
        )
        .replace(
            "const later: Payload = produce(payload.f0 + 30, payload.f62);",
            "const later: Payload = produce(CallInput { unused: relay(payload.f62), used: relay(payload.f0 + 30) });",
        );
    if checked {
        source
            .replace(
                "unused: relay(payload.f62) });",
                "unused: checked(payload.f62 + 1) });",
            )
            .replace(
                "{ unused: relay(payload.f62),",
                "{ unused: checked(payload.f62),",
            )
    } else {
        source
    }
}

pub fn inline_record_argument_transport_source() -> String {
    inline_record_argument_source(false)
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return consume_snapshot(second);",
        )
}

pub fn materialized_record_argument_source(checked: bool) -> String {
    // Keep the argument constructors (including their reversed field order)
    // before the calls, and pass only stored immutable values through aliases.
    let source = inline_record_argument_source(checked);
    let first = if checked {
        "checked(payload.f62 + 1)"
    } else {
        "relay(payload.f62)"
    };
    let later = if checked {
        "checked(payload.f62)"
    } else {
        "relay(payload.f62)"
    };
    source
        .replace(
            &format!("let first = produce(CallInput {{ used: relay(payload.f0), unused: {first} }});"),
            &format!("let first_input = CallInput {{ used: relay(payload.f0), unused: {first} }};
            let first_alias = first_input; let first = produce(first_alias);"),
        )
        .replace(
            &format!("const later: Payload = produce(CallInput {{ unused: {later}, used: relay(payload.f0 + 30) }});"),
            &format!("const later_input: CallInput = CallInput {{ unused: {later}, used: relay(payload.f0 + 30) }};
            let later_alias = later_input; const later: Payload = produce(later_alias);"),
        )
}

pub fn materialized_record_argument_transport_source() -> String {
    materialized_record_argument_source(false)
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return consume_snapshot(second);",
        )
}

pub fn stored_projection_source(checked: bool) -> String {
    let source = aggregate_result_source(checked);
    let signature =
        "    @noinline fn produce(value: i64, unused: i64) -> Payload { return Payload { ";
    let (prefix, tail) = source.split_once(signature).unwrap();
    let (fields, suffix) = tail.split_once(" }; }").unwrap();
    let fields = fields.split(", ").collect::<Vec<_>>();
    assert_eq!(fields.len(), 64);
    let definition = |range: std::ops::Range<usize>| {
        range
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let first_unused = if checked {
        "payload.f62 + 1"
    } else {
        "payload.f62"
    };
    format!(
        "{prefix} struct SelectedPayload {{ {} }} struct IgnoredPayload {{ {} }}
    struct CallEnvelope {{ selected: SelectedPayload, ignored: IgnoredPayload }}
    @noinline fn produce(value: i64, unused: i64) -> CallEnvelope {{
        return CallEnvelope {{ selected: SelectedPayload {{ {} }}, ignored: IgnoredPayload {{ {} }} }};
    }}{suffix}",
        definition(0..32), definition(32..64), fields[..32].join(", "), fields[32..].join(", "),
    )
    .replace(
        &format!("let first = produce(payload.f0, {first_unused});"),
        &format!("let first = produce(payload.f0, {first_unused}).selected;"),
    )
    .replace(
        "const later: Payload = produce(payload.f0 + 30, payload.f62);",
        "const later: SelectedPayload = produce(payload.f0 + 30, payload.f62).selected;",
    )
    .replace("f62: later.f62", "f62: later.f30")
}

pub fn stored_projection_transport_source() -> String {
    stored_projection_source(false)
        .replace(
            "    @noinline fn relay",
            "    @noinline fn consume_snapshot(value: Payload) -> i64 { return value.f0; }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return consume_snapshot(second);",
        )
}

pub fn rebound_source() -> String {
    let fields = (0..64)
        .map(|i| format!("f{i}: {}", if i == 0 { 30 } else { 0 }))
        .collect::<Vec<_>>()
        .join(", ");
    source()
        .replace(
            "return relay(second.f0);",
            &format!(
                "let payload = Payload {{ {fields} }};
            let saved = payload;
            return relay(second.f0) + saved.f0;"
            ),
        )
        .replace(
            "return fallback.f63 / fallback.f1;",
            &format!(
                "let payload = Payload {{ {fields} }};
        return fallback.f63 / fallback.f1 + payload.f0;"
            ),
        )
}

pub fn scoped_source() -> String {
    let base = include_str!("typed_sparse_captures.ns");
    let (prefix, tail) = base.split_once("    @noinline fn choose(").unwrap();
    let (_, suffix) = tail.split_once("    fn start(").unwrap();
    format!(
        "{prefix}
    @noinline fn relay(value: i64) -> i64 {{ return value; }}
    @noinline fn choose(payload: Payload) -> i64 {{
        if payload.f2 > 0 {{
            if payload.f0 > 0 {{
                let saved: Payload = payload;
                let second = saved;
                return relay(second.f0);
            }} else {{
                const saved: Payload = payload;
                let second = saved;
                return relay(second.f0);
            }}
        }}
        let saved: Payload = payload;
        let second = saved;
        return second.f63 / second.f1;
    }}
    fn start({suffix}"
    )
    .replace("choose(state)", "choose(state.payload)")
}

pub fn scoped_rebound_source() -> String {
    let fields = (0..64)
        .map(|i| format!("f{i}: {}", if i == 0 { 30 } else { 0 }))
        .collect::<Vec<_>>()
        .join(", ");
    scoped_source()
        .replace("const saved: Payload", "let saved: Payload")
        .replace(
            "return relay(second.f0);",
            &format!("let saved = Payload {{ {fields} }}; return relay(second.f0) + saved.f0;"),
        )
        .replace(
            "return second.f63 / second.f1;",
            &format!(
                "let saved = Payload {{ {fields} }}; return second.f63 / second.f1 + saved.f0;"
            ),
        )
}

pub fn loop_source() -> String {
    source()
        .replace(
            "    @noinline fn relay",
            "    struct QuotientInput { numerator: i64, denominator: i64 }
    @noinline fn repeat_division(input: QuotientInput, limit: i64) -> i64 {
        let i = 0; let result = 0;
        while i < limit {
            let i = i + 1;
            let saved = input; let second = saved;
            let result = relay(second.numerator) / second.denominator;
        }
        return result;
    }
    @noinline fn relay",
        )
        .replace(
            "return fallback.f63 / fallback.f1;",
            "return repeat_division(QuotientInput { numerator: fallback.f63, denominator: fallback.f1 }, 2);",
        )
}

pub fn wide_loop_source() -> String {
    source().replace(
        "return fallback.f63 / fallback.f1;",
        "let i = 0; let result = 0;
        while i < 2 {
            let i = i + 1;
            let saved = fallback; let second = saved;
            let result = relay(second.f63) / second.f1;
        }
        return result;",
    )
}

pub fn terminal_snapshot_source() -> String {
    let fields = (0..64)
        .map(|i| format!("f{i}: {}", if i == 0 { 30 } else { 0 }))
        .collect::<Vec<_>>()
        .join(", ");
    source()
        .replace(
            "if payload.f2 > 0 {",
            "let current = payload; let old = current;\n        if payload.f2 > 0 {",
        )
        .replace("let saved = payload;", "let saved = old;")
        .replace(
            "return relay(second.f0);",
            &format!("let current = Payload {{ {fields} }}; return relay(second.f0) + current.f0;"),
        )
        .replace("let fallback = payload;", "let fallback = current;")
        .replace(
            "return fallback.f63 / fallback.f1;",
            &format!("let current = Payload {{ {fields} }}; return fallback.f63 / fallback.f1 + current.f0;"),
        )
}

pub fn join_source() -> String {
    let base = include_str!("typed_sparse_captures.ns");
    let (prefix, tail) = base.split_once("    @noinline fn choose(").unwrap();
    let (_, suffix) = tail.split_once("    fn start(").unwrap();
    let fields = |value: &str| {
        (0..64)
            .map(|i| {
                format!(
                    "f{i}: {}",
                    match i {
                        0 => value,
                        3 => "999",
                        _ => "0",
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let yes = fields("relay(old.f0) + 27");
    let no = fields("old.f63 / old.f1 + 27");
    format!(
        "{prefix}
    @noinline fn relay(value: i64) -> i64 {{ return value; }}
    @noinline fn choose(payload: Payload) -> i64 {{
        let current = payload; let old = current;
        if payload.f2 > 0 {{
            let current = Payload {{ {yes} }};
        }} else {{
            let current = Payload {{ {no} }};
        }}
        return current.f0 + old.f3;
    }}
    fn start({suffix}"
    )
    .replace("choose(state)", "choose(state.payload)")
}

pub fn loop_join_source() -> String {
    source()
        .replace(
            "    @noinline fn relay",
            "    struct IterationInput { value: i64, divisor: i64, marker: i64, unused: i64 }
    @noinline fn repeat_update(input: IterationInput, limit: i64) -> i64 {
        let current = input; let i = 0; let result = 0;
        while i < limit {
            let i = i + 1;
            let current = current; let before = current;
            let current = IterationInput { value: before.value, divisor: before.divisor, marker: before.marker + 10, unused: 999 };
            let result = relay(before.value) / before.divisor + before.marker + current.marker;
        }
        return result;
    }
    @noinline fn relay",
        )
        .replace(
            "return relay(second.f0);",
            "return repeat_update(IterationInput { value: second.f0, divisor: 1, marker: 0, unused: 7 }, 2)
                + repeat_update(IterationInput { value: 3, divisor: 0, marker: 0, unused: 7 }, 0);",
        )
        .replace(
            "return fallback.f63 / fallback.f1;",
            "return repeat_update(IterationInput { value: fallback.f63, divisor: fallback.f1, marker: 0, unused: 7 }, 2)
                + repeat_update(IterationInput { value: 3, divisor: 0, marker: 0, unused: 7 }, 0);",
        )
}

pub fn field_seed_source() -> String {
    let base = include_str!("typed_sparse_captures.ns");
    let (prefix, tail) = base.split_once("    @noinline fn choose(").unwrap();
    let (_, suffix) = tail.split_once("    fn start(").unwrap();
    format!(
        "{prefix}
    struct IterationInput {{ value: i64, divisor: i64, marker: i64, unused: i64 }}
    struct IterationWords {{ carry0: i64, carry1: i64, carry2: i64, carry3: i64, carry4: i64 }}
    @noinline fn relay(value: i64) -> i64 {{ return value; }}
    @noinline fn select_input(payload: Payload) -> IterationInput {{
        if payload.f2 > 0 {{ return IterationInput {{ value: relay(payload.f0), divisor: 1, marker: 0, unused: 7 }}; }}
        return IterationInput {{ value: payload.f63, divisor: payload.f1, marker: 0, unused: 7 }};
    }}
    @noinline fn update_fields(marker: i64, previous: i64, value: i64, unused: i64, divisor: i64) -> IterationWords {{
        return IterationWords {{ carry0: value, carry1: divisor, carry2: marker + 10, carry3: 999,
            carry4: relay(value) / divisor + marker + marker + 10 }};
    }}
    @noinline fn repeat_update(input: IterationInput, limit: i64) -> i64 {{
        let current = input; let i = 0; let result = 0;
        while i < limit {{
            let words = update_fields(current.marker, result, current.value, current.unused, current.divisor);
            let current = IterationInput {{ value: words.carry0, divisor: words.carry1, marker: words.carry2, unused: words.carry3 }};
            let result = words.carry4;
            let i = i + 1;
        }}
        return result;
    }}
    @noinline fn choose(payload: Payload) -> i64 {{
        return repeat_update(select_input(payload), 2)
            + repeat_update(IterationInput {{ value: 3, divisor: 0, marker: 0, unused: 7 }}, 0);
    }}
    fn start({suffix}"
    )
    .replace("choose(state)", "choose(state.payload)")
}

pub fn unread_record_source() -> String {
    loop_join_source()
        .replace("before.value", "input.value")
        .replace("before.divisor", "input.divisor")
        .replace("before.marker + 10", "i * 10")
        .replace(
            "before.marker + current.marker",
            "(i - 1) * 10 + current.marker",
        )
}

pub fn partial_field_seed_source() -> String {
    field_seed_source()
        .replace(
            "value: i64, unused: i64, divisor: i64",
            "value: i64, divisor: i64",
        )
        .replace(
            "current.value, current.unused, current.divisor",
            "current.value, current.divisor",
        )
}
