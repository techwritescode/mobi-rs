use std::fs::File;
use mobi::mobi::MOBI;
use palm_database::builder::PDBBuilder;
use palm_database::PDB;

fn main() -> anyhow::Result<()> {
    let mut mobi = MOBI::new("Perfect World");
    mobi.set_content("<html><head></head><body><p>Test</p></body></html>");

    std::fs::write("output.mobi", &mobi.to_bytes()?)?;

    Ok(())
}