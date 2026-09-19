use super::*;
use crate::frontend::parse_nuis_module;

const HELPERS: &str = "
    struct Packet { first: i64, second: i64 }
    struct Other { first: i64, second: i64 }
    struct Mixed { first: i64, flag: bool }
    struct Nested { packet: Packet }
    fn make(value: i64, divisor: i64) -> Packet {
        if value < 0 { return Packet { second: divisor, first: value / divisor }; }
        return Packet { first: value / divisor, second: divisor };
    }
    fn relay(value: i64, divisor: i64) -> Packet { return make(value, divisor); }
    fn read(packet: Packet) -> i64 { return packet.first; }
    fn select(flag: bool, value: i64) -> Packet {
        if flag { return Packet { first: value, second: 1 }; }
        return Packet { first: 0, second: 0 };
    }
    fn mixed(value: i64) -> Mixed { return Mixed { first: value, flag: true }; }
    fn nested(value: i64) -> Nested { return Nested { packet: relay(value, 1) }; }
    fn effect(value: i64) -> Packet { print(value); return relay(value, 1); }
    fn effect_wrapper(value: i64) -> Packet { return effect(value); }
    fn cycle(value: i64) -> Packet { return cycle(value); }
    fn looped(value: i64) -> Packet {
        let i: i64 = 0; while i < value { let i: i64 = i + 1; }
        return Packet { first: i, second: value };
    }
    fn loop_wrapper(value: i64) -> Packet { return looped(value); }
";

fn source(body: &str) -> String {
    carries_tests::SOURCE
        .replace("fn main()", &format!("{HELPERS} fn main()"))
        .replace(
            "let total: i64 = total + index;\n      let checksum: i64 = checksum * total;",
            body,
        )
}

#[test]
fn iteration_flat_values_keep_nominal_layouts_and_local_capture_scope() {
    for body in [
        "let total: i64 = total + relay(index, stride).first;",
        "let local = relay(index, stride); let saved: Packet = local; let total: i64 = total + read(saved);",
        "let local: Packet = relay(index, stride); if local.first < bound { let total: i64 = total + read(local); }",
        "if index < bound { let local = relay(index, stride); let total: i64 = total + local.first; } else { let local = relay(index, stride); let total: i64 = total - local.second; }",
        "let local = select(index == bound || relay(index, stride).first == 0, index); let total: i64 = total + local.first;",
        "let local = Packet { second: 1, first: select(index == bound || relay(index, stride).first == 0, index).first }; let total: i64 = total + read(local);",
        "let total: i64 = total; let local = relay(total, stride); let total: i64 = total + 1; let total: i64 = total + local.first;",
        "let total: i64 = total + Packet { first: index, second: stride }.first;",
        "let unused = relay(index, stride);",
        "let local = loop_wrapper(index); let total: i64 = total + local.first;",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(catalog.contains_key("walk"), "{body}");
            let scalar_catalog = scalar_helpers::collect(&module);
            for name in ["make", "relay", "read", "select"] {
                assert!(!scalar_catalog.contains_key(name), "Buffer catalog widened: {name}");
            }
            let outlined = outline_buffer_loops(&mut module).unwrap();
            assert!(outlined.functions.contains("walk"));
            let iteration = module.functions.iter().find(|f| f.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(iteration.params.iter().all(|p| !["local", "saved", "unused"].contains(&p.name.as_str())));
        }
    }
}

#[test]
fn iteration_flat_values_reject_effects_cycles_rebinding_and_layout_drift() {
    for body in [
        "let local = mixed(index); let total: i64 = total + local.first;",
        "let local = nested(index); let total: i64 = total + local.packet.first;",
        "let local = effect(index); let total: i64 = total + local.first;",
        "let local = effect_wrapper(index); let total: i64 = total + local.first;",
        "let local = cycle(index); let total: i64 = total + local.first;",
        "let local = relay(index, stride); let local: Packet = relay(index, stride);",
        "let local: Other = relay(index, stride);",
        "let local = Other { first: index, second: stride }; let total: i64 = total + read(local);",
        "if index < bound { let local = relay(index, stride); } let total: i64 = total + local.first;",
        "let local = relay(checksum, stride); let checksum: i64 = checksum + 1; let total: i64 = total + local.first;",
        "let local = Packet { first: checksum, second: stride }; let checksum: i64 = checksum + 1; let total: i64 = total + local.first;",
        "let bound: i64 = relay(index, stride).first;",
        "let index: i64 = relay(index, stride).first;",
    ] {
        if let Ok(module) = parse_nuis_module(&source(body)) {
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(!catalog.contains_key("walk"), "{body}");
            assert!(!catalog.contains_key("choose"), "{body}");
        }
    }
    let valid = source("let total: i64 = total + relay(index, stride).first;");
    for text in [
        valid.replace("index < limit", "index < relay(limit, 1).first"),
        valid.replace("index + stride", "index + relay(stride, 1).first"),
    ] {
        let module = parse_nuis_module(&text).unwrap();
        assert!(
            !scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
    }
}

#[test]
fn iteration_flat_value_layout_width_is_not_a_native_slot_table() {
    for width in [1, 3, 7, 65] {
        let fields = (0..width)
            .map(|n| format!("field{n}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..width)
            .rev()
            .map(|n| format!("field{n}: value + {n}"))
            .collect::<Vec<_>>()
            .join(", ");
        let text = source("let local = wide(index); let total: i64 = total + local.field0;")
            .replace("fn main()", &format!("struct Wide {{ {fields} }} fn wide(value: i64) -> Wide {{ return Wide {{ {values} }}; }} fn main()"));
        let mut module = parse_nuis_module(&text).unwrap();
        assert!(
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        );
        assert!(outline_buffer_loops(&mut module)
            .unwrap()
            .functions
            .contains("wide"));
    }
}
