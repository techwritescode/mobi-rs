use crate::{PDBHeader, PDBRecord, PDB};

#[derive(Debug)]
pub enum PDBError {
    IoError(std::io::Error),
    InvalidData(String),
    MissingField(String),
}

pub struct PDBBuilder {
    name: Option<String>,
    attributes: u16,
    version: u16,
    creation_time: Option<chrono::NaiveDateTime>,
    modification_time: Option<chrono::NaiveDateTime>,
    last_backup_date: Option<chrono::NaiveDateTime>,
    modification_number: u32,
    app_info_id: u32,
    sort_info_id: u32,
    type_: Option<String>,
    creator: Option<String>,
    unique_id_seed: u32,
    next_record_list_id: u32,
    number_of_records: u16,
    records: Vec<(u32, u8, Vec<u8>)>, // (unique_id, attributes, data)
}

impl PDBBuilder {
    pub fn new() -> Self {
        PDBBuilder {
            name: None,
            attributes: 0,
            version: 0,
            creation_time: None,
            modification_time: None,
            last_backup_date: None,
            modification_number: 0,
            app_info_id: 0,
            sort_info_id: 0,
            type_: None,
            creator: None,
            unique_id_seed: 0,
            next_record_list_id: 0,
            number_of_records: 0,
            records: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(name.len() <= 31, "Name must be at most 31 characters long");
        self.name = Some(name);
        self
    }

    pub fn attributes(mut self, attributes: u16) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn version(mut self, version: u16) -> Self {
        self.version = version;
        self
    }

    pub fn type_(mut self, type_: impl Into<String>) -> Self {
        let type_ = type_.into();
        assert_eq!(type_.len(), 4, "Type must be exactly 4 characters long");
        self.type_ = Some(type_);
        self
    }

    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        let creator = creator.into();
        assert_eq!(creator.len(), 4, "Creator must be exactly 4 characters long");
        self.creator = Some(creator);
        self
    }

    pub fn add_record(mut self, unique_id: u32, attributes: u8, data: &[u8]) -> Self {
        self.records.push((unique_id, attributes, data.to_vec()));
        self
    }

    pub fn build(self) -> Result<PDB, PDBError> {
        let epoch = chrono::NaiveDate::from_ymd_opt(1904, 1, 1)
            .and_then(|t| t.and_hms_opt(0, 0, 0))
            .unwrap();
        let name = self.name.ok_or(PDBError::MissingField("name".to_owned()))?;

        Ok(PDB {
            header: PDBHeader {
                name,
                attributes: self.attributes,
                version: self.version,
                creation_time: self.creation_time.unwrap_or(chrono::Utc::now().naive_utc()),
                modification_time: self.modification_time.unwrap_or(chrono::Utc::now().naive_utc()),
                last_backup_date: self.modification_time.unwrap_or(epoch),
                modification_number: self.modification_number,
                app_info_id: self.app_info_id,
                sort_info_id: self.sort_info_id,
                type_: self.type_.ok_or(PDBError::MissingField("type".to_owned()))?,
                creator: self.creator.ok_or(PDBError::MissingField("creator".to_owned()))?,
                unique_id_seed: self.unique_id_seed,
                next_record_list_id: self.next_record_list_id,
                number_of_records: self.records.len() as u16,
            },
            records: vec![],
            record_data: self.records.iter().map(|(_, _, data)| data.clone()).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdb_builder() {
        let pdb = PDBBuilder::new()
            .name("test".to_owned())
            .attributes(0)
            .version(0)
            .type_("BOOK")
            .creator("MOBI")
            .add_record(1, 0, b"Record 1 data")
            .add_record(2, 1, b"Record 2 data")
            .build()
            .expect("Failed to build PDB");

        assert_eq!(pdb.header.name, "test");
        assert_eq!(pdb.records.len(), 2);
        assert_eq!(pdb.record_data.len(), 2);
        assert_eq!(pdb.record_data[0], b"Record 1 data");
        assert_eq!(pdb.record_data[1], b"Record 2 data");
    }
}