use super::{aggregate_carried::Case, aggregate_conditional::Fixture};

#[derive(Clone, Copy)]
pub(super) enum Gate {
    Above,
    Below,
    Equal,
    Unequal,
    Any,
    All,
}

impl Gate {
    fn source(self, state: &str) -> String {
        match self {
            Self::Above => format!("{state} > pivot"),
            Self::Below => format!("{state} < limit"),
            Self::Equal => format!("seed == {state}"),
            Self::Unequal => format!("seed != {state}"),
            Self::Any => format!("{state} > pivot || seed == {state}"),
            Self::All => format!("{state} < limit && seed != {state}"),
        }
    }

    fn evaluate(self, state: i64, case: Case, pivot: i64, count: &mut i64) -> bool {
        *count += 1;
        match self {
            Self::Above => state > pivot,
            Self::Below => state < case.limit,
            Self::Equal => case.seed == state,
            Self::Unequal => case.seed != state,
            Self::Any => state > pivot || Self::Equal.evaluate(state, case, pivot, count),
            Self::All => state < case.limit && Self::Unequal.evaluate(state, case, pivot, count),
        }
    }
}

pub(super) enum Tree {
    Leaf(bool),
    Branch(Gate, Box<Self>, Box<Self>),
}

impl Tree {
    pub(super) fn test(gate: Gate) -> Self {
        Self::branch(gate, Self::Leaf(true), Self::Leaf(false))
    }

    pub(super) fn branch(gate: Gate, selected: Self, skipped: Self) -> Self {
        Self::Branch(gate, Box::new(selected), Box::new(skipped))
    }

    pub(super) fn and(gate: Gate, rest: Self) -> Self {
        Self::branch(gate, rest, Self::Leaf(false))
    }

    pub(super) fn or(gate: Gate, rest: Self) -> Self {
        Self::branch(gate, Self::Leaf(true), rest)
    }

    pub(super) fn chain(depth: usize) -> Self {
        assert!(depth > 0);
        let mut tree = Self::test(Gate::Below);
        for level in 1..depth {
            tree = if level % 2 == 0 {
                Self::or(Gate::Unequal, tree)
            } else {
                Self::and(Gate::Above, tree)
            };
        }
        tree
    }

    pub(super) fn evaluate(&self, state: i64, case: Case, pivot: i64, count: &mut i64) -> bool {
        match self {
            Self::Leaf(selected) => *selected,
            Self::Branch(gate, selected, skipped) => {
                // Interpret the original nested decisions, not the collapsed predicate.
                if gate.evaluate(state, case, pivot, count) {
                    selected.evaluate(state, case, pivot, count)
                } else {
                    skipped.evaluate(state, case, pivot, count)
                }
            }
        }
    }

    fn render(&self, state: &str, update: &str, fallback: &str, omit_else: bool) -> String {
        match self {
            Self::Leaf(true) => update.to_owned(),
            Self::Leaf(false) => fallback.to_owned(),
            Self::Branch(gate, selected, skipped) => {
                let selected = selected.render(state, update, fallback, omit_else);
                let skipped = skipped.render(state, update, fallback, omit_else);
                let condition = gate.source(state);
                if omit_else && skipped.is_empty() {
                    format!("if {condition} {{ {selected} }}")
                } else {
                    format!("if {condition} {{ {selected} }} else {{ {skipped} }}")
                }
            }
        }
    }

    pub(super) fn source(&self, fixture: Fixture, empty_keep: bool, omit_else: bool) -> String {
        assert_eq!(fixture.compare, ">");
        assert!(!fixture.reversed);
        let mut source = fixture.source();
        for slot in 0..fixture.count {
            if slot > 0 && slot % 2 == 0 {
                continue;
            }
            let state = if slot == 0 {
                "index".into()
            } else {
                format!("carry{}", slot - 1)
            };
            let op = if slot % 3 == 2 { "*" } else { "+" };
            let update = format!("let carry{slot}: i64 = carry{slot} {op} {state};");
            let keep = slot != 0 || fixture.keep_first_else;
            let fallback = if keep {
                format!("let carry{slot}: i64 = carry{slot};")
            } else {
                format!("let carry{slot}: i64 = carry{slot} * {state};")
            };
            let original = format!("if {state} > pivot {{ {update} }} else {{ {fallback} }}");
            assert!(source.contains(&original));
            let rendered = self.render(
                &state,
                &update,
                if empty_keep && keep { "" } else { &fallback },
                omit_else,
            );
            source = source.replace(&original, &rendered);
        }
        source
    }
}
