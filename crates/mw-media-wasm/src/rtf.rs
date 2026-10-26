//! Bounded MS-OXRTFCP LZFu/MELA decoding and a conservative plain-text RTF view.
//! No HTML is emitted, and field instructions, pictures, objects and hidden
//! destinations are never executed or exposed as document text.
const MAX: usize = 1024 * 1024;
const SEED: &[u8] = b"{\\rtf1\\ansi\\mac\\deff0\\deftab720{\\fonttbl;}{\\f0\\fnil \\froman \\fswiss \\fmodern \\fscript \\fdecor MS Sans SerifSymbolArialTimes New RomanCourier{\\colortbl\\red0\\green0\\blue0\r\n\\par \\pard\\plain\\f0\\fs20\\b\\i\\u\\tab\\tx";
fn crc(bytes: &[u8]) -> u32 {
    let mut crc = 0u32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    crc
}
fn word(bytes: &[u8], offset: usize) -> Result<u32, &'static str> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("truncated compressed RTF header")?
            .try_into()
            .unwrap(),
    ))
}
pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    if bytes.len() < 16 || bytes.len() > 4 * MAX {
        return Err("compressed RTF size is invalid");
    }
    let size = word(bytes, 0)? as usize;
    let raw = word(bytes, 4)? as usize;
    let kind = word(bytes, 8)?;
    let checksum = word(bytes, 12)?;
    if size != bytes.len() - 4 || raw > MAX {
        return Err("RTF declared size exceeds limit or does not match input");
    }
    let input = &bytes[16..];
    if kind == 0x414c454d {
        if checksum != 0 || input.len() != raw {
            return Err("invalid uncompressed RTF frame");
        }
        return Ok(input.to_vec());
    }
    if kind != 0x75465a4c || crc(input) != checksum {
        return Err("unsupported RTF compression or invalid checksum");
    }
    let mut dictionary = [0u8; 4096];
    dictionary[..SEED.len()].copy_from_slice(SEED);
    let mut valid = [false; 4096];
    valid[..SEED.len()].fill(true);
    let mut write = SEED.len();
    let mut pos = 0;
    let mut output = Vec::with_capacity(raw);
    while pos < input.len() {
        let control = input[pos];
        pos += 1;
        for bit in 0..8 {
            if control & (1 << bit) == 0 {
                let byte = *input.get(pos).ok_or("truncated RTF literal")?;
                pos += 1;
                if output.len() >= raw {
                    return Err("RTF expanded size mismatch");
                }
                output.push(byte);
                dictionary[write] = byte;
                valid[write] = true;
                write = (write + 1) % 4096;
            } else {
                let pair = input.get(pos..pos + 2).ok_or("truncated RTF reference")?;
                pos += 2;
                let reference = u16::from_be_bytes([pair[0], pair[1]]);
                let mut read = (reference >> 4) as usize;
                if read == write {
                    if output.len() != raw {
                        return Err("RTF expanded size mismatch");
                    }
                    return Ok(output);
                }
                for _ in 0..(reference & 15) + 2 {
                    if !valid[read] || output.len() >= raw {
                        return Err("invalid RTF dictionary reference or expanded size");
                    }
                    let byte = dictionary[read];
                    read = (read + 1) % 4096;
                    output.push(byte);
                    dictionary[write] = byte;
                    valid[write] = true;
                    write = (write + 1) % 4096;
                }
            }
        }
    }
    Err("compressed RTF has no end marker")
}

