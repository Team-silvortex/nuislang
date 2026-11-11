use super::*;

#[test]
fn dev_tensor_state_return_prefix_exit_retains_complete_join_lineage() {
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
            "Previous state-return-join-exit checkpoint",
            "3f276e21b374afce828c9d2ffe23549302fa378e1e23684218bc308657ef61ab",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-join-exit task checkpoint",
            "10e286c8506f0b58018d4506870b0048b00792bd35934edae2949b8046fc2fc8",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-join-exit blocker checkpoint",
            "a2caa354d0d7d8a1d9dfc3868badb599be9f27ff7faedefabccb025c827b9fe5",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-join-exit action checkpoint",
            "33b2fd4e45b23cc1cdca8d93b0708fddb2c4d9350fc3a6b5ad39dd5890edb5e5",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-join-exit validation checkpoint",
            "a38dac0dcbeec27e907ed9a93036d6227d3baa903a05443e001127d5a50c397a",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-join-exit artifact checkpoint",
            "554ec9b1e5c5025df30b52415df96830e6e15d8cc44e8c6f35066101f48d8946",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_PREFIX_EXIT_20261111_JOIN_HISTORY";
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
fn dev_tensor_state_return_join_exit_retains_complete_gate_lineage() {
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
            "Previous state-return-gate-exit checkpoint",
            "4e7fe4eea5c966d31e342ac12a509deb2c901d37ef0ec3eb98b2a52b6748a433",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-gate-exit task checkpoint",
            "5bc4c157d5943f64a5868bd5c9423e7f3b49df959cffaa3d9836e68004109efe",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-gate-exit blocker checkpoint",
            "3b48706889da626e4700ce5b63a5d55d1f6064b81935b276ed670aa8485a40e4",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-gate-exit action checkpoint",
            "ab8b23ed5594577e6ecd2568690197c20c2fc1164fa0389b58d27d59a1f2f7fe",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-gate-exit validation checkpoint",
            "a9cd8d04ad65b7c7d71a0e881543b38a1caa49a1145e28421c4e05cb77801bde",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-gate-exit artifact checkpoint",
            "fcfc330ec58400e6f2fe31f395387ab699f9502fd33f36d3f7eb814f4d742a58",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_JOIN_EXIT_20261111_GATE_HISTORY";
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
fn dev_tensor_state_return_gate_exit_retains_complete_chain_lineage() {
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
            "Previous state-branch-chain-exit checkpoint",
            "3ebd4df21ed2de1c64ece468acea6c38884683d4cd39bae60a1b7b5353253ade",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-branch-chain-exit task checkpoint",
            "48d2ab8eb3ea6e105cc2be043337b399b231d77157b70b45e68e4cda20230e88",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-branch-chain-exit blocker checkpoint",
            "594e3206a23650659367c67d315d5e9e2fa13f72747175ccc0787c1e0934a0c5",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-branch-chain-exit action checkpoint",
            "a3ebd5014550f667708e9133e2c5ddc27d225e769dfc28d23fe7fa489391cd62",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-branch-chain-exit validation checkpoint",
            "254d26199082adb338cc1df730fa34a98519b4ee86b6bd7f39e11874d4cbf8dd",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-branch-chain-exit artifact checkpoint",
            "4536c3ecd87f4b8698629c4de02b9cce8d1a678d52a78f9b0905f82a4f1da7ae",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_GATE_EXIT_20261111_CHAIN_HISTORY";
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
fn dev_tensor_state_branch_chain_exit_retains_complete_helper_lineage() {
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
            "Previous state-branch-helper-exit checkpoint",
            "dc20d6f6c37d76e6c7c2b77a42b9d7e0139a9cf5f5b0547cc60978c16d16c787",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-branch-helper-exit task checkpoint",
            "4fc2c76166fdb30344926d56091d9e11c5cba22549fdcc51d4e0dd9613d9f15f",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-branch-helper-exit blocker checkpoint",
            "5013b4cbf8d938c4f57a06529879b2f0cfa661b14d971ccdf1cadf34748365be",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-branch-helper-exit action checkpoint",
            "800c0e1608c197256b944f2409c6047059d1224bb2208e04fcd52dae34675c16",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-branch-helper-exit validation checkpoint",
            "61f252aef12291edd4735fccedebfb4d457290b0ddeceaf09cce2569bd2da318",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-branch-helper-exit artifact checkpoint",
            "de66ede314811a7fd48daf26f37b3df3eff6a13c4697ffb3c94715f8205f4e40",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_BRANCH_CHAIN_EXIT_20261111_HELPER_HISTORY";
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
fn dev_tensor_state_branch_helper_exit_retains_complete_branch_lineage() {
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
            "Previous state-branch-control-exit checkpoint",
            "8f1274da6499f40f92dd2f95e245c35ce3451bc874b0727d4b11c347aa9ab5f3",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-branch-control-exit task checkpoint",
            "b49e466d815714bf50f8679bf88322f3dad879dbb5fee1d81ee1b0a0f01de330",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-branch-control-exit blocker checkpoint",
            "a616abd648c3dd15061c3aa2eca0e31ac8cce2176abaac7aa25ee265dba3df86",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-branch-control-exit action checkpoint",
            "619b201e391a54b1dfbbf66372f538e557c7d31b07889d01dddb74936063d838",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-branch-control-exit validation checkpoint",
            "f4ce33c928ecb4e8a1f4cd6448d53b828c7d66f535b2950cee75c474a2b55c2f",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-branch-control-exit artifact checkpoint",
            "45ec6238e69386ce4192323b54beb02364bf1fcf9957a644f870624fddfcbe01",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_BRANCH_HELPER_EXIT_20261111_BRANCH_HISTORY";
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
fn dev_tensor_state_branch_control_exit_retains_complete_checked_lineage() {
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
            "Previous state-checked-control-exit checkpoint",
            "648a1fa4ede8d76f311b883314aa9b6594b9046abc11f63dd189180f1b7a7698",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-checked-control-exit task checkpoint",
            "c6ab187b593b83ac7899bd09e2c746e1822ec862ef0a48b23aca8d89c6e9816c",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-checked-control-exit blocker checkpoint",
            "2ba55c8f4d98195b81d03735e9a160f6a99e068759eae1a68234075359bf30ea",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-checked-control-exit action checkpoint",
            "46c0e9f4533b980138849a846f0c600784ca2f864611916168fe6ffd7fea5a94",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-checked-control-exit validation checkpoint",
            "bda5c4a25cf1f86e7d293b63c8705f28bf85deb572cacd98decf59d12cb561dc",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-checked-control-exit artifact checkpoint",
            "31df8cc484b576e896a4a3cd67800b10f1f158822929c54f9bbf317d845d290e",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_BRANCH_CONTROL_EXIT_20261111_CHECKED_HISTORY";
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
fn dev_tensor_state_checked_control_exit_retains_complete_control_lineage() {
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
            "Previous state-control-exit checkpoint",
            "aa05ce5c46222c478d014a3b79f9664f71cd21c2239c500769735c4c84eb844d",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-control-exit task checkpoint",
            "6e43c43c2fa77931d5c3ffbb0d9e045912f4d0bc0d9d70814d4e42ab4e85fb30",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-control-exit blocker checkpoint",
            "4b8043312f6d2a366aa82281638cc6c37d54e7f3461b0b430450e28b35264810",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-control-exit action checkpoint",
            "183df07ea7ae339c8ba653770a890821fd8498f31b2e522078f7b6a37fb4f8c1",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-control-exit validation checkpoint",
            "c4d94b5d842624556b3dee91152a02f743d04f899f8f0b180218cfd1647d8bf2",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-control-exit artifact checkpoint",
            "b58763494f2f72e4587961a17c5489fd866eba6af8ce2c5575a3eefac606d780",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_CHECKED_CONTROL_EXIT_20261107_CONTROL_HISTORY";
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
