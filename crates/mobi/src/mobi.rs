use std::io::{Read, Write};
use crate::exth_header::EXTHHeader;
pub use crate::mobi_header::MOBIHeader;
pub use crate::palmdoc_header::PalmDOCHeader;
use anyhow::{anyhow, Result};
use byyte::be::ByteWriter;
use palm_database::{PDBHeader, PDB};
use rand::random;
use palm_database::timestamp::to_palm_timestamp;
use crate::mobi_header::NULL_INDEX;

#[derive(Debug, Clone)]
pub struct MOBI {
    pub palmdoc_header: PalmDOCHeader,
    pub header: MOBIHeader,
    pub pdb: PDB,
    pub content: String,

    pub multibyte: bool,
    pub trailers: u8,
}

impl MOBI {
    pub fn new(name: &str) -> Self {
        let slf = Self {
            palmdoc_header: PalmDOCHeader {
                compression: 2,
                text_length: 0,
                record_count: 0,
                record_size: 4096,
                encryption_type: 0,
            },
            header: MOBIHeader {
                identifier: "MOBI".to_string(),
                header_length: 232,
                mobi_type: 2,
                text_encoding: 65001,
                unique_id: random(),
                file_version: 6,
                orthographic_index: NULL_INDEX,
                inflection_index: NULL_INDEX,
                index_names: NULL_INDEX,
                index_keys: NULL_INDEX,
                extra_index0: NULL_INDEX,
                extra_index1: NULL_INDEX,
                extra_index2: NULL_INDEX,
                extra_index3: NULL_INDEX,
                extra_index4: NULL_INDEX,
                extra_index5: NULL_INDEX,
                first_non_book_index: 0,
                full_name_offset: 0,
                full_name_length: 0,
                locale: 9,
                input_language: 0,
                output_language: 0,
                min_version: 6,
                first_image_index: 0,
                huffman_record_offset: 0,
                huffman_record_count: 0,
                huffman_table_offset: 0,
                huffman_table_length: 0,
                exth_flags: 0,
                extra_record_data_flags: 0,
                first_content_record_number: 1,
                last_content_record_number: 0,
                fcis_record_number: 0,
                flis_record_number: 0,
            },
            pdb: PDB::new(PDBHeader{
                name: name.to_string(),
                attributes: 0,
                version: 0,
                creation_time: chrono::Local::now().naive_local(),
                modification_time: chrono::Local::now().naive_local(),
                last_backup_date: chrono::Local::now().naive_local(),
                modification_number: 0,
                app_info_id: 0,
                sort_info_id: 0,
                type_: "BOOK".to_string(),
                creator: "MOBI".to_string(),
                unique_id_seed: 0,
                next_record_list_id: 0,
                number_of_records: 0,
            }),
            content: "".to_string(),
            multibyte: false,
            trailers: 0,
        };

        slf
    }
    pub fn set_content(&mut self, content: &str) {
        let record_count = (content.len() / self.palmdoc_header.record_size as usize) + if (content.len() % self.palmdoc_header.record_size as usize) > 0 { 1 } else { 0 };
        self.palmdoc_header.text_length = content.len() as u32;
        self.header.last_content_record_number += record_count as u16;
        self.palmdoc_header.record_count = record_count as u16;
        self.content = content.to_string();

        self.header.fcis_record_number = self.header.last_content_record_number as u32 + 1;
        self.header.flis_record_number = self.header.last_content_record_number as u32 + 2;
    }

