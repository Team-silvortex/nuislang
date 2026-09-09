use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn inline_collection_rejects_effectful_prefix_before_expansion() {
    for prefix in ["store_at(buffer, 0, seed);", "if seed < 0 { return 0; }"] {
        let body = "let seed: i64 = seed + seed;".repeat(32);
        let source = format!(
            "mod cpu Main {{ fn candidate(buffer: ref Buffer, seed: i64) -> i64 {{
                {prefix} {body} return seed;
            }} fn main() -> i64 {{ return 0; }} }}"
        );
        let module = parse_nuis_module(&source).unwrap();
        assert!(!collect_inlineable_pure_helper_exprs(&module).contains_key("candidate"));
    }
}

#[test]
fn inline_collection_bounds_pure_rebinding_expansion_without_losing_purity() {
    let body = "let seed: i64 = seed + seed;".repeat(32);
    let source = format!(
        "mod cpu Main {{ fn candidate(seed: i64) -> i64 {{ {body} return seed; }}
            fn small(seed: i64) -> i64 {{ let next: i64 = seed + 1; return next + next; }}
            fn main() -> i64 {{ return 0; }} }}"
    );
    let module = parse_nuis_module(&source).unwrap();
    let helpers = collect_inlineable_pure_helper_exprs(&module);
    assert!(!helpers.contains_key("candidate"));
    assert!(helpers.contains_key("small"));
    assert!(collect_pure_helper_functions(&module).contains("candidate"));
    assert!(collect_pure_helper_blocks(&module).contains_key("candidate"));
}

#[test]
fn inline_collection_does_not_classify_effectful_return_as_pure() {
    let module = parse_nuis_module(
        "mod cpu Main { fn candidate(seed: i64) -> ref Buffer { return alloc_buffer(seed, 0); }
            fn main() -> i64 { return 0; } }",
    )
    .unwrap();
    assert!(!collect_inlineable_pure_helper_exprs(&module).contains_key("candidate"));
}
