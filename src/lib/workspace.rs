use std::path::{Path, PathBuf};

use crate::{Config, Error, ErrorKind, UserConfig};

#[derive(Debug)]
pub struct Workspace {
  pub path: PathBuf,
  pub config: Config,
}

impl Workspace {
  pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    Ok(Workspace {
      path: path.as_ref().to_path_buf(),
      config: Config::load(path)?,
    })
  }

  pub fn create<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    if path.as_ref().exists() {
      return Err(Error::new(
        ErrorKind::IO,
        Some(format!(
          "{}: workspace already initialized",
          path.as_ref().display()
        )),
        None,
      ));
    }
    let w = Workspace {
      path: path.as_ref().to_path_buf(),
      config: Config::default(),
    };
    w.config.save(path)?;
    Ok(w)
  }
}
