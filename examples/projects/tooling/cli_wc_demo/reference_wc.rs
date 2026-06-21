use std::hint::black_box;
use std::time::Instant;

const FIXTURE: &str = "alpha beta\ngamma delta\nepsilon zeta eta\n";

fn const_fixture() -> usize {
    7
}

fn bridge_fixture() -> usize {
    let bytes = FIXTURE.as_bytes();
    let restored = std::str::from_utf8(bytes).expect("fixture should stay valid utf-8");
    black_box(restored.len())
}

fn number_text_fixture() -> usize {
    let rendered = 123456789_i64.to_string();
    black_box(rendered.len())
}

fn concat_fixture() -> usize {
    let merged = ["alpha beta\n", "gamma delta\n"].concat();
    black_box(merged.len())
}

fn scan_fixture() -> usize {
    let bytes = FIXTURE.as_bytes();
    let restored = std::str::from_utf8(bytes).expect("fixture should stay valid utf-8");
    black_box(bytes.len().wrapping_add(restored.len()))
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "scan".to_owned());
    let iterations = std::env::args()
        .nth(2)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8);

    let started = Instant::now();
    let mut sink = 0usize;
    for _ in 0..iterations {
        let value = match mode.as_str() {
            "const" => const_fixture(),
            "bridge" => bridge_fixture(),
            "number_text" => number_text_fixture(),
            "concat" => concat_fixture(),
            "scan" => scan_fixture(),
            other => panic!("unknown mode: {}", other),
        };
        sink = sink.wrapping_add(value);
    }
    let total_ns = started.elapsed().as_nanos() as u64;
    let avg_ns = if iterations == 0 {
        0
    } else {
        total_ns / iterations as u64
    };

    println!(
        "{{\"kind\":\"rust_fixture_benchmark\",\"mode\":\"{}\",\"iterations\":{},\"avg_ns\":{},\"total_ns\":{},\"sink\":{}}}",
        mode, iterations, avg_ns, total_ns, sink
    );
}
