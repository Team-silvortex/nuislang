use super::*;
use crate::lowering::buffer_loop_outline::conditional_returns::tests::native::run;

#[test]
fn conditional_return_typed_snapshots_execute_default_aot_paths_and_selected_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let mut variants = 0;
    for ty in ["i32", "f32", "f64"] {
        for (shape, constant, partial, index) in [
            ("computed", false, false, 1),
            ("computed", true, false, 2),
            ("shared", false, false, 0),
            ("literal", true, false, 4),
            ("shared", true, true, 5),
            ("computed", false, false, 6),
        ] {
            let text = source(ty, shape, constant, partial, INPUTS[index]);
            let (result, prints) = expected(partial, INPUTS[index]);
            let output = prints
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            run(
                &text
                    .replace("return 11;", "return 0;")
                    .replace("return 19;", "return 0;"),
                result.map(|_| output.as_str()),
            );
            variants += 1;
        }
    }
    assert_eq!(variants, 18);
}
