use super::*;

#[test]
fn outer_flat_carries_preserve_seeds_snapshots_and_order_on_default_native_entry() {
    let source = "mod cpu Main {
        struct Pair { left: i64, right: i64 }
        struct Marker { value: i64 }
        @noinline fn work(seed: i64, limit: i64) -> i64 {
            let packet = Pair { left: seed, right: seed + 1 };
            let saved = packet;
            let marker = Marker { value: 5 };
            let index: i64 = 0;
            let total: i64 = seed;
            while index < limit {
                let index: i64 = index + 1;
                let packet = packet;
                let before = packet;
                let packet = Pair { right: packet.left + index, left: packet.right + index };
                if index < limit { let packet = Pair { left: packet.left + 3, right: packet.right + 4 }; }
                else { let packet = before; }
                let marker = Marker { value: marker.value + packet.left };
                let total: i64 = total + marker.value + saved.right;
            }
            return packet.left + packet.right + marker.value + total + saved.left;
        }
        fn main() -> i64 { return work(1, 3) + work(1, 0); }
    }";
    assert_eq!(
        compile_and_run("outer_flat_carries_entry", source).code(),
        Some(140)
    );
}

#[test]
fn single_word_record_self_copy_is_not_mistaken_for_scalar_or_owned_resource_loop() {
    let source = "mod cpu Main {
        struct Marker { value: i64 }
        @noinline fn work(limit: i64, value: i64) -> i64 {
            let marker = Marker { value: value };
            let index: i64 = 0;
            while index < limit { let index: i64 = index + 1; let marker = marker; }
            return marker.value + index;
        }
        fn main() -> i64 { return work(3, 7) + work(0, 11); }
    }";
    assert_eq!(
        compile_and_run("single_word_carry_entry", source).code(),
        Some(21)
    );
}
