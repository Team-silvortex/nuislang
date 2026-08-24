use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should follow Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nuis_stage_handoff_{label}_{nonce}"));
    fs::create_dir_all(&path).expect("create temp directory");
    path
}

fn payload_bytes() -> [(&'static str, &'static [u8]); 5] {
    [
        (
            "demo.source.ns",
            b"mod cpu Main { fn main() -> i64 { return 7; } }\n",
        ),
        ("demo.tokens.txt", b"nuis-token-stream-v1\nword\t6d6f64\n"),
        ("demo.ast.txt", b"ast mod cpu unit Main\n"),
        ("demo.nir.txt", b"nir mod cpu unit Main\n"),
        ("demo.yir", b"yir 0.1\n"),
    ]
}

fn build(producer_id: &str) -> CompilerStageHandoff {
    let payloads = payload_bytes();
    let inputs = ORDERED_STAGES
        .iter()
        .zip(payloads.iter())
        .map(|(stage, (file, bytes))| CompilerStagePayloadInput {
            stage: *stage,
            payload_file: file,
            bytes,
        })
        .collect::<Vec<_>>();
    build_compiler_stage_handoff(producer_id, "cpu", "Main", &inputs).expect("build handoff")
}

#[test]
fn compiler_stage_handoff_round_trips_and_verifies_payloads() {
    let dir = temp_dir("roundtrip");
    for (file, bytes) in payload_bytes() {
        fs::write(dir.join(file), bytes).expect("write payload");
    }
    let handoff = build("nuisc-stage0");
    let rendered = render_compiler_stage_handoff(&handoff);
    let parsed = parse_compiler_stage_handoff_from_source(
        &rendered,
        &dir.join("nuis.compiler-stage-handoff.toml"),
    )
    .expect("parse handoff");
    assert_eq!(parsed, handoff);
    assert_eq!(render_compiler_stage_handoff(&parsed), rendered);

    let manifest_path = dir.join("nuis.compiler-stage-handoff.toml");
    fs::write(&manifest_path, rendered).expect("write manifest");
    let (loaded, payloads) = read_compiler_stage_handoff(&manifest_path).expect("read handoff");
    assert_eq!(loaded, handoff);
    assert_eq!(payloads.len(), 5);
    assert_eq!(payloads[4].stage, CompilerStageKind::Yir);
    fs::remove_dir_all(dir).expect("remove temp directory");
}

#[test]
fn semantic_identity_is_independent_of_stage_implementation_name() {
    let stage0 = build("nuisc-stage0");
    let stage1 = build("nuis-stage1");

    assert_ne!(stage0.producer_id, stage1.producer_id);
    assert_eq!(stage0.semantic_root_sha256, stage1.semantic_root_sha256);
    assert_eq!(stage0.bundle_sha256, stage1.bundle_sha256);
    assert_eq!(stage0.records, stage1.records);
}

#[test]
fn payload_tampering_fails_closed() {
    let dir = temp_dir("tamper");
    for (file, bytes) in payload_bytes() {
        fs::write(dir.join(file), bytes).expect("write payload");
    }
    let manifest_path = dir.join("nuis.compiler-stage-handoff.toml");
    fs::write(
        &manifest_path,
        render_compiler_stage_handoff(&build("nuisc-stage0")),
    )
    .expect("write manifest");
    fs::write(dir.join("demo.nir.txt"), "tampered\n").expect("tamper NIR");

    let error = read_compiler_stage_handoff(&manifest_path).expect_err("tamper must fail");
    assert!(error.to_string().contains("length or SHA-256"));
    fs::remove_dir_all(dir).expect("remove temp directory");
}

