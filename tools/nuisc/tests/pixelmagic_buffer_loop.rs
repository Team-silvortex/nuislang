use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const LIBRARY: &str = include_str!("../../../stdlib/pixelmagic/lib/pixels.ns");

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compare_pixels(
    width: usize,
    height: usize,
    tile: usize,
    phase: usize,
    start: usize,
    end: usize,
    count_result: bool,
) {
    let count = width * height;
    let reads = (0..count)
        .map(|index| format!("print(pixels[{index}]);"))
        .collect::<Vec<_>>()
        .join("\n");
    let call = if count_result {
        format!(
            "fill_checkerboard_region_red_count(pixels, {start}, {end}, {width}, {tile}, {phase})"
        )
    } else if start == 0 && end == count {
        format!("fill_checkerboard(pixels, {width}, {height}, {tile}, {phase})")
    } else {
        format!("fill_checkerboard_region(pixels, {start}, {end}, {width}, {tile}, {phase})")
    };
    let admission = if count_result {
        format!("let stats: CheckerboardStats = fill_checkerboard_region_stats(pixels, {start}, {end}, {width}, {tile}, {phase}); let red_count: i64 = stats.red_count; let filled: bool = red_count >= 0;")
    } else {
        format!("let filled: bool = {call};")
    };
    let count_output = if count_result {
        "print(red_count); print(stats.checksum);"
    } else {
        ""
    };
    let main = format!(
        "fn main() -> i64 {{ let pixels: ref Buffer = alloc_buffer({count}, 7); \
        {admission} {reads} {count_output} free(pixels); if filled {{ return 0; }} return 1; }}"
    );
    let source = format!("{}\n{main}\n}}", &LIBRARY[..LIBRARY.rfind('}').unwrap()]).replacen(
        "mod cpu PixelMagicPixels",
        "mod cpu Main",
        1,
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    for (name, instruction) in [
        ("checkerboard_is_red", "call_bool"),
        ("checkerboard_parity", "call_i64"),
        ("checkerboard_red_total", "call_i64"),
        ("checkerboard_row_start", "call_i64"),
        ("checkerboard_row_end", "call_i64"),
    ] {
        assert!(compiled
            .yir
            .functions
            .iter()
            .any(|function| function.name == name));
        assert!(
            compiled
                .yir
                .nodes
                .iter()
                .any(|node| node.op.instruction == instruction
                    && node.op.args.first().map(String::as_str) == Some(name)),
            "missing scalar helper {name}"
        );
    }
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.instruction == "loop_while_i64_effect"
            && node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries")
    }));
    assert!(
        compiled.yir.functions.iter().any(|function| {
            function.name.starts_with("__nuis_buffer_iteration_")
                && compiled.yir.nodes.iter().any(|node| {
                    function.body_nodes.contains(&node.name)
                        && node.op.instruction == "loop_while_i64_effect"
                        && node.op.args.get(6).map(String::as_str)
                            == Some("scoped_call_i64_carries")
                })
        }),
        "row helpers must retain nested pixel loops, not flatten or unroll them"
    );
    assert!(
        compiled.yir.functions.iter().any(|function| {
            function.name.starts_with("__nuis_buffer_branch_")
                && function.result.as_ref().is_some_and(|result| {
                    result.ty.starts_with("__nuis_scalar_carries_")
                        && compiled.yir.nodes.iter().any(|node| {
                            node.name == result.node
                                && node.op.instruction == "return_owned_struct"
                                && node.op.args.get(1).is_some_and(|layout| {
                                    layout.ends_with("{carry0:i64;carry1:i64;carry2:i64}")
                                })
                        })
                })
                && function.body_nodes.iter().any(|name| {
                    compiled.yir.nodes.iter().any(|node| {
                        &node.name == name
                            && node.op.instruction == "call_i64"
                            && node
                                .op
                                .args
                                .first()
                                .is_some_and(|callee| callee == "checkerboard_red_total")
                    })
                })
                && function.body_nodes.iter().any(|name| {
                    compiled
                        .yir
                        .nodes
                        .iter()
                        .any(|node| &node.name == name && node.op.instruction == "guard_return")
                })
        }),
        "pixel branches must retain a real control boundary and branch-local count"
    );
    assert!(
        compiled.yir.functions.iter().any(|function| {
            function.name.starts_with("__nuis_scalar_branch_")
                && function
                    .result
                    .as_ref()
                    .is_some_and(|result| result.ty == "bool")
                && function.body_nodes.iter().any(|name| {
                    compiled
                        .yir
                        .nodes
                        .iter()
                        .any(|node| &node.name == name && node.op.instruction == "guard_return")
                })
        }),
        "source color helper must retain its typed branch guard"
    );
    let mut expected = (0..count)
        .map(|index| {
            if !(start..end).contains(&index) {
                return 7;
            }
            let alternate = (index % width / tile + index / width / tile + phase) % 2;
            if alternate == 0 {
                4278190335_i64
            } else {
                4294901760
            }
        })
        .collect::<Vec<_>>();
    if count_result {
        let checksum: i64 = expected[start..end].iter().sum();
        let red_count = expected
            .iter()
            .filter(|pixel| **pixel == 4278190335)
            .count() as i64;
        expected.push(red_count);
        expected.push(checksum);
    }
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let actual = trace
        .events
        .iter()
        .filter(|event| event.contains("cpu.print "))
        .map(|event| {
            event
                .split_whitespace()
                .last()
                .unwrap()
                .parse::<i64>()
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "reference pixels");
    assert_eq!(
        trace
            .lane_steps
            .values()
            .flatten()
            .filter(|step| step.starts_with("cpu.store_at "))
            .count(),
        end - start
    );

    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = Artifacts(std::env::temp_dir().join(format!(
        "pixelmagic-buffer-loop-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    )));
    fs::create_dir(&directory.0).unwrap();
    let input = directory.0.join("main.ns");
    fs::write(&input, &source).unwrap();
    let artifact = nuisc::aot::write_and_link_with_source(
        &input,
        &directory.0.join("out"),
        &source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&compiled.llvm_ir),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    let run = Command::new(&artifact.binary_path).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let actual = String::from_utf8(run.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "native pixels");
}

#[test]
fn pixelmagic_loop_matches_every_reference_and_native_pixel() {
    for phase in 0..2 {
        compare_pixels(32, 24, 4, phase, 0, 768, false);
        compare_pixels(5, 3, 2, phase, 2, 14, false);
    }
    compare_pixels(3, 2, 1, 0, 3, 3, false);
}

#[test]
fn pixelmagic_red_count_matches_native_partial_and_empty_ranges() {
    for phase in 0..2 {
        compare_pixels(5, 3, 2, phase, 2, 14, true);
    }
    compare_pixels(3, 2, 1, 0, 3, 3, true);
}
