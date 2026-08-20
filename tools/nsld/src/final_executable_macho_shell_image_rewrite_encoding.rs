use crate::{
    final_executable_macho_shell_layout::locate_source_address,
    reports::{NsldMachOArm64RelocationApplication, NsldMachOArm64ShellLayoutPlanReport},
};
use std::collections::BTreeMap;

pub(crate) fn encode_final_relocation(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    source_vm: u64,
    target_vm: u64,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<(Vec<u8>, i64), String> {
    match application.relocation_kind.as_str() {
        "arm64-unsigned" => encode_unsigned(application, source, target_vm, applications, shell),
        "arm64-branch26" => encode_branch26(application, source, source_vm, target_vm),
        "arm64-page21" | "arm64-got-load-page21" => {
            encode_page21(application, source, source_vm, target_vm, applications)
        }
        "arm64-pageoff12" | "arm64-got-load-pageoff12" => {
            encode_pageoff12(application, source, source_vm, target_vm, applications)
        }
        other => Err(format!(
            "Mach-O shell relocation `{}` has unsupported kind `{other}`",
            application.relocation_id
        )),
    }
}

pub(crate) fn encode_final_stub(
    stub_vm: u64,
    got_vm: u64,
    structure_id: &str,
) -> Result<Vec<u8>, String> {
    if !stub_vm.is_multiple_of(4) || !got_vm.is_multiple_of(8) {
        return Err(format!(
            "Mach-O shell stub `{structure_id}` has unaligned final addresses"
        ));
    }
    let page_delta = (i128::from(got_vm) & !0xfff) - (i128::from(stub_vm) & !0xfff);
    if page_delta % 4096 != 0 || !(-0x1_0000_0000..=0x0_ffff_f000).contains(&page_delta) {
        return Err(format!(
            "Mach-O shell stub `{structure_id}` final GOT page is out of range"
        ));
    }
    let page_immediate = ((page_delta >> 12) as i64 as u32) & 0x001f_ffff;
    let adrp =
        0x9000_0010u32 | (page_immediate & 0x3) << 29 | ((page_immediate >> 2) & 0x7ffff) << 5;
    let page_offset = (got_vm & 0x0fff) as usize;
    if !page_offset.is_multiple_of(8) {
        return Err(format!(
            "Mach-O shell stub `{structure_id}` GOT offset is unaligned"
        ));
    }
    let ldr = 0xf940_0210u32 | ((page_offset / 8) as u32) << 10;
    let mut bytes = Vec::with_capacity(12);
    bytes.extend_from_slice(&adrp.to_le_bytes());
    bytes.extend_from_slice(&ldr.to_le_bytes());
    bytes.extend_from_slice(&0xd61f_0200u32.to_le_bytes());
    Ok(bytes)
}

fn encode_unsigned(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    target_vm: u64,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<(Vec<u8>, i64), String> {
    let embedded = i128::from(read_signed_le(source)?);
    let subtractor_vm = paired_metadata(application, applications, "arm64-subtractor")?
        .map(|pair| {
            let target = pair.target_output_offset.ok_or_else(|| {
                format!("Mach-O subtractor `{}` has no target", pair.relocation_id)
            })?;
            locate_source_address(target, &shell.sections, &shell.segments)
                .map(|address| address.vm_address)
        })
        .transpose()?
        .unwrap_or(0);
    let effective = embedded - i128::from(subtractor_vm);
    let value = i128::from(target_vm) + effective;
    let encoded = match source.len() {
        4 if (0..=i128::from(u32::MAX)).contains(&value) => (value as u32).to_le_bytes().to_vec(),
        8 if (0..=i128::from(u64::MAX)).contains(&value) => (value as u64).to_le_bytes().to_vec(),
        width => {
            return Err(format!(
                "Mach-O unsigned relocation `{}` value {value} does not fit {width} bytes",
                application.relocation_id
            ));
        }
    };
    Ok((encoded, checked_i64(effective, "unsigned addend")?))
}

fn encode_branch26(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    source_vm: u64,
    target_vm: u64,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application, source_vm)?;
    if word & 0x7c00_0000 != 0x1400_0000 {
        return Err(format!(
            "Mach-O branch26 relocation `{}` source is not B/BL",
            application.relocation_id
        ));
    }
    let embedded = sign_extend(u64::from(word & 0x03ff_ffff), 26) << 2;
    let displacement = i128::from(target_vm) + i128::from(embedded) - i128::from(source_vm);
    if displacement % 4 != 0 || !(-0x0800_0000..=0x07ff_fffc).contains(&displacement) {
        return Err(format!(
            "Mach-O branch26 relocation `{}` final displacement {displacement} is invalid",
            application.relocation_id
        ));
    }
    let immediate = ((displacement >> 2) as i64 as u32) & 0x03ff_ffff;
    Ok((
        (word & !0x03ff_ffff | immediate).to_le_bytes().to_vec(),
        embedded,
    ))
}

fn encode_page21(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    source_vm: u64,
    target_vm: u64,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application, source_vm)?;
    if word & 0x9f00_0000 != 0x9000_0000 {
        return Err(format!(
            "Mach-O page21 relocation `{}` source is not ADRP",
            application.relocation_id
        ));
    }
    let immediate = u64::from(((word >> 29) & 0x3) | (((word >> 5) & 0x7ffff) << 2));
    let embedded = sign_extend(immediate, 21) << 12;
    let explicit = explicit_addend(application, applications)?;
    let effective = i128::from(embedded) + i128::from(explicit);
    let target = i128::from(target_vm) + effective;
    if target < 0 {
        return Err(format!(
            "Mach-O page21 relocation `{}` final target is negative",
            application.relocation_id
        ));
    }
    let page_delta = (target & !0xfff) - (i128::from(source_vm) & !0xfff);
    if page_delta % 4096 != 0 || !(-0x1_0000_0000..=0x0_ffff_f000).contains(&page_delta) {
        return Err(format!(
            "Mach-O page21 relocation `{}` final page delta is invalid",
            application.relocation_id
        ));
    }
    let encoded_immediate = ((page_delta >> 12) as i64 as u32) & 0x001f_ffff;
    let immlo = (encoded_immediate & 0x3) << 29;
    let immhi = ((encoded_immediate >> 2) & 0x7ffff) << 5;
    Ok((
        (word & !0x60ff_ffe0 | immlo | immhi).to_le_bytes().to_vec(),
        checked_i64(effective, "page21 addend")?,
    ))
}

