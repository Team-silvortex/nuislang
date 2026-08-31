use std::path::PathBuf;

use super::{parse_args, BootstrapCandidateFreshSourceInput, CommandKind};

#[test]
fn parses_bootstrap_candidate_fresh_source_command() {
    let args =
        "bootstrap-candidate-fresh-source candidate-root successor source.ns result capability";
    let command = parse_args(args.split_whitespace().map(str::to_owned))
        .expect("bootstrap-candidate-fresh-source parses");
    let CommandKind::BootstrapCandidateFreshSource(BootstrapCandidateFreshSourceInput {
        candidate_root,
        successor,
        source,
        result_output,
        capability_output,
    }) = command
    else {
        panic!("expected candidate fresh-source command");
    };
    assert_eq!(candidate_root, PathBuf::from("candidate-root"));
    assert_eq!(successor, PathBuf::from("successor"));
    assert_eq!(source, PathBuf::from("source.ns"));
    assert_eq!(result_output, PathBuf::from("result"));
    assert_eq!(capability_output, PathBuf::from("capability"));
}
