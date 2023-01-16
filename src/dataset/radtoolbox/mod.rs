use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::OnceCell;

use crate::error::Error;

pub mod fgr12;
pub mod icrp68;
pub mod icrp72;
mod utils;

#[derive(Debug)]
pub struct RadToolbox3 {
    root_path: PathBuf,
    fgr12: OnceCell<Arc<fgr12::Fgr12>>,
    icrp68: OnceCell<Arc<icrp68::Icrp68>>,
    icrp72: OnceCell<Arc<icrp72::Icrp72>>,
    icrp107: OnceCell<Arc<super::Icrp107>>,
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
                icrp107: OnceCell::new(),
            })
        } else {
            Err(Error::Unexpected(anyhow::anyhow!("Invalid data path")))
        }
    }

    pub fn fgr12(&self) -> Result<Arc<fgr12::Fgr12>, Error> {
        let data = self.fgr12.get_or_try_init(|| {
            fgr12::Fgr12::open(self.root_path.join("fgr12.mdb")).map(Arc::new)
        })?;
        Ok(data.clone())
    }

    pub fn icrp68(&self) -> Result<Arc<icrp68::Icrp68>, Error> {
        let data = self.icrp68.get_or_try_init(|| {
            icrp68::Icrp68::open(self.root_path.join("icrp68.mdb")).map(Arc::new)
        })?;
        Ok(data.clone())
    }

    pub fn icrp72(&self) -> Result<Arc<icrp72::Icrp72>, Error> {
        let data = self.icrp72.get_or_try_init(|| {
            icrp72::Icrp72::open(self.root_path.join("icrp72.mdb")).map(Arc::new)
        })?;
        Ok(data.clone())
    }

    pub fn icrp107(&self) -> Result<Arc<super::Icrp107>, Error> {
        let data = self
            .icrp107
            .get_or_try_init(|| super::Icrp107::open(&self.root_path).map(Arc::new))?;
        Ok(data.clone())
    }
}
