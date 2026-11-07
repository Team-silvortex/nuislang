use super::*;

#[test]
fn dev_tensor_state_control_exit_retains_complete_record_parameter_lineage() {
    use sha2::{Digest, Sha256};
    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|cell| {
            cell.architecture == "standard-library"
                && cell.module == "ns-nova"
                && cell.function == "persistent-application-session"
        })
        .unwrap();
    for (field, value, checkpoint, hash) in [
        (
            "evidence",
            cell.evidence,
            "Previous state-record-parameter checkpoint",
            "ad64c9caf53f604ba8a8dbc0c608976155bb9ec080eed2dfd18a575ece4793c8",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-record-parameter task checkpoint",
            "d25af46d361764de5fc7af28c6845aa518e093ba9235142486ca22ada8edcac2",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-record-parameter blocker checkpoint",
            "119d7ce1b42c1b708ed83461347837e026cfb1e77f5319ee92cc8c34c24c6268",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-record-parameter action checkpoint",
            "209bb36dcde337306cfe2cc09c060f6dd6af69312bfe73c100da8cdac82ab0bf",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-record-parameter validation checkpoint",
            "331d8edb3c6992a2e8a66d583bc403d0b64bd3d15b43c8a4806791172cfb496c",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-record-parameter artifact checkpoint",
            "e1b7e8714f40d6ecab0737acc3f0bb0243099c34e511cd40cb3101500747759d",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_CONTROL_EXIT_20261107_RECORD_PARAMETER_HISTORY";
            predecessor
                .strip_prefix(&format!(": <<'{delimiter}'\n"))
                .unwrap()
                .split_once(&format!("\n{delimiter}"))
                .unwrap()
                .0
        } else {
            predecessor
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(predecessor.as_bytes())),
            hash,
            "changed complete {field} predecessor"
        );
    }
}

#[test]
fn dev_tensor_state_record_parameter_exit_retains_complete_parameter_lineage() {
    use sha2::{Digest, Sha256};

    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|cell| {
            cell.architecture == "standard-library"
                && cell.module == "ns-nova"
                && cell.function == "persistent-application-session"
        })
        .unwrap();
    for (field, value, checkpoint, hash) in [
        (
            "evidence",
            cell.evidence,
            "Previous state-parameter-exit checkpoint",
            "db09b92bb1e678db5b62a924ed9d57fda653257f2e7a912e3d6bb2b47faf4998",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-parameter-exit task checkpoint",
            "5001213779cb55d5f56a89b19cff6a00e44c37df255a20c97edabf05ec1b2d39",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-parameter-exit blocker checkpoint",
            "e146d17c077a272b15730f280091ae11bdfa5bdbb6babc564c301d74105d0888",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-parameter-exit action checkpoint",
            "ea6bf29f4aca0115b275be2224d474265d3665cf3db646ee7b6d1cd930ae48cc",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-parameter-exit validation checkpoint",
            "187b1452289be1ca7dab7ecf5a76c04cf2e688379f810ba01209acf65836350e",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-parameter-exit artifact checkpoint",
            "2197dc2e11d723d79e10d23cb41f531d2ada9b4cb2e6788a06aed77d5517c711",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RECORD_PARAMETER_EXIT_20261107_PARAMETER_HISTORY";
            predecessor
                .strip_prefix(&format!(": <<'{delimiter}'\n"))
                .unwrap()
                .split_once(&format!("\n{delimiter}"))
                .unwrap()
                .0
        } else {
            predecessor
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(predecessor.as_bytes())),
            hash,
            "changed complete {field} predecessor"
        );
    }
}

#[test]
fn dev_tensor_state_parameter_exit_retains_complete_helper_exit_lineage() {
    use sha2::{Digest, Sha256};

    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|cell| {
            cell.architecture == "standard-library"
                && cell.module == "ns-nova"
                && cell.function == "persistent-application-session"
        })
        .unwrap();
    for (field, value, checkpoint, hash) in [
        (
            "evidence",
            cell.evidence,
            "Previous state-helper-exit checkpoint",
            "2d873db1233bf9d11e1c4dd382324f202bcc7d311fc0885c6721de7f846df1c3",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-helper-exit task checkpoint",
            "7d387bb7117daf185d5b4627b9b57e4b3decc71ef777a8db27f3c58571552de6",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-helper-exit blocker checkpoint",
            "38797a0a7b107bb5a38a5eac8a388d26faad84c7c0b5893ee73c7fcca0ad56d0",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-helper-exit action checkpoint",
            "18680433a47202e69b559d368fa950dc1fab4835f1e30a478acbbf3b4394da1f",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-helper-exit validation checkpoint",
            "6cd903b22989f87f96421f9481c8388bdd85d181b4cec1ab29b58df93630dfe4",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-helper-exit artifact checkpoint",
            "b0b5fb87120e78604158c7aaf4b849195db4aafcf659e31af55ada211c2d07c3",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_PARAMETER_EXIT_20261107_HELPER_EXIT_HISTORY";
            predecessor
                .strip_prefix(&format!(": <<'{delimiter}'\n"))
                .unwrap()
                .split_once(&format!("\n{delimiter}"))
                .unwrap()
                .0
        } else {
            predecessor
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(predecessor.as_bytes())),
            hash,
            "changed complete {field} predecessor"
        );
    }
}

#[test]
fn dev_tensor_state_helper_exit_retains_complete_division_exit_lineage() {
    use sha2::{Digest, Sha256};

    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|cell| {
            cell.architecture == "standard-library"
                && cell.module == "ns-nova"
                && cell.function == "persistent-application-session"
        })
        .unwrap();
    for (field, value, checkpoint, hash) in [
        (
            "evidence",
            cell.evidence,
            "Previous state-division-exit checkpoint",
            "ed49db10cdfcb14135c9bdc51e5f56a3d4eb3647a0a0fba020782f4a1cd0218f",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-division-exit task checkpoint",
            "1c889bcf26f65d6093d6db15d261a319bb0a05715e6bebfed0a10c6232972573",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-division-exit blocker checkpoint",
            "14c19f1dc8c0e2f5c2b4eadc35084726c70f012fe9d7c51a33b06e2dd1198e6c",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-division-exit action checkpoint",
            "176cfcfd297b3acf2c55bb39d88112ce7bdc39e10b3cdf7ec88c52c42b884ca8",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-division-exit validation checkpoint",
            "b343c2c3c5a1de1495a8142200e85e6ee83654402372dcc510369660c2e56e56",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-division-exit artifact checkpoint",
            "0db81cf97516cb09a1e9855c1cc8db9eeec929dd088aab2540605ba6261cf447",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_HELPER_EXIT_20261105_DIVISION_EXIT_HISTORY";
            predecessor
                .strip_prefix(&format!(": <<'{delimiter}'\n"))
                .unwrap()
                .split_once(&format!("\n{delimiter}"))
                .unwrap()
                .0
        } else {
            predecessor
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(predecessor.as_bytes())),
            hash,
            "changed complete {field} predecessor"
        );
    }
}
