use super::*;

#[test]
fn dev_tensor_state_return_record_exit_retains_complete_nested_lineage() {
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
            "Previous state-return-nested-exit checkpoint",
            "f85debe9b73bf997ccc240c5deeba5f5b453dd4b4edd651996eae6b9adab47fc",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-nested-exit task checkpoint",
            "9b993feafa2312326b11d919c32822e24f00a03f0bd31f8d527193095b84c111",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-nested-exit blocker checkpoint",
            "cb78801b5d1f34676e59f38d5df1148692e0f0d8238fc51700df0ee6be8f0eee",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-nested-exit action checkpoint",
            "0812ceab0a83f35903ec44a8d4b2f2249da8c70dd099f4541124d4b527e5e925",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-nested-exit validation checkpoint",
            "93f7ce0c47658ca132849c8a0df72be36849cffb883ca8b04df93fb91058a02b",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-nested-exit artifact checkpoint",
            "44ee1c52bf7eaa4bbf09680563af79d882d81b654ea9fab1d63e6fa77cf7f032",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_RECORD_EXIT_20261111_NESTED_HISTORY";
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
fn dev_tensor_state_return_logical_exit_retains_complete_record_lineage() {
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
            "Previous state-return-record-exit checkpoint",
            "2bb7db70fa17fc73fd50302b3177e2440c128db5eff21360fa32127cf55796d9",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-record-exit task checkpoint",
            "6a74c397b41819c46190661e322cd245095cd476d46fbacd041df89fd7719f73",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-record-exit blocker checkpoint",
            "9d1c7580f9dac22679088d702ce051ff4c0aa52438821f654e51d1be11851aac",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-record-exit action checkpoint",
            "3e1ef1ef9dd799eab42293a317c4d745e09c567adabb313d322c1ddeff5ff316",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-record-exit validation checkpoint",
            "b3e61f8a659f5681486aa8f58f2e2ce2d6e4a2d619fbca6b0b5dccda7b3e79ae",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-record-exit artifact checkpoint",
            "aa1123fe98ec15108ca54ecf3a52651b37f9070f6ec7b15d56228deeda46ab1f",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_LOGICAL_EXIT_20261113_RECORD_HISTORY";
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
fn dev_tensor_state_return_local_exit_retains_complete_logical_lineage() {
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
            "Previous state-return-logical-exit checkpoint",
            "cee4aaf0f7aa4527f838c9ee363d37d1f58183603dde3a4aadda4a919c3d1cca",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-logical-exit task checkpoint",
            "3d3bfbd2b6cfd0a188203fe35c75bc9f1ec2fd7a7246e153263a0a10d3ec4624",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-logical-exit blocker checkpoint",
            "f747c823707e2ac72e02b5c17c9cd84dd1d6af1631f1489c9abe01c2c4b6155f",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-logical-exit action checkpoint",
            "2de7ded9ea1a8139ca258b617c325b98b4b60512bec6813c65f723ef18ac0cbc",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-logical-exit validation checkpoint",
            "665a98be5d14b76601fe8a89ed505c6052e5d5e82035c4b51b5549c059342cc2",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-logical-exit artifact checkpoint",
            "4508c4a14030895fe88b2c785fa67f7c0a0e131e02c33a66ca73b7167a6f6051",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_LOCAL_EXIT_20261113_LOGICAL_HISTORY";
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
fn dev_tensor_state_return_eager_exit_retains_complete_local_lineage() {
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
            "Previous state-return-local-exit checkpoint",
            "3cd11f64bd9fa030451fd8b1eaf7e64925d00ee52a399334647adcd2404adc81",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-local-exit task checkpoint",
            "c8b57940b1d0ea47b14d7a582f806462bc9f253cebe55e2cf590e757978e2460",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-local-exit blocker checkpoint",
            "b240e6e29207ce2183fb33365a024ee4537ba9fabde38675e6b986d81982662e",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-local-exit action checkpoint",
            "23b3ad4248cbe9f2373e4e043d02a07d74a62af9ce8c236e791d80292272946b",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-local-exit validation checkpoint",
            "bf517060b756f3a15e347e74075fc85c8c5fe1cea80bf9c1c4db1c5aeecd80c9",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-local-exit artifact checkpoint",
            "f43856bcc9e4318afdd33c16edb348fea8ea3b22adfcf3cdc3588d9e48fed9f0",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_STATE_RETURN_EAGER_EXIT_20261113_LOCAL_HISTORY";
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
fn dev_tensor_direct_effectful_records_retains_complete_eager_lineage() {
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
            "Previous state-return-eager-exit checkpoint",
            "d678514b4b5b01f557937624e20f883149db31a75de2e710e78078089e985ca2",
        ),
        (
            "next_step",
            cell.next_step,
            "Previous state-return-eager-exit task checkpoint",
            "4e34e3d8344d2beaf5926d7b8116c0e59c6f554244257c3caa3082d0cdd3860d",
        ),
        (
            "blocker",
            cell.blocker,
            "Previous state-return-eager-exit blocker checkpoint",
            "f03096940925628ec323f31dbbd5b200044087c0ea673277349802c578c6b360",
        ),
        (
            "next_action",
            cell.next_action,
            "Previous state-return-eager-exit action checkpoint",
            "659465c1d16e9bb00287a4cc0957ec049b99f625aed6d717a46832c24aea22f4",
        ),
        (
            "validation_command",
            cell.validation_command,
            "Previous state-return-eager-exit validation checkpoint",
            "48b1a8daaffb57931585eca58462f02a4541617a08b4199522477ef97a805803",
        ),
        (
            "expected_artifact",
            cell.expected_artifact,
            "Previous state-return-eager-exit artifact checkpoint",
            "9c781e24bf6f48e074dad24629673424033bfe17fd5b8c4477e7950e3df48313",
        ),
    ] {
        let marker = format!("{checkpoint}: SHA-256 {hash}\n");
        let (_, predecessor) = value
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {field} checkpoint"));
        let predecessor = if field == "validation_command" {
            let delimiter = "NUIS_DIRECT_EFFECTFUL_RECORDS_20261113_EAGER_HISTORY";
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
