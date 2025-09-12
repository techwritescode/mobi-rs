use byyte::be::ByteWriter;
use palm_database::{PDB, PDBHeader};
use rand::random;
use std::io::{Read, Write};

const TEXT_RECORD_SIZE: usize = 4096;
const NULL_INDEX: u32 = 0xFFFFFFFF;

pub fn fcis(text_length: u32) -> Result<Vec<u8>, anyhow::Error> {
    let mut data = vec![];
    data.write("FCIS".as_bytes())?;
    data.write_u32(20)?;
    data.write_u32(16)?;
    data.write_u32(1)?;
    data.write_u32(0)?;
    data.write_u32(text_length)?;
    data.write_u32(0)?;
    data.write_u32(32)?;
    data.write_u32(8)?;
    data.write_u32(1)?;
    data.write_u32(1)?;
    data.write_u32(0)?;
    Ok(data)
}

pub fn flis() -> Result<Vec<u8>, anyhow::Error> {
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
    Ok(data)
}

pub fn eof() -> Vec<u8> {
    vec![233, 142, 13, 10]
}

pub struct MobiWriter {
    name: String,
    content: String,
    images: Vec<Vec<u8>>,
    text_record_count: usize,
}

impl MobiWriter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            content: "".to_owned(),
            images: vec![],
            text_record_count: 0,
        }
    }

    fn text_record_count(&self) -> usize {
        (self.content.len() / TEXT_RECORD_SIZE)
            + if (self.content.len() % TEXT_RECORD_SIZE) != 0 {
                1
            } else {
                0
            }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.text_record_count = self.text_record_count();
    }

    pub fn add_image(&mut self, image: Vec<u8>) {
        self.images.push(image);
    }

    fn generate_palmdoc(&self) -> Result<Vec<u8>, anyhow::Error> {
        let mut data = vec![];
        data.write_u16(1)?; // Palmdoc Compression
        data.write_u16(0)?;
        data.write_u32(self.content.len() as u32)?;
        data.write_u16(self.text_record_count as u16)?;
        data.write_u16(TEXT_RECORD_SIZE as u16)?;
        data.write_u16(0)?; // No Encryption
        data.write_u16(0)?; // Unknown
        Ok(data)
    }

    fn generate_mobiheader(&self) -> Result<Vec<u8>, anyhow::Error> {
        let first_non_book_index = self.text_record_count as u32 + 1;
        let last_content_index = self.text_record_count as u32 + self.images.len() as u32;

        let mut data = vec![];
        data.write("MOBI".as_bytes())?;
        data.write_u32(0xE8)?; // Header Length (might need to be updated)
        data.write_u32(0x002)?;
        data.write_u32(65001)?; // UTF-8
        data.write_u32(random())?;
        data.write_u32(6)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(NULL_INDEX)?;
        data.write(vec![0xFFu8; 24].as_slice())?;
        data.write_u32(first_non_book_index)?;
        data.write_u32(0x100)?;
        data.write_u32(self.name.len() as u32)?;
        data.write_u32(1033)?;
        data.write_u32(0)?;
        data.write_u32(0)?;
        data.write_u32(6)?;
        data.write_u32(first_non_book_index)?; // no index records, use page after text records as image
        data.write(vec![0u8; 16].as_slice())?;
        data.write_u32(0)?; // No EXTH for now
        data.write(vec![0u8; 32].as_slice())?;
        data.write_u32(NULL_INDEX)?; // Unknown

        data.write_u32(NULL_INDEX)?; // No DRM
        data.write_u32(NULL_INDEX)?;
        data.write_u32(0)?;
        data.write_u32(0)?;

        data.write(vec![0u8; 8].as_slice())?;

        data.write_u16(1)?;
        data.write_u16(last_content_index as u16 - 1)?; // TODO: this might be wrong

        data.write_u32(1)?;
        data.write_u32(last_content_index)?; // FCIS
        data.write_u32(1)?;
        data.write_u32(last_content_index + 1)?; // FLIS
        data.write_u32(1)?;

        data.write_u32(0)?;
        data.write_u32(0)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(0)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(0)?; // No extra data
        data.write_u32(NULL_INDEX)?; // No Index
        data.write(vec![0u8; 8].as_slice())?;
        // eprintln!("out {:02x}", data.len() + 0x10); // 0x100
        data.write(self.name.as_bytes())?;

        // data.write(vec![0u8; 1024].as_slice())?;

        let extra = data.len() % 4;

        data.write(vec![0u8; 4-extra].as_slice())?;

        Ok(data)
    }

    fn generate_record0(&self) -> Result<Vec<u8>, anyhow::Error> {
        let mut data = vec![];
        data.write(self.generate_palmdoc()?.as_slice())?;
        data.write(self.generate_mobiheader()?.as_slice())?;
        Ok(data)
    }

    fn generate_text_records(&self) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        let content = self.content.as_bytes();
        let mut records = vec![];

        let mut start = 0;

        for _ in 0..self.text_record_count() {
            let bytes = content[start..content.len().min(start+TEXT_RECORD_SIZE)].to_vec();
            // let data = palmdoc_compression::compress(bytes);
            records.push(bytes.to_vec());
            if bytes.len() == TEXT_RECORD_SIZE {
                start += TEXT_RECORD_SIZE;
            }
        }

        Ok(records)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        let text_records = self.generate_text_records()?;

        let pdb_header = PDBHeader {
            name: self.name[..self.name.len().min(32)].to_string(),
            attributes: 0,
            version: 0,
            creation_time: Default::default(),
            modification_time: Default::default(),
            last_backup_date: Default::default(),
            modification_number: 0,
            app_info_id: 0,
            sort_info_id: 0,
            type_: "BOOK".to_string(),
            creator: "MOBI".to_string(),
            unique_id_seed: 0,
            next_record_list_id: 0,
            number_of_records: 0,
        };

        let mut pdb = PDB::new(pdb_header);
        pdb.add_record(self.generate_record0()?);

        for text_record in text_records {
            pdb.add_record(text_record);
        }

        for image in self.images.iter() {
            pdb.add_record(image.to_vec());
        }

        pdb.add_record(flis()?);
        pdb.add_record(fcis(self.content.len() as u32)?);
        pdb.add_record(eof());

        Ok(pdb.to_bytes()?)
    }
}
