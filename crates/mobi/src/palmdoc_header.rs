use byyte::be::{ByteReader, ByteWriter};

#[derive(Debug, Clone)]
pub struct PalmDOCHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16,
}

impl PalmDOCHeader {
    pub fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let compression = reader.read_u16()?;
        _ = reader.read_u16()?; // Reserved, usually 0
        let text_length = reader.read_u32()?;
        let record_count = reader.read_u16()?;
        let record_size = reader.read_u16()?;
        let encryption_type = reader.read_u16()?;
        _ = reader.read_u16()?; // Reserved, usually 0

        Ok(PalmDOCHeader {
            compression,
            text_length,
            record_count,
            record_size,
            encryption_type,
        })
    }

    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        data.write_u16(self.compression)?;
        data.write_u16(0)?;
        data.write_u32(self.text_length)?;
        data.write_u16(self.record_count)?;
        data.write_u16(self.record_size)?;
        data.write_u16(self.encryption_type)?;
        data.write_u16(0)?;

        Ok(data)
    }
}
