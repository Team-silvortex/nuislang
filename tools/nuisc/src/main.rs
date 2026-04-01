use std::env;

fn main() {
    if let Err(error) = run_main() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run_main() -> Result<(), String> {
    let command = nuisc::cli::parse_args(env::args().skip(1))?;
    nuisc::run(command)
}
