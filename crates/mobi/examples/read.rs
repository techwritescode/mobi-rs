use std::fs::File;
use std::io::{Read, Seek, Write};
use anyhow::anyhow;
use mobi::mobi::{MOBIHeader, PalmDOCHeader};
use palm_database::PDB;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read("Quick Start Guide - John Schember.mobi")?;
    let len = data.len() as u32;
    let mut cursor = std::io::Cursor::new(data);

    let header = PDB::from_bytes(&mut cursor)?;

    let mobi_header = header.read_record(0).ok_or(anyhow!("Failed to read mobi"))?;

    let mut mobi_header_cursor = std::io::Cursor::new(mobi_header);
    let palmdoc_header = PalmDOCHeader::new(&mut mobi_header_cursor)?;
    let mobi_header = MOBIHeader::new(&mut mobi_header_cursor)?;

    println!("{:#?}", mobi_header);


    let mut str = Vec::new();
    for (i, record) in header.records.iter().enumerate().skip(1).take(mobi_header.first_non_book_index as usize - 1) {
        println!("Record #{}: {:?}", i, record);
        let record_data = header.read_record(i as u16).ok_or(anyhow!("Failed to read record"))?;
        let record_data = &record_data[..record_data.len()-2];

        File::create(format!("dump/record_{i}.bin"))?.write_all(&record_data)?;
        let new_text = palmdoc_compression::decompress(&record_data)?;
        let processed = new_text.iter().position(|&b| b == 0).map(|i| &new_text[..i]).unwrap_or(&new_text[..]);
        str.extend_from_slice(processed);
    }

    File::create(format!("dump/record.html"))?.write(String::from_utf8_lossy(&str).as_bytes())?;

    Ok(())
}
