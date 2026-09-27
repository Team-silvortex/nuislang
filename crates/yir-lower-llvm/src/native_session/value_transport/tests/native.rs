use super::*;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "nuis-value-transport-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn scalar(
    index: usize,
    case: usize,
    mixed: bool,
    body: &mut Vec<String>,
    next: &mut usize,
) -> (&'static str, LlvmValueRef, u64) {
    match if mixed { index % 5 } else { 2 } {
        0 => {
            let bit = (case + index) % 2;
            (
                "bool",
                LlvmValueRef::Bool {
                    i1: bit.to_string(),
                    i64: bit.to_string(),
                },
                bit as u64,
            )
        }
        1 => {
            let number = [i32::MIN, i32::MAX, -1, 0, 1][(case + index) % 5];
            (
                "i32",
                LlvmValueRef::I32(number.to_string()),
                number as i64 as u64,
            )
        }
        2 => {
            let number = [i64::MIN, i64::MAX, -1, 0, 1, 0x1234_5678_9abc_def0][(case + index) % 6];
            ("i64", LlvmValueRef::I64(number.to_string()), number as u64)
        }
        3 => {
            let bits = [
                0_u32,
                0x8000_0000,
                0x7f80_0000,
                0xff80_0000,
                0x7fc1_2345,
                0x7fa1_2345,
                1,
                0x7f7f_ffff,
            ][(case + index) % 8];
            let value = fresh_reg(next);
            body.push(format!("  {value} = bitcast i32 {bits} to float"));
            ("f32", LlvmValueRef::F32(value), bits as u64)
        }
        _ => {
            let bits = [
                0_u64,
                0x8000_0000_0000_0000,
                0x7ff0_0000_0000_0000,
                0xfff0_0000_0000_0000,
                0x7ff8_1234_5678_9abc,
                0x7ff1_1234_5678_9abc,
                1,
                0x7fef_ffff_ffff_ffff,
            ][(case + index) % 8];
            let value = fresh_reg(next);
            body.push(format!("  {value} = bitcast i64 {bits} to double"));
            ("f64", LlvmValueRef::F64(value), bits)
        }
    }
}

fn input(
    count: usize,
    case: usize,
    mixed: bool,
    body: &mut Vec<String>,
    next: &mut usize,
) -> (String, LlvmValueRef, Vec<u64>) {
    let mut schema = Vec::new();
    let mut fields = Vec::new();
    let mut expected = Vec::new();
    for index in 0..count {
        let (kind, value, word) = scalar(index, case, mixed, body, next);
        schema.push(format!("f{index}:{kind}"));
        fields.push((format!("f{index}"), value));
        expected.push(word);
    }
    // Opposite source orders must still use the declared transport layout.
    if case % 2 == 0 {
        fields.reverse();
    }
    let value = LlvmValueRef::Struct(StructLlvmValueRef {
        type_name: "Fields".to_owned(),
        fields,
    });
    if mixed {
        (
            format!("Packet{{child:Fields{{{}}}}}", schema.join(";")),
            record("Packet", vec![("child", value)]),
            expected,
        )
    } else {
        (format!("Fields{{{}}}", schema.join(";")), value, expected)
    }
}

fn round_trip(name: &str, layout: &NativeValueLayout) -> String {
    let mut body = Vec::new();
    let mut next = 0;
    let first = layout.unpack("%first", &mut body, &mut next);
    let second = layout.unpack("%second", &mut body, &mut next);
    let first = layout
        .prepare(&LlvmValueRef::Struct(first))
        .unwrap()
        .emit(&mut body, &mut next);
    let second = layout
        .prepare(&LlvmValueRef::Struct(second))
        .unwrap()
        .emit(&mut body, &mut next);
    let selected = fresh_reg(&mut next);
    let ty = layout.llvm_type();
    body.push(format!(
        "  {selected} = select i1 %which, {ty} {first}, {ty} {second}"
    ));
    body.push(format!("  ret {ty} {selected}"));
    format!(
        "define {ty} @{name}({ty} %first, {ty} %second, i1 %which) noinline {{\nentry:\n{}\n}}\n",
        body.join("\n")
    )
}

fn program() -> String {
    let mut definitions = Vec::new();
    let mut body = Vec::new();
    let mut next = 0;
    let mut all_ok = "true".to_owned();
    for (count, mixed) in [
        (1, false),
        (2, false),
        (7, false),
        (64, false),
        (5, true),
        (7, true),
        (64, true),
    ] {
        let mut layout = None;
        let name = format!("round_trip_{count}_{mixed}");
        for case in 0..8 {
            let (encoded, first, first_words) = input(count, case, mixed, &mut body, &mut next);
            let (_, second, second_words) = input(count, case + 1, mixed, &mut body, &mut next);
            let plan = layout.get_or_insert_with(|| NativeValueLayout::parse(&encoded).unwrap());
            let first = plan.prepare(&first).unwrap().emit(&mut body, &mut next);
            let second = plan.prepare(&second).unwrap().emit(&mut body, &mut next);
            let ty = plan.llvm_type();
            for (select, words) in [(true, &first_words), (false, &second_words)] {
                let returned = fresh_reg(&mut next);
                body.push(format!(
                    "  {returned} = call {ty} @{name}({ty} {first}, {ty} {second}, i1 {select})"
                ));
                for (index, expected) in words.iter().enumerate() {
                    let field = fresh_reg(&mut next);
                    body.push(format!("  {field} = extractvalue {ty} {returned}, {index}"));
                    let equal = fresh_reg(&mut next);
                    body.push(format!("  {equal} = icmp eq i64 {field}, {expected}"));
                    let combined = fresh_reg(&mut next);
                    body.push(format!("  {combined} = and i1 {all_ok}, {equal}"));
                    all_ok = combined;
                }
            }
        }
        definitions.push(round_trip(&name, layout.as_ref().unwrap()));
    }
    let failed = fresh_reg(&mut next);
    let status = fresh_reg(&mut next);
    body.push(format!("  {failed} = xor i1 {all_ok}, true"));
    body.push(format!("  {status} = zext i1 {failed} to i32"));
    body.push(format!("  ret i32 {status}"));
    definitions.push(format!(
        "define i32 @main() {{\nentry:\n{}\n}}\n",
        body.join("\n")
    ));
    definitions.join("\n")
}

#[test]
#[ignore = "requires a host clang driver; run explicitly with --ignored"]
fn typed_record_arguments_round_trip_through_host_llvm() {
    let llvm = program();
    for prohibited in [
        "alloca ",
        "load ",
        "store ",
        "ptr ",
        "@nuis_scheduler_",
        "@malloc",
    ] {
        assert!(!llvm.contains(prohibited), "{prohibited}");
    }
    assert!(llvm.contains("[64 x i64] %first, [64 x i64] %second, i1 %which"));
    let scratch = Scratch::new();
    let source = scratch.0.join("roundtrip.ll");
    fs::write(&source, llvm).unwrap();
    for optimization in ["-O0", "-O2"] {
        let binary = scratch
            .0
            .join(format!("roundtrip{}", std::env::consts::EXE_SUFFIX));
        let output = Command::new(std::env::var_os("CLANG").unwrap_or_else(|| "clang".into()))
            .args(["-x", "ir", "-Werror", "-Wno-override-module", optimization])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("native value-transport test requires host clang (or CLANG)");
        assert!(
            output.status.success(),
            "{optimization}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let run = Command::new(&binary).output().unwrap();
        assert!(
            run.status.success(),
            "{optimization}: {:?}: {}",
            run.status,
            String::from_utf8_lossy(&run.stderr)
        );
    }
}
