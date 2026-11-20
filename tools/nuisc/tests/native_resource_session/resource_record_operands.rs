use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::compile;
use super::resource_record_initializers::source;
use super::*;

pub(super) const TRAPS: &[(&str, &str, bool)] = &[
    ("and_skip", "word(prior.enabled && load_at(pixels, prior.index) > 0)", false),
    ("and_take", "word(!prior.enabled && load_at(pixels, prior.index) > 0)", false),
    ("or_skip", "word(!prior.enabled || load_at(pixels, prior.index) > 0)", true),
    ("or_take", "word(prior.enabled || load_at(pixels, prior.index) > 0)", false),
    ("not_skip", "word(!(prior.enabled && load_at(pixels, prior.index) > 0))", true),
    ("not_take", "word(!(!prior.enabled && load_at(pixels, prior.index) > 0))", false),
    ("xor_skip", "word((prior.enabled && load_at(pixels, prior.index) > 0) ^ !prior.enabled)", true),
    ("xor_take", "word((!prior.enabled && load_at(pixels, prior.index) > 0) ^ prior.enabled)", false),
    ("nested_skip", "word(!prior.enabled && (bytes_len(saved) > prior.zero || load_at(pixels, prior.index) > 0))", true),
    ("nested_take", "word(!prior.enabled && (bytes_len(saved) == prior.zero || load_at(pixels, prior.index) > 0))", false),
    ("helper_skip", "word(choose(prior.enabled && load_at(pixels, prior.index) > 0).value)", false),
    ("helper_take", "word(choose(!prior.enabled && load_at(pixels, prior.index) > 0).value)", false),
    ("div_skip", "word(prior.enabled && divide(bytes_len(saved) - 48 + prior.zero))", false),
    ("div_take", "word(!prior.enabled && divide(bytes_len(saved) - 48 + prior.zero))", false),
    ("left_take", "load_at(pixels, prior.index) + word(prior.enabled && bytes_len(saved) > 0)", false),
    ("right_take", "word(prior.enabled && bytes_len(saved) > 0) + 100 / prior.zero", false),
    ("init_read_take", "word(prior.enabled && bytes_len(saved) > 0)", false),
    ("init_div_take", "word(prior.enabled && bytes_len(saved) > 0)", false),
];

#[test]
fn record_operands_preserve_truth_tables_completed_reads_and_nominal_paths_for_120_events() {
    for (case, predicate) in [
        "prior.flags.valid && bytes_len(saved) == 48",
        "!(prior.flags.valid || bytes_len(saved) == 0)",
        "(prior.flags.valid && bytes_len(saved) == 48) ^ (prior.left < 3)",
        "choose(prior.flags.valid && load_at(pixels, prior.right - 6) == 99).value",
        "prior.flags.valid && (bytes_len(saved) == prior.right * 8 || load_at(pixels, 99) > 0)",
        "(prior.flags.valid && buffer_len(pixels) == prior.right) == (prior.left < 3)",
        "prior.flags.valid && buffer_len(pixels) == prior.right",
    ]
    .into_iter()
    .enumerate()
    {
        for declaration in ["let local: Envelope", "let local", "const local: Parcel"] {
            let packet = if case == 6 {
                "packet(prior.left, prior.flags.valid && bytes_len(saved) == prior.right * 8, prior.right)"
            } else {
                "prior"
            };
            let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); store_at(pixels, 0, mode); let outer = envelope(load_at(pixels, 0), mode % 2 == 0, buffer_len(pixels)); let prior: Alias = outer.packet; store_at(pixels, 0, 99); {declaration} = Envelope {{ unused: {predicate}, packet: {packet} }}; return Result.Ok(Canvas {{ width: local.packet.left - local.packet.right + local.packet.value + word(local.packet.flags.valid) + word(local.unused), pixels: move(saved) }});"))
                .replace("mod cpu Main {", "mod cpu Main { type Parcel = Envelope;");
            let program = compile(&source);
            assert_eq!(
                program.ast,
                nuisc::frontend::parse_nuis_ast(&source).unwrap()
            );
            assert_eq!(
                program.yir,
                yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir))
                    .unwrap()
            );
            let bridge = bridge(&program);
            let library = native::Library::build(&source, &program, &bridge.llvm_ir);
            let (mut session, _) = NativeResourceSession::open(
                &program.yir,
                &registration(),
                library.checked_binding(&program.yir, &bridge),
                vec![9; 6],
                3,
            )
            .unwrap();
            for mode in (0..6).cycle().take(120) {
                let even = mode % 2 == 0;
                let flag = match case {
                    1 => !even,
                    2 => even ^ (mode < 3),
                    5 => even == (mode < 3),
                    _ => even,
                };
                session.event(mode).unwrap();
                assert_eq!(
                    session.scalar_state(),
                    [mode - 6 + if even { 18 } else { 0 } + if flag { 9 } else { 0 }],
                    "{case}/{declaration}/{mode}"
                );
                assert_eq!(session.state(), [9; 6]);
                assert_eq!(library.live_blobs(), 0);
            }
            session.close(0).unwrap();
            session.completion_status().unwrap();
        }
    }
}

#[test]
fn record_operands_share_used_leaves_without_widening_capture_limits() {
    for count in [1, 17, 29, 30] {
        let fields = (0..count)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..count)
            .map(|i| format!("f{i}: width + {i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let source = source(&format!("let saved: Bytes = copy_bytes(pixels); let prior = Wide {{ {values} }}; let local = Pair {{ gate: prior.f0 > 0 && buffer_len(pixels) == 6, first: prior, second: prior }}; return Result.Ok(Canvas {{ width: local.first.f0 + local.second.f{} + word(local.gate), pixels: move(saved) }});", count - 1))
            .replace("mod cpu Main {", &format!("mod cpu Main {{ struct Wide {{ {fields} }} struct Pair {{ gate: bool, first: Wide, second: Wide }}"));
        let result = nuisc::pipeline::compile_resource_session_source(
            &source,
            &registration(),
            &Default::default(),
        )
        .and_then(|program| program.emit_llvm())
        .and_then(|program| {
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
        });
        if count <= 29 {
            result.unwrap_or_else(|error| panic!("{count}: {error}"));
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("signature metadata"), "{error}");
        }
    }
}

#[test]
fn record_operands_admit_standalone_scalar_declarations() {
    let source = source("let saved: Bytes = copy_bytes(pixels); let prior = packet(0, mode == 0, 0); let valid: bool = prior.flags.valid && bytes_len(saved) == 48; return Result.Ok(Canvas { width: word(valid), pixels: move(saved) });");
    let program = compile(&source);
    let bridge = bridge(&program);
    let library = native::Library::build(&source, &program, &bridge.llvm_ir);
    let (mut session, _) = NativeResourceSession::open(
        &program.yir,
        &registration(),
        library.checked_binding(&program.yir, &bridge),
        vec![9; 6],
        3,
    )
    .unwrap();
    for mode in (0..6).cycle().take(120) {
        session.event(mode).unwrap();
        assert_eq!(session.scalar_state(), [if mode == 0 { 9 } else { 0 }]);
        assert_eq!(session.state(), [9; 6]);
        assert_eq!(library.live_blobs(), 0);
    }
    session.close(0).unwrap();
    session.completion_status().unwrap();
}
