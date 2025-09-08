use std::io::Write;
use byyte::be::{ByteReader, ByteWriter};

pub const NULL_INDEX: u32 = 0xFFFFFFFF;

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
    pub extra_record_data_flags: u32,
    pub first_content_record_number: u16,
    pub last_content_record_number: u16,
    pub fcis_record_number: u32,
    pub flis_record_number: u32,
}

impl MOBIHeader {
    pub fn from_bytes<R: std::io::Read + std::io::Seek>(reader: &mut R) -> anyhow::Result<Self> {
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

        reader.seek_relative(32)?;
        reader.seek_relative(4)?;
        let drm_offset = reader.read_u32()?;
        let drm_count = reader.read_u32()?;
        let drm_size = reader.read_u32()?;
        let drm_flags = reader.read_u32()?;
        reader.seek_relative(8)?;
        let first_content_record_number = reader.read_u16()?;
        let last_content_record_number = reader.read_u16()?;
        reader.seek_relative(4)?;
        let fcis_record_number = reader.read_u32()?;
        reader.seek_relative(4)?;
        let flis_record_number = reader.read_u32()?;
        reader.seek_relative(4)?;
        reader.seek_relative(8)?;
        reader.seek_relative(4)?;
        let first_compilation_data_section_count = reader.read_u32()?;
        let number_of_first_compilation_data_sections = reader.read_u32()?;
        reader.seek_relative(4)?;
        let extra_record_data_flags = reader.read_u32()?;

        let indx_record_offset = reader.read_u32()?;

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
            first_content_record_number,
            last_content_record_number,
            fcis_record_number,
            flis_record_number,

            extra_record_data_flags,
        })
    }

    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut data = Vec::new();
        data.write("MOBI".as_bytes())?;
        data.write_u32(self.header_length)?;
        data.write_u32(self.mobi_type)?;
        data.write_u32(self.text_encoding)?;
        data.write_u32(self.unique_id)?;
        data.write_u32(self.file_version)?;
        data.write_u32(self.orthographic_index)?;
        data.write_u32(self.inflection_index)?;
        data.write_u32(self.index_names)?;
        data.write_u32(self.index_keys)?;
        data.write_u32(self.extra_index0)?;
        data.write_u32(self.extra_index1)?;
        data.write_u32(self.extra_index2)?;
        data.write_u32(self.extra_index3)?;
        data.write_u32(self.extra_index4)?;
        data.write_u32(self.extra_index5)?;
        data.write_u32(self.first_non_book_index)?;
        data.write_u32(self.full_name_offset)?;
        data.write_u32(self.full_name_length)?;
        data.write_u32(self.locale)?;
        data.write_u32(self.input_language)?;
        data.write_u32(self.output_language)?;
        data.write_u32(self.min_version)?;
        data.write_u32(self.first_image_index)?;
        data.write_u32(self.huffman_record_offset)?;
        data.write_u32(self.huffman_record_count)?;
        data.write_u32(self.huffman_table_offset)?;
        data.write_u32(self.huffman_table_length)?;
        data.write_u32(self.exth_flags)?;

        data.write(&[0u8; 32])?; // Unknown
        data.write_u32(NULL_INDEX)?; // Unknown

        data.write_u32(NULL_INDEX)?; // DRM Offset
        data.write_u32(NULL_INDEX)?; // DRM Count
        data.write_u32(0)?; // DRM Size
        data.write_u32(0)?; // DRM Flags

        data.write_u32(0)?; // Bytes to end of header? docs say to use 0
        data.write_u32(0)?;

        data.write_u16(self.first_content_record_number)?;
        data.write_u16(self.last_content_record_number)?;

        data.write_u32(1)?; // Unknown
        data.write_u32(self.fcis_record_number)?;
        data.write_u32(1)?; // Unknown
        data.write_u32(self.flis_record_number)?;
        data.write_u32(1)?;

        data.write_u32(0)?; // Unknown
        data.write_u32(0)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(0)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(NULL_INDEX)?;
        data.write_u32(self.extra_record_data_flags)?;
        data.write_u32(NULL_INDEX)?;

        Ok(data)
    }
}
