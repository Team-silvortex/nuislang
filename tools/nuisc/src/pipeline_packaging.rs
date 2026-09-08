use super::ResolvedCompileInput;

impl ResolvedCompileInput {
    pub(crate) fn requested_packaging_mode<'a>(
        &'a self,
        override_mode: Option<&'a str>,
    ) -> Result<Option<&'a str>, String> {
        override_mode
            .or_else(|| {
                self.project
                    .as_ref()
                    .and_then(|project| project.manifest.packaging_mode.as_deref())
            })
            .map(validate_packaging_mode)
            .transpose()
    }
}

fn validate_packaging_mode(packaging_mode: &str) -> Result<&str, String> {
    match packaging_mode {
        "native-cpu-llvm" | "window-aot-bundle" | "headless-aot-bundle" | "nuis-self-contained-image" => Ok(packaging_mode),
        other => Err(format!(
            "unsupported packaging mode `{other}`; expected `native-cpu-llvm`, `window-aot-bundle`, `headless-aot-bundle`, or `nuis-self-contained-image`"
        )),
    }
}
