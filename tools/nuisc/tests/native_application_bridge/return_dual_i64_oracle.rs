pub const SOURCE: &str = include_str!("return_dual_i64_private_pair.ns");
pub const STAGES: [i64; 9] = [90, 87, 91, 92, 95, 93, 96, 83, 97];
pub const FIELDS: [&str; 11] = [
    "first",
    "second",
    "outer",
    "init",
    "child_init",
    "cond",
    "parent_rhs",
    "child_rhs",
    "after",
    "unused",
    "answer",
];

#[derive(Clone)]
pub struct Case {
    pub seed: i64,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub pair: Option<(i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}

impl Case {
    // Imperative source replay never uses the compiler's private pair codec.
    pub fn new(seed: i64, first: bool, second: bool) -> Self {
        let mut trace = vec![99, 90, 87];
        let mut local = seed;
        let mut pair = None;
        if first && second {
            let mut child = !seed;
            trace.extend([91, 92]);
            if child < 0 {
                local ^= child;
                trace.push(95);
                child = !child;
                trace.push(93);
                if child == 0 {
                    child = !child;
                    trace.push(93);
                    local ^= child;
                    trace.push(95);
                }
            } else {
                child = !child;
                trace.push(93);
                local ^= child;
                trace.push(95);
                if child == -1 {
                    local ^= child;
                    trace.push(95);
                    child = !child;
                    trace.push(93);
                }
            }
            pair = Some((local, child));
            trace.push(96);
            if child < 0 {
                local ^= child;
                trace.push(95);
            }
            let _seal = child;
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = local == seed;
        Self {
            seed,
            first,
            second,
            owner: local,
            pair,
            result: if first {
                equal && first
            } else {
                equal || second
            },
            trace,
        }
    }

    pub fn previous(&self) -> Self {
        Self::new(self.seed, !self.first, !self.second)
    }
    pub fn values(&self) -> Vec<String> {
        let mut args = vec![self.first.to_string(), self.second.to_string()];
        args.extend(STAGES.map(|stage| {
            if self.trace.contains(&stage) {
                "1"
            } else {
                "0"
            }
            .into()
        }));
        args
    }
    pub fn words(&self) -> Vec<i64> {
        let mut args = vec![i64::from(self.first), i64::from(self.second)];
        args.extend(STAGES.map(|stage| i64::from(self.trace.contains(&stage))));
        args
    }
    pub fn state_words(&self) -> Vec<i64> {
        std::iter::once(i64::from(self.seed))
            .chain(self.words())
            .chain([i64::from(self.result)])
            .collect()
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut fields = vec![format!("seed: {}", self.seed)];
        fields.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(name, value)| format!("{name}: {value}")),
        );
        fields.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", fields.join(", "))
    }
    pub fn open_args(&self) -> String {
        format!("{},{}", self.seed, self.values().join(","))
    }
    pub fn bad_values(&self, stage: usize) -> Vec<String> {
        let mut values = self.values();
        assert_eq!(values[stage + 2], "1");
        values[stage + 2] = "0".into();
        values
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        // Source-event positions cannot be confused with observed field values.
        let mut output = Vec::new();
        for stage in self.trace.iter().take(count) {
            if *stage == 96 {
                let (owner, child) = self.pair.unwrap();
                output.extend([owner, child]);
            } else if *stage == 97 {
                output.push(self.owner);
            }
            output.push(*stage);
        }
        output
    }
    pub fn observed(&self) -> Vec<i64> {
        self.observe_until(self.trace.len())
    }
    pub fn before_failure(&self, index: usize) -> Vec<i64> {
        let position = self.trace.iter().position(|v| *v == STAGES[index]).unwrap();
        self.observe_until(position)
    }
}

pub fn cases() -> Vec<Case> {
    [i64::MIN, -4_294_967_297, -1, 0, 1, 4_294_967_296, i64::MAX]
        .into_iter()
        .flat_map(|seed| {
            [false, true].into_iter().flat_map(move |first| {
                [false, true]
                    .into_iter()
                    .map(move |second| Case::new(seed, first, second))
            })
        })
        .collect()
}

pub fn selected() -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| match stage {
            // Late faults must retain fields whose values equal trace markers.
            83 => Case::new(96, true, true),
            97 => Case::new(97, true, true),
            _ => cases()
                .into_iter()
                .find(|case| case.trace.contains(&stage))
                .unwrap(),
        })
        .collect()
}
