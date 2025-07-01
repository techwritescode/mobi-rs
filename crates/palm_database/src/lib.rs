use std::io::{Seek, Error, ErrorKind};
use byyte::ByteReader;

pub fn parse_palm_timestamp(timestamp: u32) -> Result<chrono::NaiveDateTime, Error> {
    let seconds = timestamp as i64;
    let epoch = chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
        .and_then(|t| t.and_hms_opt(0, 0, 0))
        .ok_or(Error::from(ErrorKind::InvalidData))?;
    Ok(epoch + chrono::Duration::seconds(seconds))
}


#[derive(Debug, Clone)]
pub struct PDBHeader {
    pub name: String,
    pub attributes: u16,
    pub version: u16,
    pub creation_time: chrono::NaiveDateTime,
    pub modification_time: chrono::NaiveDateTime,
    pub last_backup_date: chrono::NaiveDateTime,
    pub modification_number: u32,
    pub app_info_id: u32,
    pub sort_info_id: u32,
    pub type_: String,
    pub creator: String,
    pub unique_id_seed: u32,
    pub next_record_list_id: u32,
    pub number_of_records: u16,
}

impl PDBHeader {
    pub fn new<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let name = reader.read_string(32)?;

        let attributes = reader.read_u16()?;
        let version = reader.read_u16()?;
        let creation_time = parse_palm_timestamp(reader.read_u32()?)?;
        let modification_time = parse_palm_timestamp(reader.read_u32()?)?;
        let last_backup_date = parse_palm_timestamp(reader.read_u32()?)?;
        let modification_number = reader.read_u32()?;
        let app_info_id = reader.read_u32()?;
        let sort_info_id = reader.read_u32()?;
        let type_ = reader.read_string(4)?;
        let creator = reader.read_string(4)?;
        let unique_id_seed = reader.read_u32()?;
        let next_record_list_id = reader.read_u32()?;
        let number_of_records = reader.read_u16()?;

        Ok(PDBHeader {
            name,
            attributes,
            version,
            creation_time,
            modification_time,
            last_backup_date,
            modification_number,
            app_info_id,
            sort_info_id,
            type_,
            creator,
            unique_id_seed,
            next_record_list_id,
            number_of_records,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PDBRecord {
    pub data_offset: u32,
    pub attributes: u8,
    pub unique_id: [u8; 3],
}

impl PDBRecord {
    pub fn new<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let data_offset = reader.read_u32()?;
        let attributes = reader.read_u8()?;
        let mut unique_id = [0u8; 3];
        reader.read_exact(&mut unique_id)?;

        Ok(PDBRecord {
            data_offset,
            attributes,
            unique_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PDB {
    pub header: PDBHeader,
    pub records: Vec<PDBRecord>,
}

impl PDB {
    pub fn read_record<R: std::io::Read + std::io::Seek>(&self, reader: &mut R, index: u16) -> std::io::Result<Vec<u8>> {
        if index >= self.records.len() as u16 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Index out of bounds"));
        }

        let record = &self.records[index as usize];
        let start = record.data_offset as u64;

        let end = if index + 1 < self.records.len() as u16 {
            self.records[(index + 1) as usize].data_offset as u64
        } else {
            let end = reader.seek(std::io::SeekFrom::End(0))?;
            if end < start {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid record data offset"));
            }
            end
        };

        let len = (end - start) as usize;
        reader.seek(std::io::SeekFrom::Start(start))?;
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data)?;

        Ok(data)
    }
}

impl PDB {
    pub fn new<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let header = PDBHeader::new(reader)?;
        let mut records = Vec::new();

        for _ in 0..header.number_of_records {
            records.push(PDBRecord::new(reader)?);
        }

        Ok(PDB { header, records })
    }
}