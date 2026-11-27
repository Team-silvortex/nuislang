use super::Site;
use std::{collections::BTreeSet, fmt::Write};

// A rejected call never writes a partial uniform. Output words use native
// alignment/endianness here; the runtime serializes their bits little-endian.
pub(super) fn emit(sites: &[Site]) -> String {
    let mut ir = String::from("define i32 @nuis_packet_projection_v1(i32 %site, i32 %slot, i32 %n, ptr %words, ptr %out) {\nentry:\n  switch i32 %site, label %fail [\n");
    for site in sites {
        writeln!(ir, "    i32 {0}, label %site{0}", site.ordinal).unwrap();
    }
    ir.push_str("  ]\n");
    for &Site {
        ordinal: i,
        slot,
        width,
    } in sites
    {
        writeln!(
            ir,
            "site{i}:
  %s{i} = icmp eq i32 %slot, {slot}
  %w{i} = icmp eq i32 %n, {width}
  %ok{i} = and i1 %s{i}, %w{i}
  br i1 %ok{i}, label %project{width}, label %fail"
        )
        .unwrap();
    }
    for n in sites.iter().map(|s| s.width).collect::<BTreeSet<_>>() {
        writeln!(ir, "project{n}:").unwrap();
        let mut ok = "true".to_owned();
        for i in 0..n {
            writeln!(
                ir,
                "  %p{n}_{i} = getelementptr i64, ptr %words, i32 {i}
  %v{n}_{i} = load i64, ptr %p{n}_{i}, align 8
  %lo{n}_{i} = icmp sge i64 %v{n}_{i}, -16777216
  %hi{n}_{i} = icmp sle i64 %v{n}_{i}, 16777216
  %fits{n}_{i} = and i1 %lo{n}_{i}, %hi{n}_{i}
  %all{n}_{i} = and i1 {ok}, %fits{n}_{i}"
            )
            .unwrap();
            ok = format!("%all{n}_{i}");
        }
        writeln!(ir, "  br i1 {ok}, label %store{n}, label %fail\nstore{n}:").unwrap();
        for i in 0..4 {
            let value = if i < n {
                writeln!(ir, "  %f{n}_{i} = sitofp i64 %v{n}_{i} to float").unwrap();
                format!("%f{n}_{i}")
            } else {
                "0.0".into()
            };
            writeln!(
                ir,
                "  %d{n}_{i} = getelementptr float, ptr %out, i32 {i}
  store float {value}, ptr %d{n}_{i}, align 4"
            )
            .unwrap();
        }
        ir.push_str("  ret i32 0\n");
    }
    ir.push_str("fail:\n  ret i32 1\n}\n");
    ir.push_str(r#"
define i32 @nuis_packet_submit_v2(i32 %site, i32 %slot, i32 %width, ptr %words, ptr %ctx, ptr %submit) {
entry:
  %have_words = icmp ne ptr %words, null
  %have_ctx = icmp ne ptr %ctx, null
  %have_fn = icmp ne ptr %submit, null
  %have_input = and i1 %have_words, %have_ctx
  %ready = and i1 %have_input, %have_fn
  br i1 %ready, label %project, label %reject
project:
  %binding = alloca [4 x i32], align 4
  %status = call i32 @nuis_packet_projection_v1(i32 %site, i32 %slot, i32 %width, ptr %words, ptr %binding)
  %ok = icmp eq i32 %status, 0
  br i1 %ok, label %dispatch, label %reject
dispatch:
  %result = call i32 %submit(ptr %ctx, i32 %slot, ptr %binding)
  ret i32 %result
reject:
  ret i32 1
}
"#);
    ir
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, process::Command, time::SystemTime};

    #[test]
    fn compiled_projector_is_site_bound_exact_and_transactional() {
        struct Temp(PathBuf);
        impl Drop for Temp {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let dir = Temp(std::env::temp_dir().join(format!(
                "nuis-packet-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
        fs::create_dir(&dir.0).unwrap();
        let sites: Vec<_> = (1..=4)
            .map(|width| Site {
                ordinal: width * 9,
                slot: width + 1,
                width,
            })
            .collect();
        let ir = dir.0.join("packet.ll");
        let harness = dir.0.join("check.c");
        let exe = dir.0.join("check");
        fs::write(&ir, emit(&sites)).unwrap();
        fs::write(
            &harness,
            r#"#include <stdint.h>
#include <string.h>
extern int32_t nuis_packet_projection_v1(uint32_t, uint32_t, uint32_t, const int64_t *, uint32_t *);
typedef int32_t (*submit_fn)(void *, uint32_t, const uint32_t *);
extern int32_t nuis_packet_submit_v2(uint32_t,uint32_t,uint32_t,const int64_t *,void *,submit_fn);
struct sink { uint32_t calls, slot, expected[4]; int32_t status; };
static int32_t submit(void *ctx, uint32_t slot, const uint32_t *binding) {
    struct sink *s = ctx;
    ++s->calls;
    if (s->slot != slot || memcmp(s->expected, binding, sizeof s->expected)) return 91;
    return s->status;
}
int main(void) {
    int64_t words[4] = {-16777216, 16777216, -7, 41};
    uint32_t out[4], keep[4] = {11, 22, 33, 44};
    struct sink sink = {0};
    for (unsigned n = 1; n <= 4; ++n) {
        memcpy(out, keep, sizeof out);
        if (nuis_packet_projection_v1(n*9,n+1,n,words,out)) return 1;
        for (unsigned i = 0; i < 4; ++i) {
            float expected = i < n ? (float) words[i] : 0.0f;
            if (memcmp(&out[i], &expected, sizeof expected)) return 2;
        }
        sink.slot = n+1;
        memcpy(sink.expected, out, sizeof out);
        sink.status = n == 4 ? 17 : 0;
        if (nuis_packet_submit_v2(n*9, n+1, n, words, &sink, submit) != sink.status || sink.calls != n) return 5;
    }
    memcpy(out, keep, sizeof out);
    for (unsigned i = 0; i < 5; ++i) {
        words[3] = i == 3 ? 16777217 : i == 4 ? -16777217 : 41;
        if (!nuis_packet_projection_v1(i==0 ? 0 : 36,i==1 ? 2 : 5,i==2 ? 3 : 4,words,out)) return 3;
        if (memcmp(out, keep, sizeof out)) return 4;
        if (!nuis_packet_submit_v2(i==0 ? 0 : 36,i==1 ? 2 : 5,i==2 ? 3 : 4,words,&sink,submit) || sink.calls != 4) return 6;
    }
    return 0;
}"#,
        )
        .unwrap();
        let build = Command::new("clang")
            .args(["-O2", "-o"])
            .arg(&exe)
            .arg(&ir)
            .arg(&harness)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        assert!(Command::new(exe).status().unwrap().success());
    }
}
