use std::io;
use std::io::Result;

pub trait ByteReader: io::Read {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
    fn read_i8(&mut self) -> Result<i8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }
    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }
    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }
    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }
    fn read_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn read_cstr(&mut self, length: usize) -> Result<String> {
        let mut buf = vec![0u8; length];
        self.read_exact(&mut buf)?;
        let string = String::from_utf8(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 string"))?;
        Ok(string.trim_end_matches('\0').to_string())
    }
}

impl<R: io::Read + ?Sized> ByteReader for R {}

pub trait ByteWriter: io::Write {
    fn write_u8(&mut self, value: u8) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_u16(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_u32(&mut self, value: u32) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_i8(&mut self, value: i8) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_i16(&mut self, value: i16) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_i32(&mut self, value: i32) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_f32(&mut self, value: f32) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }
    fn write_f64(&mut self, value: f64) -> Result<()> {
        self.write_all(&value.to_be_bytes())
    }

    fn write_cstr(&mut self, string: &str) -> Result<()> {
        let bytes = string.as_bytes();
        let mut buf = bytes.to_vec();
        buf.push(0); // Null-terminate
        self.write_all(&buf)
    }
}

impl<W: io::Write + ?Sized> ByteWriter for W {}