use super::*;

const SOURCE: &str = "mod cpu Main {
    struct Pair { left: i64, right: i64 }
    @noinline fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
    @noinline fn work(limit: i64) -> i64 {
        let index: i64 = 0;
        let total: i64 = 1;
        while index < limit {
            let index: i64 = index + 1;
            let total: i64 = total;
            let packet = Pair { left: total, right: index };
            let saved = packet;
            let packet = Pair { right: packet.right + 2, left: packet.left + index };
            let shifted = packet;
            let flag = index < limit;
            if flag { let packet = Pair { left: packet.left + 3, right: packet.right + 4 }; }
            else { let packet = saved; }
            if flag == false { let packet = shifted; }
            let packet = Pair { left: packet.right, right: packet.left };
            if limit == 1 {
                let packet = Pair { left: checked(index, limit - 1), right: 0 };
                let packet = saved;
            }
            let total: i64 = packet.right + saved.left + shifted.right;
        }
        return total;
    }
    fn main() -> i64 { return work(3) + work(0); }
}";

#[test]
fn flat_rebinding_snapshots_and_joins_run_through_default_native_entry() {
    assert_eq!(
        compile_and_run("flat_rebinding_entry", SOURCE).code(),
        Some(63)
    );
}

#[test]
fn overwritten_flat_result_keeps_native_arithmetic_failure() {
    let source = SOURCE.replace("return work(3) + work(0);", "return work(1);");
    let status = compile_and_run("overwritten_flat_entry", &source);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}
