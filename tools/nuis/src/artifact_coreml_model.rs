const INPUT_FEATURE: &str = "input.features";
const OUTPUT_FEATURE: &str = "output.features";

pub(crate) fn witsage_vector_affine_model() -> Vec<u8> {
    let input = feature_description(INPUT_FEATURE, &[1, 1, 4]);
    let output = feature_description(OUTPUT_FEATURE, &[1, 1, 4]);

    let mut description = Vec::new();
    push_message(&mut description, 1, &input);
    push_message(&mut description, 10, &output);

    let mut scale = Vec::new();
    push_packed_varints(&mut scale, 1, &[1]);
    push_message(&mut scale, 2, &weight_params(&[2.0]));
    push_varint_field(&mut scale, 3, 1);
    push_packed_varints(&mut scale, 4, &[1]);
    push_message(&mut scale, 5, &weight_params(&[1.0]));

    let mut layer = Vec::new();
    push_string(&mut layer, 1, "witsage.vector.affine");
    push_string(&mut layer, 2, INPUT_FEATURE);
    push_string(&mut layer, 3, OUTPUT_FEATURE);
    push_message(&mut layer, 245, &scale);

    let mut network = Vec::new();
    push_message(&mut network, 1, &layer);

    model(description, network)
}

pub(crate) fn witsage_dense_transform_model() -> Vec<u8> {
    const CHANNELS: u64 = 16;
    const EXTENT: u64 = 64;
    let input = feature_description(INPUT_FEATURE, &[CHANNELS, EXTENT, EXTENT]);
    let output = feature_description(OUTPUT_FEATURE, &[CHANNELS, EXTENT, EXTENT]);

    let mut description = Vec::new();
    push_message(&mut description, 1, &input);
    push_message(&mut description, 10, &output);

    let weights = vec![1.0 / CHANNELS as f32; (CHANNELS * CHANNELS) as usize];
    let bias = vec![0.0; CHANNELS as usize];
    let mut projection = Vec::new();
    push_varint_field(&mut projection, 1, CHANNELS);
    push_varint_field(&mut projection, 2, CHANNELS);
    push_varint_field(&mut projection, 10, 1);
    push_packed_varints(&mut projection, 20, &[1, 1]);
    push_packed_varints(&mut projection, 30, &[1, 1]);
    push_message(&mut projection, 51, &[]);
    push_varint_field(&mut projection, 70, 1);
    push_message(&mut projection, 90, &weight_params(&weights));
    push_message(&mut projection, 91, &weight_params(&bias));

    let mut layer = Vec::new();
    push_string(&mut layer, 1, "witsage.feature-grid.projection");
    push_string(&mut layer, 2, INPUT_FEATURE);
    push_string(&mut layer, 3, OUTPUT_FEATURE);
    push_message(&mut layer, 100, &projection);

    let mut network = Vec::new();
    push_message(&mut network, 1, &layer);
    model(description, network)
}

fn model(description: Vec<u8>, network: Vec<u8>) -> Vec<u8> {
    let mut model = Vec::new();
    push_varint_field(&mut model, 1, 1);
    push_message(&mut model, 2, &description);
    push_message(&mut model, 500, &network);
    model
}

fn feature_description(name: &str, shape: &[u64]) -> Vec<u8> {
    let mut array = Vec::new();
    push_packed_varints(&mut array, 1, shape);
    push_varint_field(&mut array, 2, 65_568);

    let mut feature_type = Vec::new();
    push_message(&mut feature_type, 5, &array);

    let mut feature = Vec::new();
    push_string(&mut feature, 1, name);
    push_message(&mut feature, 3, &feature_type);
    feature
}

fn weight_params(values: &[f32]) -> Vec<u8> {
    let mut weight = Vec::new();
    push_key(&mut weight, 1, 2);
    push_varint(&mut weight, (values.len() * 4) as u64);
    for value in values {
        weight.extend_from_slice(&value.to_le_bytes());
    }
    weight
}

