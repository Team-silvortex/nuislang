use super::super::computed::{computed_expected, computed_source};

pub(super) fn staged_source(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> String {
    let value = match kind {
        "division" => "100 / stamp",
        "remainder" => "100 % stamp",
        "call" => "observe(stamp)",
        "field" => "produce(stamp).value",
        _ => unreachable!(),
    };
    computed_source(kind, mode, entry, shape, input, stamp).replace(
        &format!("print({value});"),
        &format!("const local: i64 = {value}; print(local); let copy = local; print(copy);"),
    )
}

pub(super) fn staged_expected(
    kind: &str,
    mode: &str,
    entry: &str,
    shape: &str,
    input: super::super::fixtures::Input,
    stamp: i64,
) -> (Option<i64>, Vec<i64>, usize) {
    let (result, mut prints, calls) = computed_expected(kind, mode, entry, shape, input, stamp);
    if stamp != 0 {
        if let Some(index) = prints.iter().position(|v| matches!(v, 88 | 66)) {
            let value = prints[index + 1];
            prints.insert(index + 2, value);
        }
    }
    (result, prints, calls)
}

pub(super) fn simple_source(kind: &str, outer: bool, stamp: i64) -> String {
    let body = match kind {
        "checked" => "let value = 100 / stamp; print(value); return value > 0;",
        "bool" => "let value = helper(produce(stamp)); print(88); return value;",
        "unused" => "let ignored = observe(stamp); print(88); return true;",
        "unused-checked" => "const ignored: i64 = 100 / stamp; print(88); return true;",
        "unused-between" => "print(88); let ignored = observe(stamp); print(89); return false;",
        "chain" => "print(88); let first = observe(stamp); print(first); const second: i64 = 100 / first; print(second); let copy = second; print(copy); return helper(produce(first));",
        _ => unreachable!(),
    };
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn observe(value: i64) -> i64 {{ return produce(value).value; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, stamp: i64) -> bool {{
            print(99); if outer {{ {body} }} print(77); return false;
        }}
        fn main() -> i64 {{ let result = event({outer}, {stamp});
            if result {{ print(11); return 11; }} print(19); return 19;
        }}
    }}")
}

pub(super) fn simple_expected(
    kind: &str,
    outer: bool,
    stamp: i64,
) -> (Option<i64>, Vec<i64>, usize) {
    if !outer {
        return (Some(19), vec![99, 77, 19], 0);
    }
    if stamp == 0 {
        let prints = if matches!(kind, "chain" | "unused-between") {
            vec![99, 88]
        } else {
            vec![99]
        };
        return (None, prints, 0);
    }
    let value = match kind {
        "unused" | "unused-checked" => true,
        "unused-between" => false,
        _ => stamp > 0,
    };
    let result = if value { 11 } else { 19 };
    let mut prints = vec![99];
    match kind {
        "checked" => prints.push(100 / stamp),
        "chain" => prints.extend([88, stamp, 100 / stamp, 100 / stamp]),
        "unused-between" => prints.extend([88, 89]),
        _ => prints.push(88),
    }
    prints.push(result);
    (
        Some(result),
        prints,
        usize::from(matches!(kind, "bool" | "chain")),
    )
}

pub(super) fn events(source: &str, reversed: bool) -> Vec<String> {
    let mut yir = crate::pipeline::compile_source(source).unwrap().yir;
    if reversed {
        yir.nodes.reverse();
        yir.edges.reverse();
        yir.functions.reverse();
        for function in &mut yir.functions {
            function.body_nodes.reverse();
        }
    }
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap()
    .events
}
