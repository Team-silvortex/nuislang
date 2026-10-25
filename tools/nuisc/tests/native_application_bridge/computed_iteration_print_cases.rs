#[allow(dead_code)]
#[path = "origin_record_join_cases.rs"]
mod prior;
pub const SOURCE: &str = include_str!("computed_iteration_print.ns");
pub use prior::{ENTRY_LIMIT, LAST_STATE, LOOP_LIMIT};
#[allow(unused_imports)]
pub use prior::{ENTRY_MARKERS, OBSERVERS};

#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub input: prior::Case,
}
impl Case {
    pub fn words(self) -> Vec<i64> {
        self.input.words()
    }
    pub fn args(self) -> String {
        self.input.args()
    }
    fn printed(self, trace: Vec<i64>) -> Vec<i64> {
        let mut output = Vec::new();
        for value in trace {
            // The independent predecessor emits 84 only after checked post-work succeeds.
            if value == 84 {
                output.push(
                    1i64.checked_div(self.input.post_div)
                        .expect("completed post-work"),
                );
            }
            output.push(value);
        }
        output
    }
    pub fn execute(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.input
            .execute(previous)
            .map(|(state, trace)| (state, self.printed(trace)))
            .map_err(|trace| self.printed(trace))
    }
    pub fn execute_native(self, previous: Option<i64>) -> Result<(i64, Vec<i64>), Vec<i64>> {
        self.input
            .execute_native(previous)
            .map(|(state, trace)| (state, self.printed(trace)))
            .map_err(|trace| self.printed(trace))
    }
}
pub fn prefix() -> Vec<Case> {
    prior::prefix()
        .into_iter()
        .map(|input| Case { input })
        .collect()
}
pub fn healthy() -> Vec<Case> {
    prior::healthy()
        .into_iter()
        .map(|input| Case { input })
        .collect()
}
pub fn failures() -> Vec<Case> {
    prior::failures()
        .into_iter()
        .map(|input| Case { input })
        .collect()
}
pub fn native_failures() -> Vec<Case> {
    prior::native_failures()
        .into_iter()
        .map(|input| Case { input })
        .collect()
}
