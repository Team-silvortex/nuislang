pub const SOURCE: &str = include_str!("return_i64_i32_private_pair.ns");
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
    pub child_seed: i32,
    pub first: bool,
    pub second: bool,
    pub owner: i64,
    pub pair: Option<(i64, i64)>,
    pub result: bool,
    pub trace: Vec<i64>,
}

impl Case {
    // Independent signed-width source replay never uses the compiler carrier or codec.
    pub fn new(seed: i64, child_seed: i32, first: bool, second: bool) -> Self {
        let parent = |value: i64, child: i32| if child < 0 { !value } else { value };
        let mut trace = vec![99, 90, 87];
        let mut local = seed;
        let mut pair = None;
        if first && second {
            let mut child = child_seed;
            trace.extend([91, 92]);
            if child < 0 {
                local = parent(local, child);
                trace.push(95);
                child = !child;
                trace.push(93);
                if child == 0 {
                    child = !child;
                    trace.push(93);
                    local = parent(local, child);
                    trace.push(95);
                }
            } else {
                child = !child;
                trace.push(93);
                local = parent(local, child);
                trace.push(95);
                if child == -1 {
                    local = parent(local, child);
                    trace.push(95);
                    child = !child;
                    trace.push(93);
                }
            }
            pair = Some((local, i64::from(child)));
            trace.push(96);
            if child < 0 {
                local = parent(local, child);
                trace.push(95);
            }
            let _seal = child;
            trace.push(83);
        }
        trace.extend([97, 79]);
        let equal = local == seed;
        Self {
            seed,
            child_seed,
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
        Self::new(self.seed, self.child_seed, !self.first, !self.second)
    }
    pub fn values(&self) -> Vec<String> {
        let mut values = vec![self.first.to_string(), self.second.to_string()];
        values.extend(STAGES.map(|stage| {
            if self.trace.contains(&stage) {
                "1"
            } else {
                "0"
            }
            .into()
        }));
        values
    }
    pub fn words(&self) -> Vec<i64> {
        let mut values = vec![i64::from(self.first), i64::from(self.second)];
        values.extend(STAGES.map(|stage| i64::from(self.trace.contains(&stage))));
        values
    }
    pub fn open_words(&self) -> Vec<i64> {
        [self.seed, i64::from(self.child_seed)]
            .into_iter()
            .chain(self.words())
            .collect()
    }
    pub fn state_words(&self) -> Vec<i64> {
        self.open_words()
            .into_iter()
            .chain([i64::from(self.result)])
            .collect()
    }
    pub fn state_text(&self, phase: &str) -> String {
        let mut fields = vec![
            format!("seed: {}", self.seed),
            format!("child_seed: {}i32", self.child_seed),
        ];
        fields.extend(
            FIELDS
                .into_iter()
                .zip(self.values())
                .map(|(n, v)| format!("{n}: {v}")),
        );
        fields.push(format!("result: {}", self.result));
        format!("{phase}:State{{{}}}", fields.join(", "))
    }
    pub fn open_args(&self) -> String {
        format!(
            "{},{},{}",
            self.seed,
            self.child_seed,
            self.values().join(",")
        )
    }
    pub fn bad_values(&self, index: usize) -> Vec<String> {
        let mut values = self.values();
        assert_eq!(values[index + 2], "1");
        values[index + 2] = "0".into();
        values
    }
    fn observe_until(&self, count: usize) -> Vec<i64> {
        // Event positions are independent of field words, including diagnostic sentinels.
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

#[allow(dead_code)] // Native/reference Cartesian inventory; CLI uses a compact extreme matrix.
pub fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for seed in [
        i64::MIN,
        -4_294_967_297,
        -70002,
        -70001,
        -97,
        -96,
        -1,
        0,
        1,
        96,
        97,
        4_294_967_296,
        i64::MAX,
    ] {
        for child_seed in [
            i32::MIN,
            -70002,
            -70001,
            -97,
            -96,
            -1,
            0,
            1,
            96,
            97,
            i32::MAX,
        ] {
            for first in [false, true] {
                for second in [false, true] {
                    cases.push(Case::new(seed, child_seed, first, second));
                }
            }
        }
    }
    cases
}
#[allow(dead_code)] // Compact CLI inventory is unused by the full native/reference matrix.
pub fn cli_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (seed, child_seed) in [
        (i64::MIN, -1),
        (i64::MAX, 0),
        (-70002, i32::MIN),
        (-70001, i32::MAX),
    ] {
        for first in [false, true] {
            for second in [false, true] {
                cases.push(Case::new(seed, child_seed, first, second));
            }
        }
    }
    cases
}
pub fn selected() -> Vec<Case> {
    STAGES
        .into_iter()
        .map(|stage| {
            if stage == 83 || stage == 97 {
                // Both source marker 96 and diagnostic AFTER=-70002 are valid fields.
                Case::new(70001, -97, true, true)
            } else {
                cases()
                    .into_iter()
                    .find(|case| case.trace.contains(&stage))
                    .unwrap()
            }
        })
        .collect()
}
