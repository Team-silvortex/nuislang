use super::*;

// Syntax and type proof have already ruled out logical gates in ordinary leaves.
pub(super) fn present(join: &Join<'_>) -> bool {
    let mut arms = vec![&join.yes, &join.no];
    while let Some(arm) = arms.pop() {
        if control_values::has_logical_record_fields(arm.value.as_ref()) {
            return true;
        }
        for local in &arm.locals {
            match local {
                Local::Binding { stmt, .. } => {
                    let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) = stmt else {
                        unreachable!()
                    };
                    if control_values::has_logical_record_fields(value) {
                        return true;
                    }
                }
                Local::Join { join, .. } => arms.extend([&join.yes, &join.no]),
                Local::Condition(_) => {}
            }
        }
    }
    false
}
