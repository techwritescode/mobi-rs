pub fn palmdoc_decompress(data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(data.len() * 2);
    let mut i = 0usize;

    while i < data.len() {
        let frame = data[i];
        i += 1;

        match frame {
            // 1..=8: copy next `frame` raw bytes
            1..=8 => {
                let n = frame as usize;
                if i + n <= data.len() {
                    out.extend_from_slice(&data[i..i + n]);
                } else {
                    // truncated input: copy what remains
                    out.extend_from_slice(&data[i..]);
                }
                i += n;
            }

            // 0..=127: literal single byte (ASCII / raw byte)
            0..=127 => {
                out.push(frame);
            }

            // 192..=255 (0xC0..0xFF): space + (frame ^ 0x80)
            192..=255 => {
                out.push(b' ');
                out.push(frame ^ 0x80);
            }

            // 128..=191: two-byte backreference
            128..=191 => {
                // need the next byte to form the 16-bit token
                if i >= data.len() {
                    break; // truncated input
                }
                let second = data[i];
                i += 1;

                let concat = ((frame as u16) << 8) | (second as u16);
                let distance = ((concat >> 3) & 0x07FF) as usize; // 11 bits
                let length = ((concat & 0x07) + 3) as usize;       // 3..10

                // invalid backreference -> abort to avoid panic
                if distance == 0 || distance > out.len() {
                    break;
                }

                // start position in output to copy from
                let mut src = out.len() - distance;
                for _ in 0..length {
                    // reading from `out[src]` is safe because `out` grows as we push,
                    // and `src` will always be < current length at read time.
                    let b = out[src];
                    out.push(b);
                    src += 1;
                }
            }
        }
    }

    out
}