    fn serialize_content(&mut self) {
        let content = self.content.clone();
        // self.palmdoc_header.text_length = content.len() as u32;
        // let record_count = (content.len() / self.palmdoc_header.record_size as usize) + if (content.len() % self.palmdoc_header.record_size as usize) > 0 { 1 } else { 0 };
        // self.header.last_content_record_number += record_count as u16;
        // self.palmdoc_header.record_count = record_count as u16;

        let mut remaining_bytes = content.as_bytes();

        for _ in 0..self.palmdoc_header.record_count {
            let bytes = &remaining_bytes.take(self.palmdoc_header.record_size as u64).into_inner();
            let mut data = palmdoc_compression::compress(bytes);

            self.pdb.add_record(data);
            if bytes.len() == self.palmdoc_header.record_size as usize {
                remaining_bytes = &remaining_bytes[self.palmdoc_header.record_size as usize..];
            }
        }

    }
    pub fn add_flis(&mut self) -> anyhow::Result<()> {
        let mut data = vec![];
        data.write("FLIS".as_bytes())?;
        data.write_u32(8)?;
        data.write_u16(65)?;
        data.write_u16(0)?;
        data.write_u32(0)?;
        data.write_u32(0xFFFFFFFF)?;
        data.write_u16(1)?;
        data.write_u16(3)?;
        data.write_u32(3)?;
        data.write_u32(1)?;
        data.write_u32(0xFFFFFFFF)?;

        let id = self.pdb.add_record(data);
        // self.header.flis_record_number = id as u32;

        Ok(())
    }
    pub fn add_fcis(&mut self) -> anyhow::Result<()> {
        let mut data = vec![];
        data.write("FCIS".as_bytes())?;
        data.write_u32(20)?;
        data.write_u32(16)?;
        data.write_u32(1)?;
        data.write_u32(0)?;
        data.write_u32(self.palmdoc_header.text_length)?;
        data.write_u32(0)?;
        data.write_u32(32)?;
        data.write_u32(4)?;
        data.write_u16(1)?;
        data.write_u16(1)?;
        data.write_u32(0)?;

        let id = self.pdb.add_record(data);
        // self.header.fcis_record_number = id as u32;

        Ok(())
    }
    pub fn add_eof(&mut self) -> anyhow::Result<()> {
        let mut data = vec![];
        data.write_u8(233)?;
        data.write_u8(142)?;
        data.write_u8(13)?;
        data.write_u8(10)?;
        self.pdb.add_record(data);

        Ok(())
    }
    pub fn from_bytes<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self> {
        let pdb = PDB::from_bytes(reader)?;

        let first_record = pdb
            .read_record(0)
            .ok_or(anyhow!("Failed to read mobi header"))?;
        let mut first_record_cursor = std::io::Cursor::new(first_record);
        let palmdoc_header = PalmDOCHeader::from_bytes(&mut first_record_cursor)?;
        let header = MOBIHeader::from_bytes(&mut first_record_cursor)?;
        // let exth = EXTHHeader::from_bytes(&mut first_record_cursor)?;

        let mut multibyte = false;
        let mut trailers = 0;

        if header.header_length >= 0xE4 {
            let mut flags = header.extra_record_data_flags & 0xFFFF;
            multibyte = flags & 1 == 1;

            while flags > 1 {
                trailers += 1;
                flags = flags & (flags - 2);
            }
        }

        Ok(MOBI {
            palmdoc_header,
            header,
            content: String::new(),
            pdb,
            multibyte,
            trailers,
        })
    }

    pub fn read_record(&self, index: u16) -> Result<Vec<u8>> {
        let mut bytes = self
            .pdb
            .read_record(index)
            .ok_or(anyhow!("Failed to read text record"))?;

        for _ in 0..self.trailers {
            if bytes.len() < 4 {
                continue;
            }
            let mut num = 0;

            let end_bytes = &bytes[bytes.len() - 4..];
            for v in 0..4 {
                if end_bytes[v] & 0x80 != 0 {
                    num = 0;
                }
                num = (num << 7) | (end_bytes[v] & 0x7F);
            }
            bytes = bytes[..bytes.len() - num as usize].to_vec();
        }
        if self.multibyte {
            let num = (bytes[bytes.len() - 1] & 3) + 1;
            bytes = bytes[..bytes.len() - num as usize].to_vec();
        }

        Ok(bytes)
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut output = self.clone();
        let mut writer = Vec::new();
        writer.extend_from_slice(&self.palmdoc_header.to_bytes()?);
        writer.extend_from_slice(&self.header.to_bytes()?);
        output.pdb.add_record(writer);
        output.serialize_content();

        output.header.first_non_book_index = self.pdb.records.len() as u32;

        output.add_flis()?;
        output.add_fcis()?;
        output.add_eof()?;

        Ok(output.pdb.to_bytes()?)
    }
}
