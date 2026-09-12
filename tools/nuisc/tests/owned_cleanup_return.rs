use std::{fs, path::PathBuf, process::Command};

const SOURCE: &str = r#"
mod cpu Main {
  struct Counts { value: i64, valid: bool }
  struct State { phase: i64, counts: Counts }

  fn guarded(seed: i64, early: bool) -> State {
    let buffer: ref Buffer = alloc_buffer(3, seed);
    let bytes: Bytes = copy_bytes(buffer);
    free(buffer);
    if early {
      drop_bytes(bytes);
      return State { phase: seed + 1, counts: Counts { value: 7, valid: true } };
    }
    let length: i64 = bytes_len(bytes);
    drop_bytes(bytes);
    return State { phase: seed + 2, counts: Counts { value: length, valid: false } };
  }

  fn terminal(seed: i64, choose: bool) -> State {
    let buffer: ref Buffer = alloc_buffer(2, seed);
    let bytes: Bytes = copy_bytes(buffer);
    free(buffer);
    if choose {
      drop_bytes(bytes);
      return State { phase: seed + 3, counts: Counts { value: 17, valid: true } };
    } else {
      drop_bytes(bytes);
      return State { phase: seed + 4, counts: Counts { value: 19, valid: false } };
    }
  }

  fn report(state: State) -> i64 {
    print(state.phase);
    print(state.counts.value);
    let valid: i64 = if state.counts.valid { 1 } else { 0 };
    print(valid);
    return 0;
  }

  fn main() -> i64 {
    report(guarded(10, true));
    report(guarded(20, false));
    report(terminal(30, true));
    report(terminal(40, false));
    return 0;
  }
}
"#;

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn require_no_live_blobs(llvm: &str) -> String {
    let (helpers, entry) = llvm.split_once("define i64 @nuis_yir_entry()").unwrap();
    assert_eq!(entry.matches("  ret i64 ").count(), 1);
    let probe = concat!(
        "  %cleanup_live = call i64 @nuis_scheduler_owned_blob_live_count_get_v1()\n",
        "  %cleanup_empty = icmp eq i64 %cleanup_live, 0\n",
        "  br i1 %cleanup_empty, label %cleanup_ok, label %cleanup_leaked\n",
        "cleanup_leaked:\n  call void @llvm.trap()\n  unreachable\n",
        "cleanup_ok:\n  ret i64 "
    );
    format!(
        "{helpers}\ndeclare i64 @nuis_scheduler_owned_blob_live_count_get_v1()\n\ndefine i64 @nuis_yir_entry(){}",
        entry.replacen("  ret i64 ", probe, 1)
    )
}

#[test]
fn native_nested_aggregate_cleanup_returns_preserve_both_paths_and_release_blobs() {
    let compiled = nuisc::pipeline::compile_source(SOURCE).unwrap();
    for instruction in [
        "guard_drop_owned_bytes_return",
        "branch_drop_owned_bytes_return",
    ] {
        assert!(
            compiled
                .yir
                .nodes
                .iter()
                .any(|node| node.op.instruction == instruction),
            "missing {instruction}"
        );
    }
    for reversed in [false, true] {
        let mut module = compiled.yir.clone();
        if reversed {
            module.nodes.reverse();
            for function in &mut module.functions {
                function.body_nodes.reverse();
            }
        }
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&module),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let reference = trace
            .events
            .iter()
            .filter(|event| event.contains("cpu.print "))
            .map(|event| event.split_whitespace().last().unwrap())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let expected = "11\n7\n1\n22\n24\n0\n33\n17\n1\n44\n19\n0\n";
        assert_eq!(reference, expected, "reference cleanup return paths");
        let llvm = yir_lower_llvm::emit_module(&module).unwrap();
        assert!(!llvm.contains("deferred lowering"), "{llvm}");
        let llvm = require_no_live_blobs(&llvm);
        let directory = Artifacts(std::env::temp_dir().join(format!(
            "nuis-cleanup-return-{}-{reversed}",
            std::process::id()
        )));
        fs::create_dir(&directory.0).unwrap();
        let input = directory.0.join("main.ns");
        fs::write(&input, SOURCE).unwrap();
        let artifact = nuisc::aot::write_and_link_with_source(
            &input,
            &directory.0.join("out"),
            SOURCE,
            nuisc::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &module,
                llvm_ir: Some(&llvm),
            },
            &nuisc::aot::host_cpu_build_target(),
        )
        .unwrap();
        let run = Command::new(&artifact.binary_path).output().unwrap();
        assert!(
            run.status.success(),
            "native cleanup failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8(run.stdout).unwrap(), expected);
    }
}
