use byyte::be::ByteReader;

#[derive(Debug, Clone)]
pub struct EXTHHeader {
    pub header_length: u32,
    pub record_count: u32,
}

impl EXTHHeader {
    pub fn from_bytes<R: std::io::Read + std::io::Seek>(reader: &mut R) -> anyhow::Result<Self> {
        let identifier = reader.read_cstr(4)?;
        assert_eq!(identifier, "EXTH");

        let header_length = reader.read_u32()?;
        let record_count = reader.read_u32()?;

        for _ in 0..record_count {
            let type_ = reader.read_u32()?;
            let len = reader.read_u32()?;
            let mut data = vec![0u8; len as usize - 8];
            reader.read_exact(&mut data)?;

            eprintln!("{type_}: {:?}", String::from_utf8(data));
        }

        Ok(EXTHHeader {
            header_length,
            record_count,
        })
    }
}