#[derive(Clone, Copy)]
struct State {
    skip: bool,
    hidden: bool,
    uc: usize,
    codepage: u32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            skip: false,
            hidden: false,
            uc: 1,
            codepage: 1252,
        }
    }
}
struct Text {
    units: Vec<u16>,
    pending: Vec<u8>,
    fallback: usize,
}
impl Text {
    fn flush(&mut self, state: State) -> Result<(), &'static str> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let label = match state.codepage {
            65001 => "utf-8".into(),
            1250..=1258 => format!("windows-{}", state.codepage),
            932 => "shift_jis".into(),
            936 => "gbk".into(),
            949 => "euc-kr".into(),
            950 => "big5".into(),
            _ => return Err("unsupported RTF text codepage"),
        };
        let encoding = encoding_rs::Encoding::for_label(label.as_bytes())
            .ok_or("unsupported RTF text codepage")?;
        self.units
            .extend(encoding.decode(&self.pending).0.encode_utf16());
        self.pending.clear();
        self.bound()
    }
    fn bound(&self) -> Result<(), &'static str> {
        if self.units.len() + self.pending.len() > MAX {
            Err("RTF text exceeds limit")
        } else {
            Ok(())
        }
    }
    fn byte(&mut self, byte: u8, state: State) -> Result<(), &'static str> {
        if self.fallback > 0 {
            self.fallback -= 1;
        } else if !state.skip && !state.hidden {
            self.pending.push(byte);
        }
        self.bound()
    }
    fn character(&mut self, character: char, state: State) -> Result<(), &'static str> {
        self.flush(state)?;
        if self.fallback > 0 {
            self.fallback -= 1;
        } else if !state.skip && !state.hidden {
            self.units
                .extend(character.encode_utf16(&mut [0; 2]).iter().copied());
        }
        self.bound()
    }
}
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
/// A plain-text projection, not a rich RTF renderer. Group state is bounded and
/// Unicode escapes are combined as UTF-16, including surrogate pairs.
pub fn plain_text(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.len() > MAX || !bytes.starts_with(b"{\\rtf1") {
        return Err("invalid or oversized RTF document");
    }
    let mut stack = Vec::new();
    let mut state = State::default();
    let mut text = Text {
        units: Vec::new(),
        pending: Vec::new(),
        fallback: 0,
    };
    let mut pos = 0;
    let mut closed = false;
    while pos < bytes.len() {
        let byte = bytes[pos];
        pos += 1;
        if closed {
            if byte.is_ascii_whitespace() || byte == 0 {
                continue;
            }
            return Err("trailing RTF document data");
        }
        match byte {
            b'{' => {
                text.flush(state)?;
                text.fallback = 0;
                if stack.len() >= 128 {
                    return Err("RTF group nesting exceeds limit");
                }
                stack.push(state);
            }
            b'}' => {
                text.flush(state)?;
                text.fallback = 0;
                state = stack.pop().ok_or("unbalanced RTF closing group")?;
                closed = stack.is_empty();
            }
            b'\r' | b'\n' => {}
            b'\\' => {
                let next = *bytes.get(pos).ok_or("truncated RTF escape")?;
                pos += 1;
                if next.is_ascii_alphabetic() {
                    text.flush(state)?;
                    let start = pos - 1;
                    while bytes.get(pos).is_some_and(u8::is_ascii_alphabetic) {
                        pos += 1;
                    }
                    if pos - start > 64 {
                        return Err("RTF control word exceeds limit");
                    }
                    let control = &bytes[start..pos];
                    let negative = bytes.get(pos) == Some(&b'-');
                    if negative {
                        pos += 1;
                    }
                    let digits = pos;
                    let mut number = 0i32;
                    while let Some(d @ b'0'..=b'9') = bytes.get(pos) {
                        number = number
                            .checked_mul(10)
                            .and_then(|n| n.checked_add((d - b'0') as i32))
                            .ok_or("RTF control number overflow")?;
                        pos += 1;
                    }
                    let number = if pos > digits {
                        Some(if negative { -number } else { number })
                    } else {
                        if negative {
                            return Err("invalid RTF signed control");
                        }
                        None
                    };
                    if bytes.get(pos) == Some(&b' ') {
                        pos += 1;
                    }
                    match control {
                        b"rtf" if number != Some(1) => return Err("unsupported RTF version"),
                        b"fonttbl" | b"colortbl" | b"stylesheet" | b"info" | b"pict"
                        | b"object" | b"objdata" | b"fldinst" | b"filetbl" | b"listtable"
                        | b"listoverridetable" | b"revtbl" | b"rsidtbl" | b"generator"
                        | b"datastore" | b"xmlnstbl" | b"shp" | b"shpinst" | b"shppict"
                        | b"nonshppict" | b"header" | b"footer" => state.skip = true,
                        b"v" => state.hidden = number != Some(0),
                        b"plain" => state.hidden = false,
                        b"uc" => {
                            let n = number.ok_or("RTF uc requires a count")?;
                            if !(0..=16).contains(&n) {
                                return Err("RTF fallback count exceeds limit");
                            }
                            state.uc = n as usize;
                        }
                        b"u" => {
                            let n = number.ok_or("RTF unicode escape requires a number")?;
                            if !(-32768..=65535).contains(&n) {
                                return Err("invalid RTF unicode unit");
                            }
                            if !state.skip && !state.hidden {
                                text.units.push(n as u16);
                            }
                            text.fallback = state.uc;
                            text.bound()?;
                        }
                        b"ansicpg" => {
                            let n = number.ok_or("RTF codepage requires a number")?;
                            state.codepage =
                                u32::try_from(n).map_err(|_| "invalid RTF codepage")?;
                        }
                        b"bin" => {
                            let n = usize::try_from(number.ok_or("RTF binary length missing")?)
                                .map_err(|_| "invalid RTF binary length")?;
                            pos = pos
                                .checked_add(n)
                                .filter(|end| *end <= bytes.len())
                                .ok_or("truncated RTF binary data")?;
                        }
                        b"par" | b"line" | b"row" | b"page" => text.character('\n', state)?,
                        b"tab" | b"cell" => text.character('\t', state)?,
                        b"emdash" => text.character('\u{2014}', state)?,
                        b"endash" => text.character('\u{2013}', state)?,
                        b"lquote" => text.character('\u{2018}', state)?,
                        b"rquote" => text.character('\u{2019}', state)?,
                        b"ldblquote" => text.character('\u{201c}', state)?,
                        b"rdblquote" => text.character('\u{201d}', state)?,
                        b"bullet" => text.character('\u{2022}', state)?,
                        _ => {}
                    }
                } else {
                    match next {
                        b'\\' | b'{' | b'}' => text.byte(next, state)?,
                        b'\'' => {
                            let digits =
                                bytes.get(pos..pos + 2).ok_or("truncated RTF hex escape")?;
                            pos += 2;
                            let byte = (hex(digits[0]).ok_or("invalid RTF hex escape")? << 4)
                                | hex(digits[1]).ok_or("invalid RTF hex escape")?;
                            text.byte(byte, state)?;
                        }
                        b'*' => {
                            text.flush(state)?;
                            state.skip = true;
                        }
                        b'~' => text.character('\u{a0}', state)?,
                        b'_' => text.character('\u{2011}', state)?,
                        b'-' => text.character('\u{ad}', state)?,
                        b'\r' | b'\n' => text.character('\n', state)?,
                        _ => {}
                    }
                }
            }
            byte => text.byte(byte, state)?,
        }
    }
    if !closed || !stack.is_empty() {
        return Err("unclosed RTF document");
    }
    let result = String::from_utf16_lossy(&text.units);
    if result.len() > MAX {
        return Err("RTF UTF8 text exceeds limit");
    }
    Ok(result.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn microsoft_compressed_vector_and_crc_refusal() {
        // MS-OXRTFCP 3.1.1.1 independent published bytes.
        let bytes = [
            0x2d, 0, 0, 0, 0x2b, 0, 0, 0, 0x4c, 0x5a, 0x46, 0x75, 0xf1, 0xc5, 0xc7, 0xa7, 0x03, 0,
            0x0a, 0, 0x72, 0x63, 0x70, 0x67, 0x31, 0x32, 0x35, 0x42, 0x32, 0x0a, 0xf3, 0x20, 0x68,
            0x65, 0x6c, 0x09, 0, 0x20, 0x62, 0x77, 0x05, 0xb0, 0x6c, 0x64, 0x7d, 0x0a, 0x80, 0x0f,
            0xa0,
        ];
        assert_eq!(SEED.len(), 207);
        let raw = decompress(&bytes).unwrap();
        assert_eq!(plain_text(&raw).unwrap(), "hello world");
        for n in 0..bytes.len() {
            assert!(decompress(&bytes[..n]).is_err());
        }
        let mut corrupt = bytes;
        corrupt[16] ^= 1;
        assert!(decompress(&corrupt).is_err());
        let mut oversized = bytes;
        oversized[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decompress(&oversized).is_err());
    }
    #[test]
    fn uncompressed_mela_checks_exact_size_and_zero_crc() {
        let raw = br"{\rtf1 plain\par text}";
        let mut frame = ((raw.len() + 12) as u32).to_le_bytes().to_vec();
        frame.extend((raw.len() as u32).to_le_bytes());
        frame.extend(0x414c454du32.to_le_bytes());
        frame.extend(0u32.to_le_bytes());
        frame.extend(raw);
        assert_eq!(
            plain_text(&decompress(&frame).unwrap()).unwrap(),
            "plain\ntext"
        );
        frame[12] = 1;
        assert!(decompress(&frame).is_err());
    }

    #[test]
    fn unicode_escaped_braces_destinations_and_binary_are_plain_only() {
        let rtf=br"{\rtf1\ansi\ansicpg1252{\fonttbl{\f0 SecretFont;}}{\*\unknown secret}{\field{\*\fldinst HYPERLINK evil}{\fldrslt Visible}}\par Jos\'e9 \u-10179?\u-8704? \{ok\} {\pict\bin4 {}}x}\v hidden\v0 safe}";
        let text = plain_text(rtf).unwrap();
        assert_eq!(text, "Visible\nJosé 😀 {ok} safe");
        assert!(!text.contains("secret"));
    }
    #[test]
    fn malformed_groups_binary_counts_and_depth_refuse() {
        for raw in [
            b"{\\rtf1 unclosed".as_slice(),
            br"{\rtf1\bin999 nope}",
            br"{\rtf1\u999999?}",
            br"{\rtf1\uc99 nope}",
            br"{\rtf1\'gg}",
        ] {
            assert!(plain_text(raw).is_err());
        }
        let deep = format!("{{\\rtf1 {}x{} }}", "{".repeat(129), "}".repeat(129));
        assert!(plain_text(deep.as_bytes()).is_err());
    }
}
