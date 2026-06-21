# Subprojects

This directory hosts closely related but still boundary-conscious sibling
projects that currently live inside the `nuislang` workspace tree.

Current subprojects:

* `yalivia`
  future runtime / lifecycle / JIT-adjacent execution project
* `vulpoya`
  future analyzer / verifier / YIR contract re-check project

Current policy:

* keep them physically close to `nuislang` during rapid architecture changes
* keep their protocol and ownership boundaries explicit so they can still split
  back into standalone repositories later if that becomes useful
* preserve their own nested `.git` histories for now rather than flattening
  them into the main repository immediately
* do not add them to the root Cargo workspace until their boundaries are
  intentionally defined and there is a real reason to build them together

Boundary rule of thumb:

* `tools/` and `crates/` are current main-repo implementation surfaces
* `subprojects/yalivia` and `subprojects/vulpoya` are sibling ecosystem
  projects hosted nearby, not compiler-internal folders wearing different names
