use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Project(PathBuf);
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn project(source: &str) -> Project {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Project(std::env::temp_dir().join(format!(
        "nuis-yir-checkpoint-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&dir.0).unwrap();
    fs::write(dir.0.join("main.ns"), source).unwrap();
    fs::write(dir.0.join("nuis.toml"),
        "name = \"checkpoint\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n"
    ).unwrap();
    dir
}

#[test]
fn headless_checkpoint_preserves_registered_callback_state() {
    let source = r#"
mod cpu Main {
  struct Counter { count: i64 }
  fn start(seed: i64) -> Counter { return Counter { count: seed }; }
  fn advance(state: Counter, input: i64) -> Counter {
    return Counter { count: state.count + input };
  }
  fn step(state: Counter, input: i64) -> Counter {
    if input > 0 { return advance(state, input); }
    return state;
  }
  fn stop(state: Counter) -> Counter { return state; }
  fn main() { print(999); }
}
"#;
    let project = project(source);
    let resolved = nuisc::pipeline::resolve_compile_input(&project.0).unwrap();
    let checkpoint = resolved
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
        checkpoint.yir(),
        &registry,
        "counter",
        vec![yir_core::Value::Int(40)],
    )
    .unwrap();
    session.event(vec![yir_core::Value::Int(2)]).unwrap();
    session.close(vec![]).unwrap();
    session.completion_status().unwrap();
    let yir_core::Value::Struct(state) = session.state() else {
        panic!("missing counter state")
    };
    assert_eq!(state.fields[0].1, yir_core::Value::Int(42));
    drop(session);
    checkpoint.emit_llvm().unwrap();
}

#[test]
fn headless_checkpoint_executes_compound_while_callbacks() {
    let source = r#"
mod cpu Main {
  struct Counter { count: i64 }
  fn start(seed: i64) -> Counter { return Counter { count: seed }; }
  fn step(state: Counter, input: i64) -> Counter {
    let x: i64 = state.count;
    while x < 100 {
      let x: i64 = x + 1;
      if x > input && x < 50 { break; }
    }
    return Counter { count: x };
  }
  fn stop(state: Counter) -> Counter { return state; }
  fn main() { print(999); }
}
"#;
    let async_source = source
        .replace("fn step(", "async fn step(")
        .replace("let x: i64 = x + 1;", "let x: i64 = await increment(x);")
        .replace(
            "  fn stop(",
            "  async fn increment(x: i64) -> i64 { return x + 1; }\n  fn stop(",
        );
    for source in [source.to_owned(), async_source] {
        for (operator, cases) in [
            ("&&", [(40, 41, 42), (48, 49, 100), (100, 0, 100)]),
            ("||", [(40, 60, 41), (50, 52, 53), (100, 0, 100)]),
        ] {
            let fixture = project(&source.replace("&&", operator));
            let resolved = nuisc::pipeline::resolve_compile_input(&fixture.0).unwrap();
            let checkpoint = resolved
                .compile_to_verified_yir(&Default::default())
                .unwrap();
            let registry = yir_verify::default_registry();
            for (seed, input, expected) in cases {
                let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
                    checkpoint.yir(),
                    &registry,
                    "counter",
                    vec![yir_core::Value::Int(seed)],
                )
                .unwrap();
                session.event(vec![yir_core::Value::Int(input)]).unwrap();
                session.close(vec![]).unwrap();
                let yir_core::Value::Struct(state) = session.state() else {
                    panic!("missing counter state")
                };
                assert_eq!(state.fields[0].1, yir_core::Value::Int(expected));
            }
            checkpoint.emit_llvm().unwrap();
        }
    }
}

#[test]
fn headless_checkpoint_does_not_bypass_invalid_registration_or_native_codegen_requirements() {
    let source = "mod cpu Main { fn main() { print(1); } }";
    let fixture = project(source);
    let resolved = nuisc::pipeline::resolve_compile_input(&fixture.0).unwrap();
    let error = resolved
        .compile_to_verified_yir(&Default::default())
        .err()
        .unwrap();
    assert!(error.contains("start"), "{error}");
    let output = fixture.0.join("rejected");
    assert!(nuisc::run(nuisc::CommandKind::Compile {
        input: fixture.0.clone(),
        output_dir: output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("headless-aot-bundle".to_owned()),
    })
    .unwrap_err()
    .contains("start"));
    assert!(
        !output.exists(),
        "invalid registration must fail before packaging"
    );

    let compiled = nuisc::pipeline::compile_source(source).unwrap();
    let error = nuisc::aot::write_and_link_with_source(
        &fixture.0.join("main.ns"),
        &output,
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: None,
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .err()
    .unwrap();
    assert!(error.contains("requires a real LLVM checkpoint"), "{error}");
    assert!(
        !output.exists(),
        "no empty native LLVM artifact may be fabricated"
    );
}
