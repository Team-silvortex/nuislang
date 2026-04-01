use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // This tool is an adapter for the current CPU-hosted preview path.
    // It intentionally interprets a subset of `cpu`-mod ops into a UI plan
    // without making those ops part of YIR core semantics.
    let mut args = env::args().skip(1);
    let input = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-export-ui-plan -- <module.yir> <output.plan>".to_owned()
    })?;
    let output = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-export-ui-plan -- <module.yir> <output.plan>".to_owned()
    })?;

    let source =
        fs::read_to_string(&input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;

    let mut lines = Vec::new();

    for node in &module.nodes {
        match (node.op.module.as_str(), node.op.instruction.as_str()) {
            ("cpu", "window") => {
                if node.op.args.len() == 3 {
                    lines.push(format!("window.title={}", node.op.args[2]));
                    lines.push(format!("window.width={}", node.op.args[0]));
                    lines.push(format!("window.height={}", node.op.args[1]));
                }
            }
            ("cpu", "input_i64") => {
                if node.op.args.len() == 2 {
                    lines.push(format!("input={},{}", node.op.args[0], node.op.args[1]));
                }
            }
            _ => {}
        }
    }

    if lines.is_empty() {
        return Err("no cpu.window or cpu.input_i64 nodes found in module".to_owned());
    }

    fs::write(&output, lines.join("\n"))
        .map_err(|error| format!("failed to write `{output}`: {error}"))?;
    println!("exported ui plan to {}", output);
    Ok(())
}
