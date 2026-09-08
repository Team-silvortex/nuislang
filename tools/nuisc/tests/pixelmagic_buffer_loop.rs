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
) {
    let count = width * height;
    let reads = (0..count)
        .map(|index| format!("print(pixels[{index}]);"))
        .collect::<Vec<_>>()
        .join("\n");
    let call = if start == 0 && end == count {
        format!("fill_checkerboard(pixels, {width}, {height}, {tile}, {phase})")
    } else {
        format!("fill_checkerboard_region(pixels, {start}, {end}, {width}, {tile}, {phase})")
    };
    let main = format!(
        "fn main() -> i64 {{ let pixels: ref Buffer = alloc_buffer({count}, 7); \
        let filled: bool = {call}; {reads} free(pixels); if filled {{ return 0; }} return 1; }}"
    );
    let source = format!("{}\n{main}\n}}", &LIBRARY[..LIBRARY.rfind('}').unwrap()]).replacen(
        "mod cpu PixelMagicPixels",
        "mod cpu Main",
        1,
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.instruction == "loop_while_i64_effect"
            && node.op.args.get(6).map(String::as_str) == Some("scoped_call")
    }));
    let expected = (0..count)
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
        compare_pixels(32, 24, 4, phase, 0, 768);
        compare_pixels(5, 3, 2, phase, 2, 14);
    }
    compare_pixels(3, 2, 1, 0, 3, 3);
}
