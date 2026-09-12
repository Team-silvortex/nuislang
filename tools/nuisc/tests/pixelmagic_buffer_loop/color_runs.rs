use super::*;

const RED: i64 = 4_278_190_335;
const BLUE: i64 = 4_294_901_760;
const GREEN: i64 = 4_278_255_360;

fn literal(value: i64) -> String {
    if value < 0 {
        format!("(0 - {})", value.unsigned_abs())
    } else {
        value.to_string()
    }
}

#[test]
fn recolor_run_matches_native_and_reference_boundaries_without_touching_the_tail() {
    let cases = [
        (1, 8, RED, GREEN), // Stops at the different pixel, not a later matching run.
        (4, 8, RED, GREEN), // The first pixel already differs.
        (1, 3, RED, GREEN), // The explicit range ends inside a matching run.
        (5, 7, RED, GREEN),
        (0, 8, BLUE, GREEN),
        (7, 8, BLUE, GREEN),
        (1, 8, RED, RED), // Equal colors still advance and count actual writes.
        (1, 8, RED, 0),
        (1, 8, RED, 4_294_967_295),
        (0, 0, RED, GREEN),
        (8, 8, RED, GREEN), // No speculative read at the Buffer length.
        (0, 8, 0, GREEN),
        (-1, 8, RED, GREEN),
        (4, 3, RED, GREEN),
        (0, 9, RED, GREEN),
        (9, 9, RED, GREEN),
        (0, 8, -1, GREEN),
        (0, 8, 4_294_967_296, GREEN),
        (1, 8, RED, -1),
        (1, 8, RED, 4_294_967_296),
        (8, 8, -1, GREEN), // Even an empty range validates both colors.
        (8, 8, RED, -1),
    ];
    let calls = cases
        .iter()
        .map(|&(start, end, expected, replacement)| {
            format!(
                "exercise({}, {}, {}, {});",
                literal(start),
                literal(end),
                literal(expected),
                literal(replacement)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let reads = (0..8)
        .map(|index| format!("print(pixels[{index}]);"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "{}\nfn exercise(start: i64, end: i64, expected: i64, replacement: i64) -> i64 {{
            let pixels: ref Buffer = alloc_buffer(8, {RED});
            pixels[0] = {BLUE}; pixels[4] = {BLUE}; pixels[7] = {BLUE};
            let stats: PixelRunStats = recolor_run(pixels, start, end, expected, replacement);
            print(stats.end); print(stats.written); print(stats.checksum);
            {reads} free(pixels); return 0;
        }}
        fn main() -> i64 {{ {calls} return 0; }}
        }}",
        &LIBRARY[..LIBRARY.rfind('}').unwrap()]
    )
    .replacen("mod cpu PixelMagicPixels", "mod cpu Main", 1);
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.instruction == "loop_while_i64_effect"
            && node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break")
            && node.op.args[9].ends_with("{carry0:i64;carry1:i64;carry2:i64}")
    }));

    let mut expected_output = Vec::new();
    let mut expected_writes = 3 * cases.len();
    for (start, end, expected, replacement) in cases {
        let mut pixels = vec![BLUE, RED, RED, RED, BLUE, RED, RED, BLUE];
        let valid = 0 <= start
            && start <= end
            && end <= 8
            && (0..=4_294_967_295).contains(&expected)
            && (0..=4_294_967_295).contains(&replacement);
        let mut index = start;
        let mut written = 0;
        if valid {
            while index < end && pixels[index as usize] == expected {
                pixels[index as usize] = replacement;
                written += 1;
                index += 1;
            }
        } else {
            index = -1;
        }
        expected_output.extend([index, written, written * replacement]);
        expected_output.extend(pixels);
        expected_writes += written as usize;
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
    assert_eq!(
        actual, expected_output,
        "reference run statistics and every pixel"
    );
    assert_eq!(
        trace
            .lane_steps
            .values()
            .flatten()
            .filter(|step| step.starts_with("cpu.store_at "))
            .count(),
        expected_writes,
        "invalid input and the nonmatching suffix must perform no writes"
    );

    let directory = Artifacts(
        std::env::temp_dir().join(format!("pixelmagic-color-runs-{}", std::process::id())),
    );
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
    assert_eq!(
        actual, expected_output,
        "native run statistics and every pixel"
    );
}
