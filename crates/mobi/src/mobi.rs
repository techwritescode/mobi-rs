use std::io::Seek;
use byyte::be::ByteReader;

#[derive(Debug, Clone)]
pub struct PalmDOCHeader {
    pub compression: u16,
    pub text_length: u32,
    pub record_count: u16,
    pub record_size: u16,
    pub encryption_type: u16
}

impl PalmDOCHeader {
    pub fn new<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
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
}

#[derive(Debug, Clone)]
pub struct MOBIHeader {
    pub identifier: String,
    pub header_length: u32,
    pub mobi_type: u32,
    pub text_encoding: u32,
    pub unique_id: u32,
    pub file_version: u32,
    pub orthographic_index: u32,
    pub inflection_index: u32,
    pub index_names: u32,
    pub index_keys: u32,
    pub extra_index0: u32,
    pub extra_index1: u32,
    pub extra_index2: u32,
    pub extra_index3: u32,
    pub extra_index4: u32,
    pub extra_index5: u32,
    pub first_non_book_index: u32,
    pub full_name_offset: u32,
    pub full_name_length: u32,
    pub locale: u32,
    pub input_language: u32,
    pub output_language: u32,
    pub min_version: u32,
    pub first_image_index: u32,
    pub huffman_record_offset: u32,
    pub huffman_record_count: u32,
    pub huffman_table_offset: u32,
    pub huffman_table_length: u32,
    pub exth_flags: u32,
}

impl MOBIHeader {
    pub fn new<R: std::io::Read + std::io::Seek>(reader: &mut R) -> std::io::Result<Self> {
        let identifier = reader.read_cstr(4)?;
        let header_length = reader.read_u32()?;
        let mobi_type = reader.read_u32()?;
        let text_encoding = reader.read_u32()?;
        let unique_id = reader.read_u32()?;
        let file_version = reader.read_u32()?;
        let orthographic_index = reader.read_u32()?;
        let inflection_index = reader.read_u32()?;
        let index_names = reader.read_u32()?;
        let index_keys = reader.read_u32()?;
        let extra_index0 = reader.read_u32()?;
        let extra_index1 = reader.read_u32()?;
        let extra_index2 = reader.read_u32()?;
        let extra_index3 = reader.read_u32()?;
        let extra_index4 = reader.read_u32()?;
        let extra_index5 = reader.read_u32()?;
        let first_non_book_index = reader.read_u32()?;
        let full_name_offset = reader.read_u32()?;
        let full_name_length = reader.read_u32()?;
        let locale = reader.read_u32()?;
        let input_language = reader.read_u32()?;
        let output_language = reader.read_u32()?;
        let min_version = reader.read_u32()?;
        let first_image_index = reader.read_u32()?;
        let huffman_record_offset = reader.read_u32()?;
        let huffman_record_count = reader.read_u32()?;
        let huffman_table_offset = reader.read_u32()?;
        let huffman_table_length = reader.read_u32()?;
        let exth_flags = reader.read_u32()?;

        let offset = reader.stream_position()?;
        eprintln!("Read MOBI header. New offset: {:#}", offset);

        Ok(MOBIHeader {
            identifier,
            header_length,
            mobi_type,
            text_encoding,
            unique_id,
            file_version,
            orthographic_index,
            inflection_index,
            index_names,
            index_keys,
            extra_index0,
            extra_index1,
            extra_index2,
            extra_index3,
            extra_index4,
            extra_index5,
            first_non_book_index,
            full_name_offset,
            full_name_length,
            locale,
            input_language,
            output_language,
            min_version,
            first_image_index,
            huffman_record_offset,
            huffman_record_count,
            huffman_table_offset,
            huffman_table_length,
            exth_flags,
        })
    }
}