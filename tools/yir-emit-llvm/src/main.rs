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
        .ok_or_else(|| "usage: cargo run -p yir-emit-llvm -- <module.yir>".to_owned())?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{path}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    let llvm_ir = yir_lower_llvm::emit_module(&module)?;
    println!("{llvm_ir}");
    Ok(())
}
