mod cli;
mod display;
mod handoff;
mod hetero_trace;
mod json;
mod model;
mod replay;
mod report;
mod sidecar;

use crate::{
    cli::{parse_args, resolve_manifest_input, Command},
    display::{print_nsdb_events_report, print_nsdb_inspect_report, print_nsdb_replay_plan},
    json::{nsdb_events_report_json, nsdb_inspect_report_json, nsdb_replay_plan_json},
    report::nsdb_inspect_report,
};
use std::{env, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Status => {
            println!("Nsdb YIR debugger front-door");
            println!("  tool: nsdb");
            println!("  phase: alpha-0.6.0 debugger metadata boundary");
            println!("  debug_model: yir-metadata");
            println!("  native_debugger_visibility: host-shell-only");
            println!("  nsdb_visibility: yir domains, clock edges, data segments, lowering units");
        }
        Command::Inspect {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_inspect_report_json(&report));
            } else {
                print_nsdb_inspect_report(&report);
            }
        }
        Command::Events {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_events_report_json(&report));
            } else {
                print_nsdb_events_report(&report);
            }
        }
        Command::ReplayPlan {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_replay_plan_json(&report));
            } else {
                print_nsdb_replay_plan(&report);
            }
        }
    }
    Ok(())
}
