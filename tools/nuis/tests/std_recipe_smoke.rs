use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_std_recipe_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

const PRINT_RECIPE_CASES: &[(&str, &str, bool)] = &[
    (
        "std_line_input",
        "../../stdlib/std/line_input_recipe.ns",
        true,
    ),
    (
        "pixelmagic_invert",
        "../../stdlib/pixelmagic/core/invert_recipe.ns",
        false,
    ),
    (
        "witsage_knn",
        "../../stdlib/witsage/core/knn_recipe.ns",
        false,
    ),
    (
        "std_net_result_enum",
        "../../stdlib/std/net_result_enum_recipe.ns",
        true,
    ),
];

#[test]
fn print_style_official_recipes_build_and_run() {
    for (label, source, has_host_extern) in PRINT_RECIPE_CASES {
        let output_dir = temp_dir(label);
        let output_dir_text = output_dir.display().to_string();

        let build = run_nuis(&["build", source, &output_dir_text]);
        assert_success(&build, &format!("nuis build {source}"));
        assert!(String::from_utf8_lossy(&build.stdout).contains("ready_to_run: true"));

        let json = run_nuis(&["run-artifact", "--json", &output_dir_text]);
        assert_success(&json, &format!("nuis run-artifact --json {source}"));
        if *has_host_extern {
            let json_stdout = String::from_utf8_lossy(&json.stdout);
            assert!(json_stdout.contains("\"runtime_host_yir_attempted\":false"));
            assert!(json_stdout.contains(
                "\"runtime_host_yir_skip_reason\":\"host_ffi_externs_present_or_no_yir\""
            ));
        }

        let run = run_nuis(&["run-artifact", &output_dir_text]);
        assert_success(&run, &format!("nuis run-artifact {source}"));
        assert!(String::from_utf8_lossy(&run.stdout).contains("exit_status: 0"));
    }
}
