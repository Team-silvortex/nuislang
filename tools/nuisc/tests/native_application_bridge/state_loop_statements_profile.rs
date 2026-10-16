use super::cases;
#[path = "state_loop_effects_profile.rs"]
#[allow(dead_code)]
mod calls;

pub const LOOP_LIMIT: u64 = 6;
pub const WORK_OBSERVER: &str = calls::WORK_OBSERVER;

pub fn source_variants() -> [String; 4] {
    std::array::from_fn(|variant| {
        let source = cases::SOURCE.to_owned();
        let selected = if variant % 2 == 0 {
            "let selected = gate != even; if choose(selected, proof) { SELECTED } else { current = current + 1; }"
        } else {
            "let selected = gate == even; if choose(selected, proof) { current = current + 1; } else { SELECTED }"
        };
        let selected = selected.replace("SELECTED", "print(19); let local = refine(current, 1 / (index - hole), index); print(20); current = local + 1;");
        let work = format!("current = current + 0; let even = current % 2 == 0; let proof = 1 / root; print(10); {selected}");
        let original = "current = update(current, gate, root, hole, index);";
        assert_eq!(source.matches(original).count(), 1);
        let source = source.replace(original, &work);
        // A real second counted loop, with no business carry or artificial
        // source helper. Both loops debit the same callback work budget.
        let original = "        let mut index = 0;";
        assert_eq!(source.matches(original).count(), 1);
        let source = source.replace(original, "        let mut echo = 0;\n        while echo < trips { print(7); echo = echo + 1; }\n        let mut index = 0;");
        if variant >= 2 {
            source
                .replace("refine(current,", "refine(current + 0,")
                .replace("answer_work(total,", "answer_work(total + 0,")
        } else {
            source
        }
    })
}

fn prefix(mut trace: Vec<i64>, call: cases::Invocation) -> Vec<i64> {
    if !trace.is_empty() {
        trace.splice(
            1..1,
            std::iter::repeat_n(7, call.input.trips.max(0) as usize),
        );
    }
    trace
}

pub fn trace(call: cases::Invocation, observed: bool) -> Vec<i64> {
    prefix(calls::trace(call, observed), call)
}

pub fn before_failure(call: cases::Invocation, bad: Option<usize>) -> Vec<i64> {
    prefix(calls::before_failure(call, bad), call)
}