fn push_packed_varints(out: &mut Vec<u8>, field: u64, values: &[u64]) {
    let mut packed = Vec::new();
    for value in values {
        push_varint(&mut packed, *value);
    }
    push_message(out, field, &packed);
}

fn push_string(out: &mut Vec<u8>, field: u64, value: &str) {
    push_message(out, field, value.as_bytes());
}

fn push_message(out: &mut Vec<u8>, field: u64, value: &[u8]) {
    push_key(out, field, 2);
    push_varint(out, value.len() as u64);
    out.extend_from_slice(value);
}

fn push_varint_field(out: &mut Vec<u8>, field: u64, value: u64) {
    push_key(out, field, 0);
    push_varint(out, value);
}

fn push_key(out: &mut Vec<u8>, field: u64, wire_type: u64) {
    push_varint(out, (field << 3) | wire_type);
}

fn push_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[cfg(test)]
mod tests {
    use super::{witsage_dense_transform_model, witsage_vector_affine_model};

    #[test]
    fn emits_a_stable_nonempty_coreml_specification() {
        let first = witsage_vector_affine_model();
        assert_eq!(first, witsage_vector_affine_model());
        assert!(first.len() > 100);
        assert_eq!(first.first(), Some(&8));
    }

    #[test]
    fn emits_a_stable_compute_dense_coreml_specification() {
        let first = witsage_dense_transform_model();
        assert_eq!(first, witsage_dense_transform_model());
        assert!(first.len() > 1_000);
        assert_eq!(first.first(), Some(&8));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn executes_dense_model_through_coreml_compute_plan() {
        use std::{fs, process::Command, time::SystemTime};

        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuis-coreml-dense-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("runner.m");
        let binary = root.join("runner");
        let model = root.join("dense.mlmodel");
        let input = root.join("input.bin");
        fs::write(
            &source,
            include_str!("../../nsdb/provider-runners/coreml_vector_affine.m"),
        )
        .unwrap();
        fs::write(&model, witsage_dense_transform_model()).unwrap();
        fs::write(
            &input,
            vec![1.0f32; 16 * 64 * 64]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let compile = Command::new("clang")
            .args([
                "-fobjc-arc",
                "-fblocks",
                "-framework",
                "Foundation",
                "-framework",
                "CoreML",
            ])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let execution = Command::new(&binary)
            .args([&model, &input])
            .args(["input.features", "output.features", "16x64x64"])
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&execution.stdout);
        assert!(
            execution.status.success(),
            "{}",
            String::from_utf8_lossy(&execution.stderr)
        );
        assert!(stdout.contains("compute_plan_status=ready"), "{stdout}");
        assert!(
            stdout.contains("compute_plan_preferred_devices=neural-engine"),
            "{stdout}"
        );
        assert!(
            stdout.contains("compute_plan_supported_devices=") && stdout.contains("neural-engine"),
            "{stdout}"
        );
        assert!(stdout.contains("output_bytes=262144"), "{stdout}");

        let affine_model = root.join("affine.mlmodel");
        let affine_input = root.join("affine-input.bin");
        fs::write(&affine_model, witsage_vector_affine_model()).unwrap();
        fs::write(
            &affine_input,
            [1.0f32, 2.0, 3.0, 4.0]
                .into_iter()
                .flat_map(f32::to_le_bytes)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let affine = Command::new(&binary)
            .args([&affine_model, &affine_input])
            .args(["input.features", "output.features", "1x1x4"])
            .output()
            .unwrap();
        let affine_stdout = String::from_utf8_lossy(&affine.stdout);
        assert!(
            affine.status.success(),
            "{}",
            String::from_utf8_lossy(&affine.stderr)
        );
        assert!(
            affine_stdout.contains("compute_plan_preferred_devices=cpu"),
            "{affine_stdout}"
        );
        assert!(
            affine_stdout.contains("output_hex=000040400000a0400000e04000001041"),
            "{affine_stdout}"
        );
        let _ = fs::remove_dir_all(root);
    }
}
