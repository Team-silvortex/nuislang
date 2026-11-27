use super::*;

const SOURCE: &str = include_str!(
    "../../../../examples/projects/domains/native_guarded_canvas_session_demo/main.ns"
);
const MANIFEST: &str = include_str!(
    "../../../../examples/projects/domains/native_guarded_canvas_session_demo/nuis.toml"
);

fn check(output: Output, image: &Path) {
    let output = success(output);
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("native_resource_scalars=close:7:[3, 2, 5, 2]"),
        "{text}"
    );
    assert!(text.contains("native_resource_completed=1"), "{text}");
    let mut expected = b"P6\n3 2\n255\n\xff\x00\x00\x00\xff\x00".to_vec();
    expected.extend([0; 12]);
    assert_eq!(fs::read(image).unwrap(), expected);
    fs::remove_file(image).unwrap();
}

#[test]
fn shared_resource_continuation_survives_cache_nsb_and_binary_only_launch() {
    delivery(SOURCE);
}

#[test]
fn nested_resource_reads_survive_cache_nsb_and_binary_only_launch() {
    delivery(&nested_source());
}

#[test]
fn resource_helper_reads_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace(
            "mod cpu Main {",
            "mod cpu Main { fn nonempty(value: i64) -> bool { return value > 0; }",
        )
        .replace(
            "bytes_len(snapshot) == width * height * 8",
            "nonempty(bytes_len(snapshot))",
        )
        .replace("load_at(pixels, 99) > 0", "nonempty(load_at(pixels, 99))");
    delivery(&source);
}

#[test]
fn resource_logical_arguments_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn decision(flag: bool) -> Validation { return Validation { extent: Validity { invalid: !flag } }; }")
        .replace(
            "color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)",
            "!decision(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)).extent.invalid",
        );
    delivery(&source);
}

#[test]
fn resource_record_initializers_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn decision(flag: bool) -> Validation { return Validation { extent: Validity { invalid: !flag } }; }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let validation: Validation = decision(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)); let valid: bool = !validation.extent.invalid;");
    assert!(source.contains("let validation: Validation = decision(color >= 0 &&"));
    delivery(&source);
}

#[test]
fn resource_record_literals_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source().replace(
        "let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
        "let validation: Validation = Validation { extent: Validity { invalid: !(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)) } }; let valid: bool = !validation.extent.invalid;");
    assert!(source.contains("let validation: Validation = Validation { extent: Validity"));
    delivery(&source);
}

#[test]
fn resource_record_helper_fields_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn status(flag: bool) -> Validity { return Validity { invalid: !flag }; }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let validation: Validation = Validation { extent: status(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)) }; let valid: bool = !validation.extent.invalid;");
    assert!(source.contains("let validation: Validation = Validation { extent: status("));
    delivery(&source);
}

#[test]
fn resource_record_local_fields_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { struct Check { extent: Validity, valid: bool }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let prior: Validation = Validation { extent: Validity { invalid: color < 0 } }; let checked = Check { extent: prior.extent, valid: color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0) }; let valid: bool = !checked.extent.invalid && checked.valid;");
    assert!(source.contains("extent: prior.extent, valid: color >= 0 &&"));
    delivery(&source);
}

#[test]
fn resource_record_liveness_survives_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { struct Check { extent: Validity, valid: bool }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let prior: Validation = Validation { extent: Validity { invalid: color < 0 } }; let checked = Check { valid: color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0), extent: prior.extent }; let valid: bool = !checked.extent.invalid && checked.valid;");
    assert!(source.contains("load_at(pixels, 99) > 0), extent: prior.extent"));
    delivery(&source);
}

#[test]
fn scalar_selection_survives_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace(
            "color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)",
            "word(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)) == 1",
        );
    delivery(&source);
}

#[test]
fn resource_record_operands_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { struct Check { extent: Validity, valid: bool }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let prior: Validation = Validation { extent: Validity { invalid: color < 0 } }; let checked = Check { valid: !prior.extent.invalid && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0), extent: prior.extent }; let valid: bool = !checked.extent.invalid && checked.valid;");
    assert!(source.contains("valid: !prior.extent.invalid &&"));
    delivery(&source);
}

#[test]
fn wrapped_resource_predicates_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source().replace(
        "color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)",
        "((color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)) == (width > 0)) ^ (height < 0)",
    );
    delivery(&source);
}

#[test]
fn local_record_bool_initializer_survives_cache_nsb_and_binary_only_launch() {
    delivery(&record_scalar_source(false));
}

#[test]
fn local_record_i64_initializer_survives_cache_nsb_and_binary_only_launch() {
    delivery(&record_scalar_source(true));
}

#[test]
fn local_record_condition_survives_cache_nsb_and_binary_only_launch() {
    let predicate = "!prior.extent.invalid && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)";
    let declaration = format!("let valid: bool = {predicate};");
    let condition = format!("if !({predicate}) {{");
    let source = record_scalar_source(false);
    assert_eq!(source.matches(&declaration).count(), 1);
    assert_eq!(source.matches("if !valid {").count(), 1);
    let source = source
        .replacen(&declaration, "", 1)
        .replacen("if !valid {", &condition, 1);
    assert_eq!(source.matches(&condition).count(), 1);
    assert!(!source.contains(&declaration) && !source.contains("if !valid {"));
    delivery(&source);
}

