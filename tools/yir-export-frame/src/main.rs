use std::{env, fs, process};

use yir_core::Value;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .ok_or_else(|| "usage: cargo run -p yir-export-frame -- <module.yir> <output.ppm> [scale]".to_owned())?;
    let output = args
        .next()
        .ok_or_else(|| "usage: cargo run -p yir-export-frame -- <module.yir> <output.ppm> [scale]".to_owned())?;
    let scale = args
        .next()
        .map(|raw| raw.parse::<usize>().map_err(|_| format!("invalid scale `{raw}`")))
        .transpose()?
        .unwrap_or(16);

    let source = fs::read_to_string(&input)
        .map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let module = yir_syntax::parse_module(&source)?;
    yir_verify::verify_module(&module)?;
    let trace = yir_exec::execute_module(&module)?;

    let frame = trace
        .values
        .values()
        .find_map(|value| match value {
            Value::Frame(frame) => Some(frame),
            _ => None,
        })
        .ok_or_else(|| "no frame value found in executed YIR graph".to_owned())?;

    let image = yir_host_render::rasterize_frame(frame, scale);
    fs::write(&output, image.to_ppm())
        .map_err(|error| format!("failed to write `{output}`: {error}"))?;

    println!(
        "exported frame {}x{} to {}",
        image.width, image.height, output
    );
    Ok(())
}
