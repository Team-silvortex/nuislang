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