fn encode_pageoff12(
    application: &NsldMachOArm64RelocationApplication,
    source: &[u8],
    source_vm: u64,
    target_vm: u64,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<(Vec<u8>, i64), String> {
    let word = read_instruction(source, application, source_vm)?;
    let scale = pageoff_scale(word).ok_or_else(|| {
        format!(
            "Mach-O pageoff12 relocation `{}` has unsupported instruction",
            application.relocation_id
        )
    })?;
    let embedded = i64::from((word >> 10) & 0x0fff) * scale as i64;
    let explicit = explicit_addend(application, applications)?;
    let effective = i128::from(embedded) + i128::from(explicit);
    let address = i128::from(target_vm) + effective;
    if address < 0 {
        return Err(format!(
            "Mach-O pageoff12 relocation `{}` final target is negative",
            application.relocation_id
        ));
    }
    let page_offset = (address as u128 & 0x0fff) as usize;
    if !page_offset.is_multiple_of(scale) {
        return Err(format!(
            "Mach-O pageoff12 relocation `{}` offset is not scale-aligned",
            application.relocation_id
        ));
    }
    let immediate = page_offset / scale;
    if immediate > 0x0fff {
        return Err("Mach-O pageoff12 immediate exceeds 12 bits".to_owned());
    }
    Ok((
        (word & !(0x0fff << 10) | (immediate as u32) << 10)
            .to_le_bytes()
            .to_vec(),
        checked_i64(effective, "pageoff12 addend")?,
    ))
}

fn paired_metadata<'a>(
    application: &NsldMachOArm64RelocationApplication,
    applications: &'a BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
    expected_kind: &str,
) -> Result<Option<&'a NsldMachOArm64RelocationApplication>, String> {
    let Some(pair_id) = application.pair_relocation_id.as_deref() else {
        return Ok(None);
    };
    let pair = applications.get(pair_id).copied().ok_or_else(|| {
        format!(
            "Mach-O shell relocation `{}` references missing pair `{pair_id}`",
            application.relocation_id
        )
    })?;
    if pair.relocation_kind != expected_kind
        || pair.application_status != "paired-metadata"
        || pair.pair_relocation_id.as_deref() != Some(application.relocation_id.as_str())
    {
        return Err(format!(
            "Mach-O shell relocation `{}` has an invalid `{expected_kind}` pair",
            application.relocation_id
        ));
    }
    Ok(Some(pair))
}

fn explicit_addend(
    application: &NsldMachOArm64RelocationApplication,
    applications: &BTreeMap<&str, &NsldMachOArm64RelocationApplication>,
) -> Result<i64, String> {
    let Some(pair) = paired_metadata(application, applications, "arm64-addend")? else {
        return Ok(0);
    };
    pair.explicit_addend.ok_or_else(|| {
        format!(
            "Mach-O shell addend pair `{}` has no value",
            pair.relocation_id
        )
    })
}

fn read_instruction(
    source: &[u8],
    application: &NsldMachOArm64RelocationApplication,
    source_address: u64,
) -> Result<u32, String> {
    if !source_address.is_multiple_of(4) {
        return Err(format!(
            "Mach-O shell instruction `{}` address is unaligned",
            application.relocation_id
        ));
    }
    let bytes: [u8; 4] = source.try_into().map_err(|_| {
        format!(
            "Mach-O shell instruction `{}` requires four bytes",
            application.relocation_id
        )
    })?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_signed_le(bytes: &[u8]) -> Result<i64, String> {
    match bytes {
        [a, b, c, d] => Ok(i32::from_le_bytes([*a, *b, *c, *d]) as i64),
        [a, b, c, d, e, f, g, h] => Ok(i64::from_le_bytes([*a, *b, *c, *d, *e, *f, *g, *h])),
        _ => Err(format!(
            "Mach-O shell absolute relocation width {} is unsupported",
            bytes.len()
        )),
    }
}

fn pageoff_scale(word: u32) -> Option<usize> {
    if word & 0x7f00_0000 == 0x1100_0000 && word & (1 << 22) == 0 {
        return Some(1);
    }
    if word & 0x3b00_0000 == 0x3900_0000 {
        let vector_q =
            word & (1 << 26) != 0 && (word >> 30) & 0x3 == 0 && (word >> 22) & 0x3 == 0x3;
        return Some(if vector_q {
            16
        } else {
            1usize << ((word >> 30) & 0x3)
        });
    }
    None
}

fn sign_extend(value: u64, bits: u32) -> i64 {
    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}

fn checked_i64(value: i128, label: &str) -> Result<i64, String> {
    i64::try_from(value).map_err(|_| format!("Mach-O shell {label} exceeds i64"))
}
