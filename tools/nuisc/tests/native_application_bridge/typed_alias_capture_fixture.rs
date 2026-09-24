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
