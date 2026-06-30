#[cfg(test)]
mod tests {
    use super::super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nuisc_cache_{label}_{nonce}"))
    }

    #[test]
    fn fingerprint_records_is_stable_across_record_order_after_sorting() {
        let temp_dir = temp_path("fingerprint_order");
        fs::create_dir_all(&temp_dir).unwrap();
        let alpha = temp_dir.join("alpha.txt");
        let beta = temp_dir.join("beta.txt");
        fs::write(&alpha, "alpha-file").unwrap();
        fs::write(&beta, "beta-file").unwrap();

        let mut left = vec![
            CacheFingerprintRecord::file_path("b.file", beta.clone()),
            CacheFingerprintRecord::inline_bytes("a.inline", b"inline".to_vec()),
            CacheFingerprintRecord::file_path("c.file", alpha.clone()),
        ];
        let mut right = vec![
            CacheFingerprintRecord::file_path("c.file", alpha),
            CacheFingerprintRecord::file_path("b.file", beta),
            CacheFingerprintRecord::inline_bytes("a.inline", b"inline".to_vec()),
        ];
        left.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));
        right.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));

        let left_hash = fingerprint_records(&left).unwrap();
        let right_hash = fingerprint_records(&right).unwrap();
        assert_eq!(left_hash, right_hash);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn store_compile_cache_refreshes_existing_entry_contents() {
        let temp_dir = temp_path("store_refresh");
        let cache_root = temp_dir.join("cache");
        let output_dir = temp_dir.join("out");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(output_dir.join("marker.txt"), "first").unwrap();

        let key = CompileCacheKey {
            root: cache_root,
            key: "demo-key".to_owned(),
            input_labels: vec!["demo".to_owned()],
        };

        let entry = store_compile_cache(&key, &output_dir).unwrap();
        assert_eq!(
            fs::read_to_string(entry.entry_dir.join("marker.txt")).unwrap(),
            "first"
        );

        fs::write(output_dir.join("marker.txt"), "second").unwrap();
        let entry = store_compile_cache(&key, &output_dir).unwrap();
        assert_eq!(
            fs::read_to_string(entry.entry_dir.join("marker.txt")).unwrap(),
            "second"
        );

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