#[test]
fn structurally_invalid_ast_fails_even_with_recomputed_hash_chain() {
    let dir = temp_dir("structural_tamper");
    for (file, bytes) in payload_bytes() {
        fs::write(dir.join(file), bytes).expect("write payload");
    }
    let malformed_ast = b"ast mod cpu unit Main\nuse text Text\n";
    fs::write(dir.join("demo.ast.txt"), malformed_ast).expect("tamper AST");

    let mut handoff = build("nuisc-stage0");
    handoff.records[2].payload_bytes = malformed_ast.len();
    handoff.records[2].payload_sha256 = sha256_hex(malformed_ast);
    let mut parent = handoff.records[1].record_sha256.clone();
    for record in &mut handoff.records[2..] {
        record.parent_sha256 = parent.clone();
        record.record_sha256 = record_identity(
            &handoff.semantic_root_sha256,
            &parent,
            record.ordinal,
            record.stage,
            &record.encoding,
            record.payload_bytes,
            &record.payload_sha256,
        );
        parent = record.record_sha256.clone();
    }
    handoff.bundle_sha256 = parent;

    let manifest_path = dir.join("nuis.compiler-stage-handoff.toml");
    fs::write(&manifest_path, render_compiler_stage_handoff(&handoff))
        .expect("write recomputed manifest");
    let error = read_compiler_stage_handoff(&manifest_path)
        .expect_err("structural tampering must fail after hash verification");
    assert!(
        error.to_string().contains("structurally indented"),
        "{error}"
    );
    fs::remove_dir_all(dir).expect("remove temp directory");
}

#[test]
fn projection_identity_must_match_handoff_during_build() {
    let payloads = payload_bytes();
    let mismatched_ast = b"ast mod cpu unit Other\n";
    let inputs = ORDERED_STAGES
        .iter()
        .zip(payloads.iter())
        .map(|(stage, (file, bytes))| CompilerStagePayloadInput {
            stage: *stage,
            payload_file: file,
            bytes: if *stage == CompilerStageKind::Ast {
                mismatched_ast
            } else {
                bytes
            },
        })
        .collect::<Vec<_>>();

    let error = build_compiler_stage_handoff("nuisc-stage0", "cpu", "Main", &inputs)
        .expect_err("projection identity drift must fail");
    assert!(error.to_string().contains("does not match handoff module"));
}

#[test]
fn noncanonical_manifest_text_fails_closed() {
    let dir = temp_dir("manifest_canonical");
    for (file, bytes) in payload_bytes() {
        fs::write(dir.join(file), bytes).expect("write payload");
    }
    let manifest_path = dir.join("nuis.compiler-stage-handoff.toml");
    let mut rendered = render_compiler_stage_handoff(&build("nuisc-stage0"));
    rendered.push_str("\n# noncanonical trailing content\n");
    fs::write(&manifest_path, rendered).expect("write noncanonical manifest");

    let error = read_compiler_stage_handoff(&manifest_path).expect_err("manifest must fail");
    assert!(error.to_string().contains("not canonically encoded"));
    fs::remove_dir_all(dir).expect("remove temp directory");
}

#[test]
fn noncanonical_order_and_parent_paths_are_rejected() {
    let payloads = payload_bytes();
    let wrong_order = [
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Tokens,
            payload_file: payloads[1].0,
            bytes: payloads[1].1,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Source,
            payload_file: payloads[0].0,
            bytes: payloads[0].1,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Ast,
            payload_file: payloads[2].0,
            bytes: payloads[2].1,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Nir,
            payload_file: payloads[3].0,
            bytes: payloads[3].1,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Yir,
            payload_file: payloads[4].0,
            bytes: payloads[4].1,
        },
    ];
    let error = build_compiler_stage_handoff("nuisc-stage0", "cpu", "Main", &wrong_order)
        .expect_err("wrong stage order must fail");
    assert!(error.to_string().contains("must be `source`"));

    let mut escaping = wrong_order;
    escaping[0] = CompilerStagePayloadInput {
        stage: CompilerStageKind::Source,
        payload_file: "../demo.ns",
        bytes: payloads[0].1,
    };
    escaping[1] = CompilerStagePayloadInput {
        stage: CompilerStageKind::Tokens,
        payload_file: payloads[1].0,
        bytes: payloads[1].1,
    };
    let error = build_compiler_stage_handoff("nuisc-stage0", "cpu", "Main", &escaping)
        .expect_err("parent path must fail");
    assert!(error.to_string().contains("one relative file name"));

    escaping[0].payload_file = r"..\demo.ns";
    let error = build_compiler_stage_handoff("nuisc-stage0", "cpu", "Main", &escaping)
        .expect_err("portable parent path must fail");
    assert!(error.to_string().contains("one relative file name"));

    escaping[0].payload_file = payloads[0].0;
    escaping[1].payload_file = payloads[0].0;
    let error = build_compiler_stage_handoff("nuisc-stage0", "cpu", "Main", &escaping)
        .expect_err("duplicate payload file must fail");
    assert!(error.to_string().contains("registered more than once"));
}
