use super::*;
use crate::frontend::parse_nuis_module;

pub(super) const SOURCE: &str = "mod cpu Main {
  struct Pair { value: i64, seed: i64 }
  fn walk(initial: i64, bound: i64, step: i64) -> i64 {
    let limit: i64 = bound;
    let stride: i64 = step;
    let index: i64 = initial;
    let total: i64 = initial;
    let checksum: i64 = 1;
    while index < limit {
      let index: i64 = index + stride;
      let total: i64 = total + index;
      let checksum: i64 = checksum * total;
    }
    return index + total + checksum;
  }
  fn wrap(a: i64, b: i64, step: i64) -> i64 { return walk(a, b, step); }
  fn choose(flag: bool, a: i64, b: i64, step: i64) -> Pair {
    if flag { return Pair { value: wrap(a, b, step), seed: a }; }
    return Pair { value: a, seed: b };
  }
  fn main() -> i64 { return 0; }
}";

#[test]
fn ordered_carries_share_preparation_without_widening_buffer_catalog() {
    let mut module = parse_nuis_module(SOURCE).unwrap();
    let layouts = control_values::layouts(&module);
    for _ in 0..2 {
        let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
        for name in ["walk", "wrap", "choose"] {
            assert!(catalog[name].may_loop, "{name}");
        }
        assert_eq!(
            scalar_helpers::collect(&module).keys().collect::<Vec<_>>(),
            ["main"]
        );
        module.functions.reverse();
    }
    let outlined = outline_buffer_loops(&mut module).unwrap();
    assert!(outlined.functions.contains("walk"));
    assert!(outlined.functions.contains("choose"));
    assert_eq!(outlined.guarded_functions.len(), 2);
}

#[test]
fn carry_catalog_is_not_a_finite_backend_profile_table() {
    // The native bridge owns its slot limit; the source catalog owns shape/type.
    for count in [1, 3, 7, 65] {
        let mut seeds = String::new();
        let mut updates = String::new();
        for slot in 0..count {
            seeds.push_str(&format!("let carry{slot}: i64 = initial;"));
            let rhs = if slot == 0 {
                "index".into()
            } else {
                format!("carry{}", slot - 1)
            };
            updates.push_str(&format!("let carry{slot}: i64 = carry{slot} + {rhs};"));
        }
        let source = SOURCE
            .replace("let total: i64 = initial;", &seeds)
            .replace("let checksum: i64 = 1;", "")
            .replace("let total: i64 = total + index;", &updates)
            .replace("let checksum: i64 = checksum * total;", "")
            .replace(
                "index + total + checksum",
                &format!("index + carry{}", count - 1),
            );
        let module = parse_nuis_module(&source).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(catalog["choose"].may_loop, "{count}");
    }
}

#[test]
fn carries_reject_seed_order_effect_and_header_mutation_drift() {
    for (from, to) in [
        ("let total: i64 = initial;", "const total: i64 = initial;"),
        ("let total: i64 = initial;", "let total: i64 = initial / 1;"),
        ("total + index", "total + checksum"),
        ("total + index", "total + (index / stride)"),
        ("total + index", "total + (index % stride)"),
        ("total + index", "total + wrap(index, limit, stride)"),
        ("let checksum: i64 = checksum * total;", "print(total);"),
        (
            "let checksum: i64 = checksum * total;",
            "let total: i64 = total + index;",
        ),
        (
            "let checksum: i64 = checksum * total;",
            "let limit: i64 = limit + index;",
        ),
        (
            "let checksum: i64 = checksum * total;",
            "let stride: i64 = stride + index;",
        ),
        (
            "let checksum: i64 = checksum * total;",
            "let bound: i64 = bound + index;",
        ),
        (
            "let total: i64 = total + index;",
            "if index > 0 { let total: i64 = total + index; print(total); }",
        ),
        (
            "let total: i64 = total + index;",
            "while total < limit { let total: i64 = total + 1; }",
        ),
        (
            "let index: i64 = index + stride;",
            "let checksum: i64 = checksum + index; let index: i64 = index + stride;",
        ),
    ] {
        // A fallible seed remains valid: unlike body updates, it executes once
        // in source order and is protected by ordinary checked arithmetic.
        let source = SOURCE.replace(from, to);
        let module = parse_nuis_module(&source).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        for name in ["walk", "wrap", "choose"] {
            assert_eq!(
                catalog.contains_key(name),
                to == "let total: i64 = initial / 1;",
                "{name}: {to}"
            );
        }
    }
}

#[test]
fn carry_updates_require_exact_i64_declarations_and_values() {
    let module = parse_nuis_module(SOURCE).unwrap();
    let mut unseeded = module.clone();
    unseeded
        .functions
        .iter_mut()
        .find(|f| f.name == "walk")
        .unwrap()
        .body
        .retain(|stmt| !matches!(stmt, NirStmt::Let { name, .. } if name == "total"));
    let catalog =
        scalar_helpers::collect_with_layouts(&unseeded, &control_values::layouts(&unseeded));
    assert!(!catalog.contains_key("walk"));
    assert!(!catalog.contains_key("choose"));
    for mutate_seed in [false, true] {
        let mut changed = module.clone();
        let walk = changed
            .functions
            .iter_mut()
            .find(|f| f.name == "walk")
            .unwrap();
        let body = if mutate_seed {
            &mut walk.body
        } else {
            let NirStmt::While { body, .. } = walk
                .body
                .iter_mut()
                .find(|stmt| matches!(stmt, NirStmt::While { .. }))
                .unwrap()
            else {
                unreachable!()
            };
            body
        };
        let NirStmt::Let { ty, .. } = body
            .iter_mut()
            .find(|stmt| matches!(stmt, NirStmt::Let { name, .. } if name == "total"))
            .unwrap()
        else {
            unreachable!()
        };
        *ty = Some(scalar_type("i32"));
        let catalog =
            scalar_helpers::collect_with_layouts(&changed, &control_values::layouts(&changed));
        assert!(!catalog.contains_key("walk"));
        assert!(!catalog.contains_key("choose"));
    }
}
