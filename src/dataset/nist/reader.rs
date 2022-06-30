use std::collections::BTreeMap;
use std::path::Path;

use super::super::reader::FileReader;
use super::{MassAttenCoefRecord, MaterialConstant, MaterialConstantRecord};

use crate::error::Error;
use crate::nuclide::Symbol;

pub struct MaterialConstantReader {
    reader: FileReader,
}

impl MaterialConstantReader {
    pub fn new(path: &Path) -> Self {
        Self {
            reader: FileReader::new(&path.join("material_constants")).skip_lines(2),
        }
    }

    pub fn read(&mut self) -> Result<BTreeMap<Symbol, MaterialConstant>, Error> {
        let mut content = BTreeMap::new();

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let row: MaterialConstantRecord =
                fixed_width::from_str(&buf).map_err(|e| Error::Unexpected(e.into()))?;
            content.insert(row.symbol, row.into());
        }

        Ok(content)
    }
}

pub struct MassAttenCoefReader {
    reader: FileReader,
}

impl MassAttenCoefReader {
    pub fn new(path: &Path, z: u8) -> Self {
        Self {
            reader: FileReader::new(&path.join(format!("{:02}", z))).skip_lines(2),
        }
    }

    pub fn read(&mut self) -> Result<Vec<MassAttenCoefRecord>, Error> {
        let mut content = vec![];

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let row: MassAttenCoefRecord =
                fixed_width::from_str(&buf).map_err(|e| Error::Unexpected(e.into()))?;
            content.push(row);
        }

        Ok(content)
    }
}
