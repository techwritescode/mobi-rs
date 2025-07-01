pub fn decompress_palmdoc(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let c = data[i];
        i += 1;

        match c {
            0x01..=0x08 => {
                // Literal run of c bytes
                let run_len = c as usize;
                if i + run_len > data.len() {
                    return Err("Not enough data for literal run".to_string());
                }
                output.extend_from_slice(&data[i..i+run_len]);
                i += run_len;
            }
            0x09..=0x7F => {
                // Single literal byte
                output.push(c);
            }
            0x80..=0xBF => {
                // Space + lowercase letter
                // c XOR 0x80 gives the letter ASCII
                output.push(b' ');
                output.push(c ^ 0x80);
            }
            0xC0..=0xFF => {
                // Reserved or unknown, replace with '?'
                output.push(b'?');
            }
            0x00 => {
                // Backreference (2 bytes total)
                if i >= data.len() {
                    return Err("Not enough data for backreference".to_string());
                }
                let byte2 = data[i];
                i += 1;

                // Extract length and offset
                let length = ((c >> 6) & 0x03) + 3; // length is bits 6-7 + 3
                let offset = (((c & 0x3F) as usize) << 8) | (byte2 as usize);

                if offset == 0 || offset > output.len() {
                    eprintln!("Invalid backreference offset {} at output len {}", offset, output.len());

                    return Err("Invalid backreference offset".to_string());
                }

                for _ in 0..length {
                    let val = output[output.len() - offset];
                    output.push(val);
                }
            }
            _ => {
                // Unknown byte (shouldn't happen), just push it
                output.push(c);
            }
        }
    }

    Ok(output)
}