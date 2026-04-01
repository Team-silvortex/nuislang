use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| "usage: cargo run -p yir-run -- <module.yir>".to_owned())?;
    let source =
        fs::read_to_string(&path).map_err(|error| format!("failed to read `{path}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    yir_verify::verify_module(&module)?;
    let trace = yir_exec::execute_module(&module)?;

    println!("yir version: {}", module.version);
    println!("resources: {}", module.resources.len());
    println!("nodes: {}", module.nodes.len());
    println!("edges: {}", module.edges.len());
    println!("trace:");
    for event in &trace.events {
        println!("  {event}");
    }
    println!("steps:");
    for (lane, steps) in &trace.lane_steps {
        println!("  {lane}:");
        for step in steps {
            println!("    {step}");
        }
    }
    println!("lanes:");
    for (lane, events) in &trace.lane_events {
        println!("  {lane}:");
        for event in events {
            println!("    {event}");
        }
    }
    println!("values:");
    for (name, value) in &trace.values {
        println!("  {name} = {value}");
    }

    Ok(())
}
