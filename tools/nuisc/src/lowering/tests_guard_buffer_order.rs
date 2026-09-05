use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn guard_buffer_reads_precede_recursive_call_snapshot_and_free() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn fill(buffer: ref Buffer, index: i64) -> bool {
            if index >= buffer.len { return true; }
            buffer[index] = index + 10;
            return fill(buffer, index + 1);
          }
          fn checked(buffer: ref Buffer, length: i64) -> bool {
            if buffer.len != length { return false; }
            return fill(buffer, 0);
          }
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 0);
            let filled: bool = checked(buffer, 3);
            let snapshot: Bytes = copy_bytes(buffer);
            free(buffer);
            drop_bytes(snapshot);
            if filled { return 0; }
            return 1;
          }
        }
        "#,
    )
    .unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    yir_verify::verify_module(&yir).expect("guard reads must be ordered before owner consumption");
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Effect
            && edge.from.starts_with("guard_return_")
            && yir
                .nodes
                .iter()
                .any(|node| node.name == edge.to && node.op.instruction == "call_bool")
    }));
    let source = crate::render::render_yir(&yir);
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &source,
        &yir_verify::default_registry(),
    )
    .expect("recursive guard must stop before another write or recursive call");
    assert_eq!(
        trace
            .lane_steps
            .values()
            .flatten()
            .filter(|step| step.starts_with("cpu.store_at "))
            .count(),
        3
    );
}

#[test]
fn pixelmagic_generator_rejects_invalid_inputs_without_writes_or_caller_return() {
    let library = include_str!("../../../../stdlib/pixelmagic/lib/pixels.ns");
    let end = library.rfind('}').unwrap();
    let source = format!(
        "{}\n{}\n}}",
        &library[..end],
        r#"
      fn main() -> i64 {
        let pixels: ref Buffer = alloc_buffer(4, 7);
        let zero: bool = fill_checkerboard(pixels, 0, 2, 1, 0);
        let size: bool = fill_checkerboard(pixels, 3, 2, 1, 0);
        let tile: bool = fill_checkerboard(pixels, 2, 2, 0, 0);
        let phase: bool = fill_checkerboard(pixels, 2, 2, 1, 2);
        let range: bool = fill_checkerboard_region(pixels, 0, 5, 2, 1, 0);
        let empty: bool = fill_checkerboard_region(pixels, 2, 2, 2, 1, 0);
        let unchanged: i64 = pixels[0] + pixels[1] + pixels[2] + pixels[3];
        free(pixels);
        print(unchanged);
        print(zero || size || tile || phase || range);
        print(empty);
        return 0;
      }
    "#
    );
    let source = source.replacen("mod cpu PixelMagicPixels", "mod cpu Main", 1);
    let artifacts = crate::pipeline::compile_source(&source).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&artifacts.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    assert_eq!(
        trace
            .lane_steps
            .values()
            .flatten()
            .filter(|step| step.starts_with("cpu.store_at "))
            .count(),
        0
    );
    let prints = trace
        .events
        .iter()
        .filter(|event| event.contains("cpu.print"))
        .collect::<Vec<_>>();
    assert_eq!(
        prints.len(),
        3,
        "helper guards must not return from their caller"
    );
    assert!(prints[0].ends_with("28"), "{prints:?}");
    assert!(prints[1].ends_with("false"), "{prints:?}");
    assert!(prints[2].ends_with("true"), "{prints:?}");
}