fn record_scalar_source(integer: bool) -> String {
    let predicate = "!prior.extent.invalid && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)";
    let declaration = if integer {
        format!("let code: i64 = word({predicate}); let valid: bool = code == 1;")
    } else {
        format!("let valid: bool = {predicate};")
    };
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace("let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);", &format!("let prior: Validation = Validation {{ extent: Validity {{ invalid: color < 0 }} }}; {declaration}"));
    assert!(source.contains(&declaration));
    source
}

#[test]
fn direct_i64_resource_arguments_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace(
            "let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);",
            "let choice: i64 = word(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)); let valid = choice == 1;",
        );
    delivery(&source);
}

#[test]
fn inline_resource_conditions_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source().replace(
        "let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);\n    if !valid",
        "if !(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0))",
    );
    assert!(!source.contains("let valid = color >= 0"));
    delivery(&source);
}

#[test]
fn returned_resource_fields_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace("cursor: cursor, pixels: move(snapshot)", "cursor: cursor + word(color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0)) - 1, pixels: move(snapshot)");
    assert!(source.contains("cursor: cursor + word"));
    delivery(&source);
}

#[test]
fn local_record_return_field_survives_source_free_delivery() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace("if color == 0 {", "if color == 0 { let prior = measure(width, height);")
        .replace("cursor: cursor, pixels: move(snapshot)", "cursor: cursor + word(prior.extent.words == width * height && (bytes_len(snapshot) == prior.extent.words * 8 || load_at(pixels, 99) > 0)) - 1, pixels: move(snapshot)");
    assert_eq!(
        source
            .matches("cursor: cursor + word(prior.extent.words")
            .count(),
        1
    );
    delivery(&source);
}

#[test]
fn buffer_length_return_fields_survive_cache_nsb_and_binary_only_launch() {
    let source = nested_source()
        .replace("mod cpu Main {", "mod cpu Main { fn word(flag: bool) -> i64 { if flag { return 1; } return 0; }")
        .replace("cursor: cursor, pixels: move(snapshot)", "cursor: cursor + buffer_len(pixels) * word(color >= 0 && (bytes_len(snapshot) == buffer_len(pixels) * 8 || load_at(pixels, buffer_len(pixels)) > 0)) - width * height, pixels: move(snapshot)");
    assert!(source.contains("cursor: cursor + buffer_len(pixels) * word"));
    delivery(&source);
}

fn nested_source() -> String {
    let mut source = SOURCE.to_owned();
    let start = source.find("    let valid: bool = color >= 0;").unwrap();
    let end = source.find("\n  fn canvas_close").unwrap();
    source.replace_range(start..end, r#"
    let snapshot: Bytes = copy_bytes(pixels);
    let valid = color >= 0 && (bytes_len(snapshot) == width * height * 8 || load_at(pixels, 99) > 0);
    if !valid { return Result.Err(8); }
    if color == 0 {
      return Result.Ok(Canvas { width: width, height: height, events: events + 1,
                               cursor: cursor, pixels: move(snapshot) });
    }
    drop_bytes(snapshot);
    store_at(pixels, cursor, color);
    return Result.Ok(Canvas { width: width, height: height, events: events + 1,
                             cursor: (cursor + 1) % buffer_len(pixels), pixels: copy_bytes(pixels) });
  }
"#);
    source
}

fn delivery(source: &str) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    fs::write(project.0.join("main.ns"), source).unwrap();
    fs::write(project.0.join("nuis.toml"), MANIFEST).unwrap();
    project.build(None);
    let build = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&build.join("nuis.build.manifest.toml")).unwrap();
    let binary = build.join(&report.artifact_binary_name);
    let image = project.0.join("guarded.ppm");
    let args = [
        "--resource-session",
        "canvas",
        "--buffer-words",
        "4278190080,4278190080,4278190080,4278190080,4278190080,4278190080",
        "--open-arg",
        "3",
        "--event-arg",
        "0",
        "--event-arg",
        "4278190335",
        "--event-arg",
        "0",
        "--event-arg",
        "4278255360",
        "--event-arg",
        "0",
        "--close-arg",
        "0",
        "--resource-frame-fields",
        "width,height",
        "--resource-frame-output",
        image.to_str().unwrap(),
    ];
    check(project.command("run-artifact", &build, &args), &image);
    assert!(project.build(None).contains("compile_cache: hit"));
    check(project.command("run-artifact", &build, &args), &image);
    let artifact = project.0.join("nuis.compiled.artifact");
    let bare = project.0.join("guarded-app");
    fs::copy(build.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::copy(binary, &bare).unwrap();
    fs::remove_dir_all(&build).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    check(
        Command::new(&bare)
            .args(args)
            .env("PATH", "")
            .current_dir(&project.0)
            .output()
            .unwrap(),
        &image,
    );
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    check(project.command("run-artifact", &relocated, &args), &image);
}
