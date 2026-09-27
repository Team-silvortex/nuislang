pub fn source(nested: bool) -> String {
    let fields = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let definitions = if nested {
        format!("struct Payload {{ {fields} }} struct State {{ payload: Payload }}")
    } else {
        format!("struct State {{ {fields} }}")
    };
    let wrap = |values: String| {
        if nested {
            format!("State {{ payload: Payload {{ {values} }} }}")
        } else {
            format!("State {{ {values} }}")
        }
    };
    let path = if nested { "payload." } else { "" };
    let initial = wrap(
        (0..64)
            .rev()
            .map(|i| format!("f{i}: seed + {i}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    let advanced = wrap(
        (0..64)
            .rev()
            .map(|i| format!("f{i}: value.{path}f{i}{}", if i == 0 { " + 1" } else { "" }))
            .collect::<Vec<_>>()
            .join(", "),
    );
    format!(
        "mod cpu Main {{
        {definitions}
        @noinline fn relay(value: State) -> State {{ return {advanced}; }}
        fn start(seed: i64) -> State {{ return {initial}; }}
        fn step(state: State) -> State {{
            let next = state;
            if state.{path}f0 > 0 {{ let next = relay(state); }}
            return next;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

pub fn mixed_source() -> String {
    let kinds = ["bool", "i32", "i64", "f32", "f64"];
    let fields = (0..64)
        .map(|i| format!("f{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    let params = (0..64)
        .map(|i| format!("p{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    let initial = (0..64)
        .rev()
        .map(|i| format!("f{i}: p{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let advanced = (0..64)
        .rev()
        .map(|i| {
            format!(
                "f{i}: value.payload.f{i}{}",
                if i % 5 == 2 { " + 1" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("mod cpu Main {{
        struct Payload {{ {fields} }}
        struct State {{ payload: Payload }}
        @noinline fn relay(value: State) -> State {{ return State {{ payload: Payload {{ {advanced} }} }}; }}
        fn start({params}) -> State {{ return State {{ payload: Payload {{ {initial} }} }}; }}
        fn step(state: State) -> State {{
            let first = relay(state);
            let second = relay(first);
            let next = first;
            if state.payload.f0 {{ let next = relay(second); }}
            else {{ let next = relay(first); }}
            return next;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}")
}
