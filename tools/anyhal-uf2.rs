use std::{collections::BTreeMap, env, fs, io, path::Path};

const UF2_MAGIC_START0: u32 = 0x0a32_4655;
const UF2_MAGIC_START1: u32 = 0x9e5d_5157;
const UF2_MAGIC_END: u32 = 0x0ab1_6f30;
const UF2_FLAG_FAMILY_ID: u32 = 0x0000_2000;
const SAMD51_FAMILY_ID: u32 = 0x5511_4460;
const PAYLOAD_SIZE: usize = 256;
const APPLICATION_START: u32 = 0x0000_4000;
const FLASH_END: u32 = 0x0008_0000;

#[derive(Debug)]
struct Segment<'a> {
    address: u32,
    bytes: &'a [u8],
}

fn u16_at(data: &[u8], offset: usize) -> io::Result<u16> {
    let bytes = data.get(offset..offset + 2).ok_or_else(invalid_elf)?;
    Ok(u16::from_le_bytes(bytes.try_into().expect("two bytes")))
}

fn u32_at(data: &[u8], offset: usize) -> io::Result<u32> {
    let bytes = data.get(offset..offset + 4).ok_or_else(invalid_elf)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
}

fn invalid_elf() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "invalid or truncated ELF32 file",
    )
}

fn load_segments(elf: &[u8]) -> io::Result<Vec<Segment<'_>>> {
    if elf.get(0..7) != Some(b"\x7fELF\x01\x01\x01") {
        return Err(invalid_elf());
    }
    let table = u32_at(elf, 28)? as usize;
    let entry_size = u16_at(elf, 42)? as usize;
    let count = u16_at(elf, 44)? as usize;
    let mut segments = Vec::new();
    for index in 0..count {
        let header = table + index * entry_size;
        if u32_at(elf, header)? != 1 {
            continue;
        }
        let offset = u32_at(elf, header + 4)? as usize;
        let physical_address = u32_at(elf, header + 12)?;
        let file_size = u32_at(elf, header + 16)? as usize;
        // LLVM emits an ELF-header LOAD segment at address zero. Restrict UF2
        // output to the bootloader-safe application window.
        if file_size == 0 || !(APPLICATION_START..FLASH_END).contains(&physical_address) {
            continue;
        }
        let bytes = elf
            .get(offset..offset + file_size)
            .ok_or_else(invalid_elf)?;
        segments.push(Segment {
            address: physical_address,
            bytes,
        });
    }
    if segments.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "ELF has no flash segments",
        ));
    }
    Ok(segments)
}

fn push_u32(block: &mut [u8], offset: usize, value: u32) {
    block[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn convert(input: &Path, output: &Path) -> io::Result<()> {
    let elf = fs::read(input)?;
    let segments = load_segments(&elf)?;
    let mut pages = BTreeMap::<u32, [u8; PAYLOAD_SIZE]>::new();
    for segment in segments {
        for (offset, byte) in segment.bytes.iter().copied().enumerate() {
            let address = segment.address + offset as u32;
            let page_address = address & !((PAYLOAD_SIZE as u32) - 1);
            pages.entry(page_address).or_insert([0; PAYLOAD_SIZE])
                [(address - page_address) as usize] = byte;
        }
    }
    let block_count = pages.len();
    let mut uf2 = Vec::with_capacity(block_count * 512);
    for (block_number, (address, payload)) in pages.into_iter().enumerate() {
        let mut block = [0_u8; 512];
        push_u32(&mut block, 0, UF2_MAGIC_START0);
        push_u32(&mut block, 4, UF2_MAGIC_START1);
        push_u32(&mut block, 8, UF2_FLAG_FAMILY_ID);
        push_u32(&mut block, 12, address);
        push_u32(&mut block, 16, PAYLOAD_SIZE as u32);
        push_u32(&mut block, 20, block_number as u32);
        push_u32(&mut block, 24, block_count as u32);
        push_u32(&mut block, 28, SAMD51_FAMILY_ID);
        block[32..32 + PAYLOAD_SIZE].copy_from_slice(&payload);
        push_u32(&mut block, 508, UF2_MAGIC_END);
        uf2.extend_from_slice(&block);
    }
    fs::write(output, uf2)
}

fn main() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .ok_or_else(|| io::Error::other("usage: anyhal-uf2 <input.elf> <output.uf2>"))?;
    let output = args
        .next()
        .ok_or_else(|| io::Error::other("usage: anyhal-uf2 <input.elf> <output.uf2>"))?;
    if args.next().is_some() {
        return Err(io::Error::other(
            "usage: anyhal-uf2 <input.elf> <output.uf2>",
        ));
    }
    convert(Path::new(&input), Path::new(&output))
}
