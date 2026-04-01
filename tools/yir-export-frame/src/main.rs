use std::{env, fs, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut stdout_ppm = false;
    let first = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-export-frame -- [--stdout-ppm] <module.yir> <output.ppm> [scale]"
            .to_owned()
    })?;
    let input = if first == "--stdout-ppm" {
        stdout_ppm = true;
        args.next().ok_or_else(|| {
            "usage: cargo run -p yir-export-frame -- [--stdout-ppm] <module.yir> <output.ppm> [scale]"
                .to_owned()
        })?
    } else {
        first
    };
    let output = args.next().ok_or_else(|| {
        "usage: cargo run -p yir-export-frame -- [--stdout-ppm] <module.yir> <output.ppm> [scale]"
            .to_owned()
    })?;
    let scale = args
        .next()
        .map(|raw| {
            raw.parse::<usize>()
                .map_err(|_| format!("invalid scale `{raw}`"))
        })
        .transpose()?
        .unwrap_or(16);

    let source =
        fs::read_to_string(&input).map_err(|error| format!("failed to read `{input}`: {error}"))?;
    let ppm = yir_runtime_host::render_module_to_ppm_bytes(&source, scale)?;
    if stdout_ppm {
        use std::io::Write;
        std::io::stdout()
            .write_all(&ppm)
            .map_err(|error| format!("failed to write PPM to stdout: {error}"))?;
    } else {
        fs::write(&output, &ppm).map_err(|error| format!("failed to write `{output}`: {error}"))?;
    }

    let (width, height) = parse_ppm_dimensions(&ppm)?;

    eprintln!("exported frame {}x{} to {}", width, height, output);
    Ok(())
}

fn parse_ppm_dimensions(ppm: &[u8]) -> Result<(usize, usize), String> {
    let mut index = 0usize;

    fn next_token(ppm: &[u8], index: &mut usize) -> Option<String> {
        while *index < ppm.len() {
            let byte = ppm[*index];
            if byte == b'#' {
                while *index < ppm.len() && ppm[*index] != b'\n' {
                    *index += 1;
                }
            } else if byte.is_ascii_whitespace() {
                *index += 1;
            } else {
                break;
            }
        }

        let start = *index;
        while *index < ppm.len() && !ppm[*index].is_ascii_whitespace() && ppm[*index] != b'#' {
            *index += 1;
        }
        if start == *index {
            return None;
        }
        String::from_utf8(ppm[start..*index].to_vec()).ok()
    }

    let magic = next_token(ppm, &mut index).ok_or_else(|| "generated invalid ppm header".to_owned())?;
    if magic != "P6" {
        return Err("generated invalid ppm magic".to_owned());
    }
    let width = next_token(ppm, &mut index)
        .ok_or_else(|| "generated missing ppm width".to_owned())?
        .parse::<usize>()
        .map_err(|_| "generated invalid ppm width".to_owned())?;
    let height = next_token(ppm, &mut index)
        .ok_or_else(|| "generated missing ppm height".to_owned())?
        .parse::<usize>()
        .map_err(|_| "generated invalid ppm height".to_owned())?;
    Ok((width, height))
}
