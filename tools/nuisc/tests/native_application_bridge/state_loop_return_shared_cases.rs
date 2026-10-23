#[path = "state_loop_return_parent_mixed_cases.rs"]
mod mixed;
#[allow(unused_imports)]
pub use mixed::*;

#[path = "state_loop_return_shared_loop_calls.rs"]
pub mod loop_calls;

#[path = "state_loop_return_shared_branch_calls.rs"]
pub mod branch_calls;

#[path = "state_loop_return_shared_bound_calls.rs"]
pub mod bound_calls;

pub fn variants() -> Vec<Variant> {
    mixed::variants()
        .into_iter()
        .filter(|v| v.kind == Kind::Record)
        .collect()
}

// Keep the established independent source oracle and observable work. This
// program carries one real mutable record instead of four scalar locals.
pub fn source(variant: Variant) -> String {
    assert_eq!(variant.kind, Kind::Record);
    let text = mixed::source(variant);
    let begin = text.find("        let mut current = total;").unwrap();
    let end = text.find("    fn start(").unwrap();
    let mut body = text[begin..end].to_owned();
    let init = "let mut current = total;\n        let mut choice = flag;\n        let mut narrow = gain;\n        let mut wide = marker;";
    assert_eq!(body.matches(init).count(), 1);
    body = body.replace(
        init,
        "let mut carry = State { result: flag, total: total, gain: gain, marker: marker };",
    );
    for (binding, expressions) in [
        (
            "current",
            vec![
                "current + outer",
                "current + index + 1",
                "current + proof",
                "current + check",
                "current + 2",
            ],
        ),
        ("choice", vec!["choice != true"]),
        ("narrow", vec!["-narrow"]),
        ("wide", vec!["-wide"]),
    ] {
        for expression in expressions {
            let field = match binding {
                "current" => "total",
                "choice" => "result",
                "narrow" => "gain",
                "wide" => "marker",
                _ => unreachable!(),
            };
            let fields = [
                ("result", "choice"),
                ("total", "current"),
                ("gain", "narrow"),
                ("marker", "wide"),
            ]
            .map(|(key, old)| format!("{key}: {}", if key == field { expression } else { old }))
            .join(", ");
            let old = format!("{binding} = {expression};");
            assert!(body.contains(&old), "{old}");
            body = body.replace(&old, &format!("carry = State {{ {fields} }};"));
        }
    }
    for (old, new) in [
        ("current", "carry.total"),
        ("choice", "carry.result"),
        ("narrow", "carry.gain"),
        ("wide", "carry.marker"),
    ] {
        body = body.replace(old, new);
    }
    let flip = "carry = State { result: carry.result != true";
    assert_eq!(body.matches(flip).count(), 3);
    for index in 0..3 {
        body = body.replacen(flip, &format!("let flipped_{index} = carry.result; carry = State {{ result: flipped_{index} != true"), 1);
    }
    format!("{}{}{}", &text[..begin], body, &text[end..])
}

pub const INIT_PRINTS: [i64; 2] = [701, 702];

pub fn source_with_prefix(variant: Variant) -> String {
    let source = source(variant);
    let init = "let mut carry = State { result: flag, total: total, gain: gain, marker: marker };";
    assert_eq!(source.matches(init).count(), 1);
    source.replace(init, &format!("print(701); {init} print(702);"))
}

pub fn with_prefix(mut trace: Vec<i64>, printed: bool) -> Vec<i64> {
    if printed {
        // start/step prints 5 before invoking work. Keep that outer effect first.
        assert_eq!(trace.first(), Some(&5));
        trace.splice(1..1, INIT_PRINTS);
    }
    trace
}

// These are real outer carries but read-only inner captures. Publishing a
// return into one would introduce an unproved inner write before its read.
pub fn source_with_alternatives(variant: Variant, rejected: usize) -> String {
    assert!(rejected > 0);
    let mut source = source_with_prefix(variant);
    let init = "let mut carry = State { result: flag, total: total, gain: gain, marker: marker };";
    let mut declarations = String::new();
    let mut outer = String::new();
    let mut inner = String::new();
    for index in 0..rejected {
        declarations.push_str(&format!("let mut candidate_{index} = State {{ result: flag, total: total, gain: gain, marker: marker }}; "));
        outer.push_str(&format!("candidate_{index} = candidate_{index}; "));
        inner.push_str(&format!("let snapshot_{index} = candidate_{index}.total; "));
    }
    assert_eq!(source.matches("print(8);").count(), 1);
    assert_eq!(source.matches("print(10);").count(), 1);
    source = source.replace(init, &format!("{declarations}{init}"));
    source = source.replace("print(8);", &format!("{outer}print(8);"));
    source.replace("print(10);", &format!("{inner}print(10);"))
}

