use super::*;
use crate::frontend::parse_nuis_module;

fn source(body: &str) -> String {
    format!(
        "mod cpu Main {{
        struct Packet {{ first: i64, second: i64 }}
        struct Other {{ first: i64, second: i64 }}
        struct Mixed {{ first: i64, second: bool }}
        struct Nested {{ packet: Packet }}
        fn walk(seed: Packet, limit: i64) -> i64 {{
            let packet = seed;
            let saved = packet;
            let index: i64 = 0;
            let total: i64 = 0;
            while index < limit {{ let index: i64 = index + 1; {body} }}
            return packet.first + packet.second + total + saved.first;
        }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn outer_flat_carries_admit_seeded_locals_and_keep_buffer_authority_closed() {
    for body in [
        "let packet = packet;",
        "let packet = saved;",
        "let packet = Packet { second: packet.first, first: packet.second + index };",
        "if index == 1 { let packet = Packet { first: packet.second, second: packet.first }; }",
        "let packet = packet; let old = packet; if index < limit { let packet = saved; } else { let packet = old; } let total: i64 = total + packet.first;",
    ] {
        for reversed in [false, true] {
            let mut module = parse_nuis_module(&source(body)).unwrap();
            if reversed { module.functions.reverse(); }
            let catalog = scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
            assert!(catalog["walk"].may_loop, "{body}");
            assert!(!scalar_helpers::collect(&module).contains_key("walk"));
            assert!(outline_buffer_loops(&mut module).unwrap().functions.contains("walk"));
            let iteration = module.functions.iter().find(|function| function.name.starts_with("__nuis_scalar_iteration_")).unwrap();
            assert!(iteration.params.iter().any(|param| param.name == "packet" && param.ty == scalar_type("Packet")));
            let layout = module.structs.iter().find(|layout| Some(scalar_type(&layout.name)) == iteration.return_type).unwrap();
            assert!(layout.fields.iter().all(|field| field.ty == scalar_type("i64")));
        }
    }
}

#[test]
fn outer_flat_carries_reject_nominal_mutability_scope_and_order_drift() {
    let base = source("let packet = Packet { first: packet.first + index, second: packet.second }; let total: i64 = total + packet.first;");
    for text in [
        base.replace("let packet = seed;", "const packet: Packet = seed;"),
        base.replace("let packet = seed;", ""),
        base.replace(
            "let packet = seed;",
            "let packet = seed; let total: i64 = 7;",
        ),
        base.replace("let packet = Packet", "let seed = Packet"),
        base.replace("let packet = Packet", "let packet: Other = Packet"),
        base.replace("let packet = Packet", "let packet = Other"),
        base.replace("let packet = Packet", "let packet = Mixed")
            .replace("second: packet.second", "second: true"),
        base.replace("packet.first + index", "packet.first + total"),
        base.replace(
            "let total: i64 = total + packet.first;",
            "let limit: i64 = packet.first;",
        ),
        source("let packet = packet; while index < limit { let index: i64 = index + 1; }"),
        source("if index < limit { let temporary = packet; } let packet = temporary;"),
        source("let packet = packet; print(packet.first);"),
        source("let saved = saved;")
            .replace("let saved = packet;", "const saved: Packet = packet;"),
        source("let gate = false;")
            .replace("let packet = seed;", "let packet = seed; let gate = true;"),
    ] {
        let admitted = parse_nuis_module(&text).is_ok_and(|module| {
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                .contains_key("walk")
        });
        assert!(!admitted, "{text}");
    }
}

#[test]
fn outer_flat_carry_width_is_layout_driven_not_a_native_arity_table() {
    for width in [1, 3, 7, 65] {
        let fields = (0..width)
            .map(|n| format!("field{n}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..width)
            .rev()
            .map(|n| format!("field{n}: packet.field{n} + index"))
            .collect::<Vec<_>>()
            .join(", ");
        let text = format!("mod cpu Main {{
            struct Packet {{ {fields} }}
            fn walk(seed: Packet, limit: i64) -> Packet {{
                let packet = seed;
                let index: i64 = 0;
                while index < limit {{ let index: i64 = index + 1; let packet = Packet {{ {values} }}; }}
                return packet;
            }}
            fn main() -> i64 {{ return 0; }}
        }}");
        let mut module = parse_nuis_module(&text).unwrap();
        assert!(
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module))
                ["walk"]
                .may_loop
        );
        outline_buffer_loops(&mut module).unwrap();
        assert!(module.structs.iter().any(|definition| definition
            .name
            .starts_with("__nuis_scalar_carries_")
            && definition.fields.len() == width));
    }
}
