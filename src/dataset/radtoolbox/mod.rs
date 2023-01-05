use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

use crate::error::Error;

pub mod fgr12;
pub mod icrp68;
pub mod icrp72;
mod utils;

#[derive(Debug)]
pub struct RadToolbox3 {
    root_path: PathBuf,
    fgr12: OnceCell<fgr12::Fgr12>,
    icrp68: OnceCell<icrp68::Icrp68>,
    icrp72: OnceCell<icrp72::Icrp72>,
}

impl RadToolbox3 {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let root_path = path.as_ref().to_path_buf();
        if root_path.is_dir() {
            Ok(Self {
                root_path,
                fgr12: OnceCell::new(),
                icrp68: OnceCell::new(),
                icrp72: OnceCell::new(),
            })
        } else {
            Err(Error::Unexpected(anyhow::anyhow!("Invalid data path")))
        }
    }

    pub fn fgr12(&self) -> Result<&fgr12::Fgr12, Error> {
        self.fgr12
            .get_or_try_init(|| fgr12::Fgr12::open(self.root_path.join("fgr12.mdb")))
    }

    pub fn icrp68(&self) -> Result<&icrp68::Icrp68, Error> {
        self.icrp68
            .get_or_try_init(|| icrp68::Icrp68::open(self.root_path.join("icrp68.mdb")))
    }

    pub fn icrp72(&self) -> Result<&icrp72::Icrp72, Error> {
        self.icrp72
            .get_or_try_init(|| icrp72::Icrp72::open(self.root_path.join("icrp72.mdb")))
    }
}