pub fn with_reassignments(source: String) -> String {
    assert_eq!(source.matches("print(702);").count(), 1);
    source.replace("print(702);", "carry = State { result: carry.result, total: carry.total + 3, gain: -carry.gain, marker: -carry.marker }; print(703); carry = State { result: carry.result, total: carry.total - 3, gain: -carry.gain, marker: -carry.marker }; print(702);")
}

pub fn with_helper_calls(source: String) -> String {
    let mut source = with_reassignments(source);
    for (sign, delta) in [("+", 3), ("-", -3)] {
        let old = format!("carry = State {{ result: carry.result, total: carry.total {sign} 3, gain: -carry.gain, marker: -carry.marker }};");
        assert_eq!(source.matches(&old).count(), 1);
        let (argument_root, callee_root) = if delta == 3 {
            ("1", "1")
        } else {
            ("root + 7", "root + 8")
        };
        source = source.replace(&old, &format!("carry = prefix_record(prefix_seed(carry), prefix_delta({delta}, {argument_root}), prefix_gate({callee_root}));"));
    }
    assert_eq!(source.matches("    @noinline fn work(").count(), 1);
    source.replace(
        "    @noinline fn work(",
        "
    @noinline fn prefix_seed(seed: State) -> State { print(710); return seed; }
    @noinline fn prefix_delta(delta: i64, root: i64) -> i64 {
        print(711); return delta + 0 / root;
    }
    @noinline fn prefix_gate(root: i64) -> i64 { print(714); return root; }
    @noinline fn prefix_record(seed: State, delta: i64, root: i64) -> State {
        print(712); let check = 0 / root; print(713);
        return State { result: seed.result, total: seed.total + delta + check,
            gain: -seed.gain, marker: -seed.marker };
    }
    @noinline fn work(",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    Plain,
    Prefix,
    Candidates,
    Reassigned,
    HelperCalls,
    LoopHelperCalls,
    BranchHelperCalls,
    BranchBoundCalls,
}

impl SourceKind {
    pub fn helper_entry_limit(self) -> u64 {
        match self {
            Self::BranchHelperCalls => 128,
            Self::BranchBoundCalls => 160,
            _ => 100,
        }
    }

    pub fn source(self, variant: Variant) -> String {
        match self {
            Self::Plain => source(variant),
            Self::Prefix => source_with_prefix(variant),
            Self::Candidates => source_with_alternatives(variant, 1),
            Self::Reassigned => with_reassignments(source_with_prefix(variant)),
            Self::HelperCalls => with_helper_calls(source_with_prefix(variant)),
            Self::LoopHelperCalls => loop_calls::source(variant),
            Self::BranchHelperCalls => branch_calls::source(variant),
            Self::BranchBoundCalls => bound_calls::source(variant),
        }
    }

    pub fn trace(self, trace: Vec<i64>) -> Vec<i64> {
        assert!(
            !matches!(
                self,
                Self::LoopHelperCalls | Self::BranchHelperCalls | Self::BranchBoundCalls
            ),
            "loop effects require semantic oracle hooks"
        );
        let mut trace = with_prefix(trace, self != Self::Plain);
        if matches!(self, Self::Reassigned | Self::HelperCalls) {
            assert_eq!(&trace[..3], &[5, 701, 702]);
            trace.insert(2, 703);
        }
        if self == Self::HelperCalls {
            trace.splice(3..3, [710, 711, 714, 712, 713]);
            trace.splice(2..2, [710, 711, 714, 712, 713]);
        }
        trace
    }

    pub fn execute(
        self,
        call: Call,
        observed: bool,
        budget: Option<u64>,
    ) -> Result<(State, Vec<i64>), Vec<i64>> {
        if self == Self::LoopHelperCalls {
            return loop_calls::execute(call, observed, budget);
        }
        if self == Self::BranchHelperCalls {
            return branch_calls::execute(call, observed, budget);
        }
        if self == Self::BranchBoundCalls {
            return bound_calls::execute(call, observed, budget);
        }
        call.execute(observed, budget)
            .map(|(state, trace)| (state, self.trace(trace)))
            .map_err(|trace| self.trace(trace))
    }

    pub fn sequences(self, variant: Variant) -> Vec<Vec<Call>> {
        let mut rows = sequences(variant);
        if matches!(
            self,
            Self::LoopHelperCalls | Self::BranchHelperCalls | Self::BranchBoundCalls
        ) {
            for (gain, marker) in [(8, 9), (10, 8)] {
                let mut call = base(variant);
                call.input.gain = gain;
                call.input.marker = marker;
                call.select_first(true);
                rows.push(sequence(call));
            }
        }
        if self == Self::BranchBoundCalls {
            rows.push(sequence(bound_calls::state_dependent(variant)));
            rows = rows.into_iter().map(bound_calls::sequence).collect();
        }
        rows
    }

    pub fn next(self, call: Call) -> State {
        self.execute(call, false, None).unwrap().0
    }
}
