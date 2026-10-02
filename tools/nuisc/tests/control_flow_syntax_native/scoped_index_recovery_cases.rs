pub struct Case {
    pub name: &'static str,
    pub body: String,
    pub recoveries: usize,
    pub expected: [i64; 5],
}

pub fn cases() -> Vec<Case> {
    let prefix = "let index = 0; let total = 0;";
    let iteration = "while index < limit {
        let index = index + 1;
        let total = total + index;
        if index == 2 { break; }
    }";
    let nested = "let outer = 0;
        while outer < limit {
            let outer = outer + 1;
            let index = index; let total = total;
            CHILD
        } return total;";
    let child = "while index < limit {
        let index = index + 1;
        let total = total + index;
        break;
    }";
    vec![
        Case {
            name: "dead_exit",
            body: format!("{prefix} {iteration} return total;"),
            recoveries: 0,
            expected: [0, 1, 3, 3, 3],
        },
        Case {
            name: "live_exit",
            body: format!("{prefix} {iteration} return index * 10 + total;"),
            recoveries: 1,
            expected: [0, 11, 23, 23, 23],
        },
        Case {
            name: "outer_suffix",
            body: format!("{prefix} if limit > 0 {{ {iteration} }} return index * 10 + total;"),
            recoveries: 1,
            expected: [0, 11, 23, 23, 23],
        },
        Case {
            name: "parent_backedge",
            body: format!("{prefix} {}", nested.replace("CHILD", child)),
            recoveries: 1,
            expected: [0, 1, 3, 6, 10],
        },
        Case {
            name: "branch_helper_output",
            body: format!(
                "{prefix} {}",
                nested.replace("CHILD", &format!("if outer > 0 {{ {child} }}"))
            ),
            recoveries: 1,
            expected: [0, 1, 3, 6, 10],
        },
        Case {
            name: "dead_mixed_exit",
            body: format!(
                "{prefix} {} return total;",
                iteration.replace(
                    "let total = total + index;",
                    "if index == 1 { continue; } let total = total + index;"
                )
            ),
            recoveries: 0,
            expected: [0, 0, 2, 2, 2],
        },
    ]
}

pub fn source(body: &str, limit: usize) -> String {
    format!(
        "mod cpu Main {{
        @noinline
        fn work(limit: i64) -> i64 {{ {body} }}
        fn main() -> i64 {{ return work({limit}); }}
    }}"
    )
}
