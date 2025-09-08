pub mod builder;
pub mod timestamp;

use crate::timestamp::to_palm_timestamp;
use byyte::be::{ByteReader, ByteWriter};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};

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
    pub fn from_bytes<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let name = reader.read_cstr(32)?;

        let attributes = reader.read_u16()?;
        let version = reader.read_u16()?;
        let creation_time = parse_palm_timestamp(reader.read_u32()?)?;
        let modification_time = parse_palm_timestamp(reader.read_u32()?)?;
        let last_backup_date = parse_palm_timestamp(reader.read_u32()?)?;
        let modification_number = reader.read_u32()?;
        let app_info_id = reader.read_u32()?;
        let sort_info_id = reader.read_u32()?;
        let type_ = reader.read_cstr(4)?;
        let creator = reader.read_cstr(4)?;
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

    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut name = self.name.as_bytes().to_vec();
        name.resize(32, 0);

        let mut bytes = Vec::new();
        bytes.write(&name)?;
        bytes.write_u16(self.attributes)?;
        bytes.write_u16(self.version)?;
        bytes.write_u32(to_palm_timestamp(self.creation_time)?)?;
        bytes.write_u32(to_palm_timestamp(self.modification_time)?)?;
        bytes.write_u32(to_palm_timestamp(self.last_backup_date)?)?;
        bytes.write_u32(self.modification_number)?;
        bytes.write_u32(self.app_info_id)?;
        bytes.write_u32(self.sort_info_id)?;
        bytes.write(&self.type_.as_bytes()[0..=3])?;
        bytes.write(&self.creator.as_bytes()[0..=3])?;
        bytes.write_u32(self.unique_id_seed)?;
        bytes.write_u32(self.next_record_list_id)?;
        bytes.write_u16(self.number_of_records)?;
        assert_eq!(bytes.len(), 78, "PDB header must be exactly 78 bytes long");

        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct PDBRecord {
    pub data_offset: u32,
    pub attributes: u32, // First byte is attributes, next 3 are unique ID
}

impl PDBRecord {
    pub fn from_bytes<R: Read>(reader: &mut R) -> std::io::Result<(Self, u32)> {
        let data_offset = reader.read_u32()?;
        let attributes = reader.read_u32()?;

        Ok((
            PDBRecord {
                data_offset,
                attributes,
            },
            data_offset,
        ))
    }
    pub fn to_bytes(&self, data_offset: u32) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.write_u32(data_offset)?;
        bytes.write_u32(self.attributes)?;
        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct PDB {
    pub header: PDBHeader,
    pub records: Vec<PDBRecord>,
    pub record_data: Vec<Vec<u8>>,
}

impl PDB {
    pub fn new(header: PDBHeader) -> Self {
        Self {
            header,
            records: vec![],
            record_data: vec![],
        }
    }

    pub fn read_record(&self, index: u16) -> Option<Vec<u8>> {
        self.record_data.get(index as usize).cloned()
    }

    pub fn add_record(&mut self, data: Vec<u8>) -> u16 {
        let id = self.header.unique_id_seed;
        let data_offset = self
            .records
            .last()
            .unwrap_or(&PDBRecord {
                data_offset: 0,
                attributes: 0,
            })
            .data_offset
            + self.record_data.last().unwrap_or(&vec![]).len() as u32;
        self.records.push(PDBRecord {
            data_offset,
            attributes: id,
        });
        self.record_data.push(data);
        self.header.unique_id_seed += 2;
        self.header.number_of_records += 1;


        self.header.number_of_records - 1
    }
}

impl PDB {
    pub fn from_bytes<R: Read + Seek>(reader: &mut R) -> std::io::Result<Self> {
        let header = PDBHeader::from_bytes(reader)?;
        let mut records = Vec::new();
        let mut record_data = Vec::new();

        for _ in 0..header.number_of_records {
            records.push(PDBRecord::from_bytes(reader)?);
        }

        let record_count = records.len();

        for (i, (_, data_offset)) in records.iter().cloned().enumerate() {
            let start = data_offset as u64;

            let end = if i + 1 < record_count {
                records[i + 1].1 as u64
            } else {
                let end = reader.seek(SeekFrom::End(0))?;
                if end < start {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Invalid record data offset",
                    ));
                }
                end
            };

            let len = (end - start) as usize;
            reader.seek(SeekFrom::Start(start))?;
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data)?;
            record_data.push(data);
        }

        Ok(PDB {
            header,
            records: records.iter().map(|(record, _)| record).cloned().collect(),
            record_data,
        })
    }

    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes = self.header.to_bytes()?;
        let mut offset: u32 = 78 + self.records.len() as u32 * 8; // Header and records size
        for (i, record) in self.records.iter().enumerate() {
            let data_offset = offset;
            let data = self
                .record_data
                .get(i)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing record data"))?;
            offset += data.len() as u32;
            bytes.extend_from_slice(&record.to_bytes(data_offset)?);
        }
        for data in &self.record_data {
            bytes.extend_from_slice(data);
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::PDBBuilder;
    use std::fs::File;

    #[test]
    fn test_pdb_header_to_bytes() {
        let header = PDBHeader {
            name: "TestDB".to_owned(),
            attributes: 0,
            version: 1,
            creation_time: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            modification_time: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            last_backup_date: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            modification_number: 0,
            app_info_id: 0,
            sort_info_id: 0,
            type_: "TEST".to_owned(),
            creator: "TEST".to_owned(),
            unique_id_seed: 1,
            next_record_list_id: 1,
            number_of_records: 0,
        };
        let bytes = header
            .to_bytes()
            .expect("Failed to convert header to bytes");
        assert_eq!(bytes.len(), 78, "PDB header must be exactly 78 bytes long");
    }

    #[test]
    fn test_pdb_to_bytes() {
        let pdb = PDBBuilder::new()
            .name("TestDB".to_owned())
            .attributes(0)
            .version(1)
            .type_("TEST".to_owned())
            .creator("TEST".to_owned())
            .add_record(1, 0, b"Record 1 data")
            .add_record(2, 1, b"Record 2 data")
            .build()
            .expect("Failed to build PDB");
        _ = pdb.to_bytes().expect("Failed to convert PDB to bytes");
    }
}
