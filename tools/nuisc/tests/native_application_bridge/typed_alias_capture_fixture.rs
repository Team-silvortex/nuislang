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
