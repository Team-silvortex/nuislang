use super::*;
use crate::frontend::parse_nuis_module;

fn module() -> NirModule {
    parse_nuis_module(
        "mod cpu Main {
        struct Leaf { flag: bool, tag: i32, value: i64, gain: f32, scale: f64 }
        struct Packet { payload: Leaf }
        struct Flat { value: i64 }
        fn leaf(value: Packet) -> Packet { return value; }
        fn checked(value: i64, divisor: i64) -> i32 { return i32_from_i64(value / divisor); }
        fn count(limit: i64) -> i64 {
            let index: i64 = 0;
            while index < limit { let index: i64 = index + 1; }
            return index;
        }
        fn wrapped(limit: i64, value: Packet) -> Packet {
            let work: i64 = count(limit); return leaf(value);
        }
        fn mixed_loop(limit: i64, value: Packet) -> Packet {
            let current = value;
            let index: i64 = 0;
            while index < limit {
                let current = leaf(current);
                let index: i64 = index + 1;
            }
            return current;
        }
        fn effect(value: Packet) -> Packet { print(value.payload.value); return value; }
        fn caller(value: Packet) -> Packet { return effect(value); }
        fn cycle_a(value: Packet) -> Packet { return cycle_b(value); }
        fn cycle_b(value: Packet) -> Packet { return cycle_a(value); }
        fn main() -> i64 { return 0; }
    }",
    )
    .unwrap()
}

#[test]
fn typed_values_reuse_flat_dependencies_without_admitting_mixed_loop_carries_or_effects() {
    let module = module();
    let flat = layouts(&module);
    let loops = scalar_helpers::collect_with_layouts(&module, &flat);
    let typed = TypedLayouts::collect(&module);
    let values = scalar_helpers::collect_typed_values(&module, &typed, &loops);
    assert_eq!(
        values.keys().map(String::as_str).collect::<Vec<_>>(),
        ["checked", "count", "leaf", "main", "wrapped"]
    );
    assert!(values["wrapped"].may_loop);
    assert!(!loops.contains_key("leaf"));
    assert!(!supported_type(&scalar_type("Packet"), &flat));
    let mut reversed = module.clone();
    reversed.structs.reverse();
    reversed.functions.reverse();
    let typed_reversed = TypedLayouts::collect(&reversed);
    let reordered = scalar_helpers::collect_typed_values(&reversed, &typed_reversed, &loops);
    assert_eq!(
        values.keys().collect::<Vec<_>>(),
        reordered.keys().collect::<Vec<_>>()
    );
    let zero = zero_value(&scalar_type("Packet"), &typed);
    assert_eq!(
        value_type(&zero, &Scope::new(), &values, &typed),
        Some(scalar_type("Packet"))
    );
    assert!(value_type(&zero, &Scope::new(), &values, &flat).is_none());
    let mut inputs = BTreeSet::new();
    collect_inputs(&zero, &mut inputs);
    assert!(inputs.is_empty());
}

#[test]
fn typed_layouts_reject_cycles_resources_empty_and_duplicate_fields_transitively() {
    for mutation in ["cycle", "resource", "empty", "duplicate", "generic"] {
        let mut module = module();
        let leaf = module
            .structs
            .iter_mut()
            .find(|d| d.name == "Leaf")
            .unwrap();
        match mutation {
            "cycle" => leaf.fields[0].ty = scalar_type("Packet"),
            "resource" => leaf.fields[0].ty = scalar_type("Bytes"),
            "empty" => leaf.fields.clear(),
            "duplicate" => leaf.fields[1].name = leaf.fields[0].name.clone(),
            "generic" => leaf.fields[0].ty.generic_args.push(scalar_type("i64")),
            _ => unreachable!(),
        }
        let typed = TypedLayouts::collect(&module);
        assert!(!supported_type(&scalar_type("Leaf"), &typed), "{mutation}");
        assert!(
            !supported_type(&scalar_type("Packet"), &typed),
            "{mutation}"
        );
        assert!(supported_type(&scalar_type("Flat"), &typed));
    }
}

#[test]
fn typed_constructors_require_exact_leaf_types_unique_fields_and_nominal_records() {
    let module = module();
    let typed = TypedLayouts::collect(&module);
    for mutation in ["kind", "duplicate", "missing", "nominal", "generic"] {
        let mut value = zero_value(&scalar_type("Leaf"), &typed);
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut value
        else {
            unreachable!()
        };
        match mutation {
            "kind" => fields[1].1 = NirExpr::Int(0),
            "duplicate" => fields[1].0 = fields[0].0.clone(),
            "missing" => {
                fields.pop();
            }
            "nominal" => *type_name = "Flat".into(),
            "generic" => type_args.push(scalar_type("i64")),
            _ => unreachable!(),
        }
        assert!(
            value_type(&value, &Scope::new(), &ScalarHelpers::new(), &typed).is_none(),
            "{mutation}"
        );
    }
    assert!(binary_type(NirBinaryOp::Add, scalar_type("f32"), scalar_type("f32")).is_none());
}

#[test]
fn typed_layouts_bound_neutral_initializer_expansion_before_materializing_a_type_dag() {
    for (width, levels, accepted, rejected) in [(2, 20, 8, 12), (1, 70, 63, 64)] {
        let mut module = module();
        let template = module
            .structs
            .iter()
            .find(|d| d.name == "Leaf")
            .unwrap()
            .clone();
        for level in 0..levels {
            let mut definition = template.clone();
            definition.name = format!("R{level}");
            definition.fields = (0..width)
                .map(|field| {
                    let mut entry = template.fields[0].clone();
                    entry.name = format!("f{field}");
                    entry.ty = if level == 0 {
                        scalar_type("bool")
                    } else {
                        scalar_type(&format!("R{}", level - 1))
                    };
                    entry
                })
                .collect();
            module.structs.push(definition);
        }
        module.structs.reverse();
        let layouts = TypedLayouts::collect(&module);
        assert!(supported_type(
            &scalar_type(&format!("R{accepted}")),
            &layouts
        ));
        assert!(!supported_type(
            &scalar_type(&format!("R{rejected}")),
            &layouts
        ));
        assert!(!supported_type(
            &scalar_type(&format!("R{}", levels - 1)),
            &layouts
        ));
    }
}
