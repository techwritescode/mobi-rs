use anyhow::anyhow;
use mobi::mobi::{MOBI, MOBIHeader, PalmDOCHeader};
use palm_database::PDB;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use mobi::compression::palmdoc_decompress;

fn main() -> anyhow::Result<()> {
    let mut data = File::open("test.mobi")?;
    let mobi = MOBI::from_bytes(&mut data)?;
    eprintln!("{:#?}", mobi.palmdoc_header);
    eprintln!("{:#?}", mobi.header);

    std::fs::create_dir("dump2")?;

    let mut str = Vec::new();
    for i in 1..=mobi.palmdoc_header.record_count
    {
        let record_data = mobi.read_record(i)?;

        File::create(format!("dump2/record_{i}.bin"))?.write_all(&record_data)?;
        let new_text = palmdoc_decompress(&record_data);
        File::create(format!("dump2/record_{i}.txt"))?.write(&new_text)?;
        str.extend_from_slice(&new_text);
    }

    for i in mobi.palmdoc_header.record_count+1..mobi.header.last_content_record_number {
        let record_data = mobi.read_record(i)?;
        File::create(format!("dump2/record_{i}.bin"))?.write_all(&record_data)?;
    }

    for i in mobi.header.last_content_record_number..mobi.pdb.header.number_of_records {
        let record_data = mobi.pdb.read_record(i).unwrap();
        File::create(format!("dump2/record_{i}.bin"))?.write_all(&record_data)?;
    }

    File::create("dump2/record.html".to_string())?.write(String::from_utf8_lossy(&str).as_bytes())?;

    Ok(())
}
