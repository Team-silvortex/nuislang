pub fn source(count: usize, breaking: bool) -> String {
    let fields = (0..count)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let initial = (0..count)
        .rev()
        .map(|i| format!("f{i}: seed + {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let advanced = (0..count)
        .rev()
        .map(|i| format!("f{i}: value.f{i} + 1"))
        .collect::<Vec<_>>()
        .join(", ");
    let alternate = (0..count)
        .rev()
        .map(|i| format!("f{i}: carry.f{i} + 2"))
        .collect::<Vec<_>>()
        .join(", ");
    let control = if breaking { "if i == 3 { break; }" } else { "" };
    let body = if breaking {
        format!(
            "if i <= 2 {{ let carry = relay(carry); }}
                else {{ let carry = State {{ {alternate} }}; }} {control}"
        )
    } else {
        let shifted = (0..count)
            .rev()
            .map(|n| format!("f{n}: carry.f{n}{}", if n == 0 { " + i" } else { "" }))
            .collect::<Vec<_>>()
            .join(", ");
        format!("let carry = relay(carry); let carry = State {{ {shifted} }};")
    };
    format!(
        "mod cpu Main {{
        struct State {{ {fields} }}
        @noinline fn relay(value: State) -> State {{ return State {{ {advanced} }}; }}
        fn start(seed: i64) -> State {{ return State {{ {initial} }}; }}
        fn step(state: State) -> State {{
            let carry = state; let i = 0; let limit = state.f1;
            while i < limit {{
                let i = i + 1;
                {body}
            }}
            return carry;
        }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}
