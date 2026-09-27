use super::*;

#[path = "../../../nuisc/tests/native_application_bridge/scoped_record_fixture.rs"]
mod fixture;

#[derive(Clone, Copy)]
enum Shape {
    Scoped,
    Branch,
    Mixed,
    Signed,
    Float,
    Double,
}

fn verify_runs(project: &Project, output: &Path, shape: Shape) -> Vec<Vec<String>> {
    let branching = matches!(shape, Shape::Branch);
    let mixed = matches!(
        shape,
        Shape::Mixed | Shape::Signed | Shape::Float | Shape::Double
    );
    let signed = matches!(shape, Shape::Signed);
    let float = matches!(shape, Shape::Float);
    let double = matches!(shape, Shape::Double);
    [-3_i64, 0, 5]
        .into_iter()
        .map(|seed| {
            let run = success(project.command(
                "run-artifact",
                output,
                &[
                    "--native-session",
                    "counter",
                    "--open-args",
                    &seed.to_string(),
                    "--event-args",
                    "",
                    "--event-args",
                    "",
                    "--close-args",
                    "",
                ],
            ));
            assert!(run.stdout.is_empty());
            assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
            let mut words = (0..64).map(|i| seed + i).collect::<Vec<_>>();
            if mixed {
                words[63] = i64::from(seed > 0);
            }
            if signed {
                words[62] = (2147483646_i64 + seed) as i32 as i64;
            }
            if float {
                words[62] = i64::from(1.5_f32.to_bits());
            }
            if double {
                words[62] = (-1.5_f64).to_bits() as i64;
            }
            let expected = ["open", "event", "event", "close"]
                .into_iter()
                .map(|phase| {
                    if phase == "event" {
                        let trips = words[1].max(0);
                        let delta = if branching {
                            trips.min(2) + 2 * (trips - 2).max(0)
                        } else {
                            trips
                        };
                        for (index, word) in words.iter_mut().enumerate() {
                            if mixed && index == 63 {
                                *word ^= trips % 2;
                            } else if signed && index == 62 {
                                *word = (*word as i32).wrapping_add(delta as i32) as i64;
                            } else if float && index == 62 {
                                *word = i64::from(
                                    (f32::from_bits(*word as u32) + delta as f32 * 0.5).to_bits(),
                                );
                            } else if double && index == 62 {
                                *word = (f64::from_bits(*word as u64) + delta as f64 * 0.5)
                                    .to_bits() as i64;
                            } else {
                                *word += delta;
                            }
                        }
                        if !branching {
                            words[0] += trips * (trips + 1) / 2;
                        }
                    }
                    let fields = words
                        .iter()
                        .enumerate()
                        .map(|(i, value)| {
                            if mixed && i == 63 {
                                format!("f{i}: {}", *value != 0)
                            } else if signed && i == 62 {
                                format!("f{i}: {value}i32")
                            } else if float && i == 62 {
                                format!("f{i}: {}f32", f32::from_bits(*value as u32))
                            } else if double && i == 62 {
                                format!("f{i}: {}f64", f64::from_bits(*value as u64))
                            } else {
                                format!("f{i}: {value}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{phase}:State{{{fields}}}")
                })
                .collect::<Vec<_>>();
            assert_eq!(states(&run), expected);
            expected
        })
        .collect()
}

#[test]
fn native_scoped_record_inputs_cache_and_restore_the_full_mapping_without_sources() {
    check(Shape::Scoped);
}

#[test]
fn native_branch_record_inputs_cache_and_restore_guarded_branches_without_sources() {
    check(Shape::Branch);
}

#[test]
fn native_mixed_record_carries_cache_and_restore_typed_maps_without_sources() {
    check(Shape::Mixed);
}

#[test]
fn native_i32_record_carries_cache_and_restore_signed_maps_without_sources() {
    check(Shape::Signed);
}

#[test]
fn native_f32_record_carries_cache_and_restore_bit_maps_without_sources() {
    check(Shape::Float);
}

#[test]
fn native_f64_record_carries_cache_and_restore_full_width_maps_without_sources() {
    check(Shape::Double);
}

fn check(shape: Shape) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let branching = matches!(shape, Shape::Branch);
    let mut source = fixture::source(64, branching).replace("if i == 3 { break; }", "");
    if matches!(
        shape,
        Shape::Mixed | Shape::Signed | Shape::Float | Shape::Double
    ) {
        source = source
            .replace("f63: i64", "f63: bool")
            .replace("f63: seed + 63", "f63: seed > 0")
            .replace("f63: value.f63 + 1", "f63: !value.f63");
    }
    if matches!(shape, Shape::Signed) {
        source = source
            .replace("f62: i64", "f62: i32")
            .replace("f62: seed + 62", "f62: i32_from_i64(2147483646 + seed)")
            .replace("f62: value.f62 + 1", "f62: value.f62 + i32_from_i64(1)");
    }
    if matches!(shape, Shape::Float) {
        source = source
            .replace("f62: i64", "f62: f32")
            .replace("f62: seed + 62", "f62: 1.5")
            .replace("f62: value.f62 + 1", "f62: value.f62 + 0.5");
    }
    if matches!(shape, Shape::Double) {
        source = source
            .replace("f62: i64", "f62: f64")
            .replace("f62: seed + 62", "f62: -1.5")
            .replace("f62: value.f62 + 1", "f62: value.f62 + 0.5");
    }
    let project = Project::new(&source);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    assert!(llvm.lines().any(|line| line.starts_with("define ")
        && line.contains("@nuis_fn___nuis_scalar_iteration_")
        && line.split_once('(').unwrap().1.contains("[64 x i64] %arg")));
    if branching {
        assert!(llvm.lines().any(|line| line.starts_with("define ")
            && line.contains("@nuis_fn___nuis_buffer_branch_")
            && line.split_once('(').unwrap().1.contains("[64 x i64] %arg")));
    }
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    let expected = verify_runs(&project, &output, shape);
    assert!(project.build(None).contains("compile_cache: hit"));
    assert_eq!(verify_runs(&project, &output, shape), expected);
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &["--native-session", "counter", "--open-args", ""],
    ));
    fs::write(output.join(&llvm_name), format!("{llvm}\n")).unwrap();
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &["--native-session", "counter", "--open-args", "5"],
    ));
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[restored.to_str().unwrap()],
    ));
    assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm);
    assert_eq!(verify_runs(&project, &restored, shape), expected);
}
