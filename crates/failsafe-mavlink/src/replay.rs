use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct ReplayRecord {
    pub host_ms: u32,
    pub frame: Vec<u8>,
}

pub fn read_rec_file(path: &Path) -> std::io::Result<Vec<ReplayRecord>> {
    let mut f = File::open(path)?;
    let mut data = Vec::new();
    f.read_to_end(&mut data)?;
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 6 <= data.len() {
        let host_ms = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
        let len = u16::from_le_bytes([data[i + 4], data[i + 5]]) as usize;
        i += 6;
        if i + len > data.len() {
            break;
        }
        let frame = data[i..i + len].to_vec();
        i += len;
        out.push(ReplayRecord { host_ms, frame });
    }
    Ok(out)
}

pub fn read_hex_fixture(path: &Path) -> std::io::Result<Vec<u8>> {
    let s = std::fs::read_to_string(path)?;
    let hex: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < hex.len() {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        out.push(byte);
        i += 2;
    }
    Ok(out)
}
