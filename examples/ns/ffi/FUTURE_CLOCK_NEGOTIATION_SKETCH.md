# Future Clock Negotiation Sketch

This note is intentionally **forward-looking**.

It does **not** describe current repository guarantees.

Instead, it marks one small region in
[hello_clock_test_facades.ns](hello_clock_test_facades.ns)
as a plausible first probe for future `YIR`-level clock negotiation work.

## Current Shape

Today the example already collects a compact bridge-shaped summary:

* `declared_global_code`
* `resolved_global_code`
* `wall_ns`
* `monotonic_ns`
* `global_domain_id`
* `global_epoch_ns`
* `global_tick`
* `global_scale_ppm`

It also pairs that summary with a task-facing timeout test:

* `clock_domain="global"`
* `clock_policy="bridge"`

So the file already sits right at the boundary between:

* source-visible clock intent
* host/runtime clock reads
* front-door timeout interpretation

## Why This Is A Good First Negotiation Probe

Compared with a larger future timing protocol, this sample is still small and
staged:

* it already distinguishes declared vs resolved domain
* it already distinguishes wall vs monotonic readings
* it already exposes domain/epoch/scale-like fields
* it already lines up with front-door runner output

That makes it a good first probe for the question:

* which of these fields should later become more formal negotiation metadata,
  rather than remaining only summary values?

## Likely First Negotiation Candidates

If the repository later grows explicit multi-domain timing negotiation, the
most natural first candidates in this sample are probably:

1. `declared_global_code`
   * future declared source/reference domain identity
2. `resolved_global_code`
   * future negotiated target/runtime-resolved domain identity
3. `global_domain_id`
   * future local-domain identity or domain registry anchor
4. `global_epoch_ns`
   * future epoch-conversion anchor
5. `global_scale_ppm`
   * future scale/drift/tolerance anchor

These are the fields that already feel closest to negotiation metadata rather
than raw local measurements.

## What Should Probably Stay Separate

The sample also contains:

* `wall_ns`
* `monotonic_ns`
* `global_tick`

Those are still important, but they likely serve a different role:

* concrete local observations
* local runtime timing values
* local host-read surfaces

So a future negotiation contract should probably avoid collapsing everything
into one flat “clock packet.”

The healthier direction is more like:

* some fields become negotiation metadata
* some fields remain local observations

## Why This Note Exists

This note gives future clock work a concrete starting point:

* begin with the smallest sample that already exposes declared/resolved bridge
  shape
* identify which fields are really metadata
* identify which fields are really local time observations
* then grow a formal negotiation contract from there

That fits the repository’s current style well:

* keep timing assumptions visible
* avoid pretending the global clock protocol is already solved
* grow reliability before aggressive timing-sensitive optimization
