use std::{
  path::{Path, PathBuf},
  rc::Rc,
};

use crate::{Config, Error, ErrorKind, UserConfig};

#[derive(Clone)]
pub struct Format<T> {
  pub exts: Vec<String>,
  pub serialize: Rc<dyn Fn(&Path, &T) -> crate::Result<()>>,
  pub deserialize: Rc<dyn Fn(&Path) -> crate::Result<T>>,
}

impl<T> Format<T> {
  pub fn new<
    X: AsRef<str>,
    Xi: IntoIterator<Item = X>,
    S: Fn(&Path, &T) -> crate::Result<()> + 'static,
    D: Fn(&Path) -> crate::Result<T> + 'static,
  >(
    exts: Xi,
    serialize: S,
    deserialize: D,
  ) -> Self {
    Self {
      exts: exts
        .into_iter()
        .map(|ext| ext.as_ref().to_string())
        .collect::<Vec<_>>(),
      serialize: Rc::new(serialize),
      deserialize: Rc::new(deserialize),
    }
  }
}

pub fn config_formats() -> Vec<Format<Config>> {
  vec![
    #[cfg(feature = "json")]
    Format::new(
      vec!["json"],
      |path, value| {
        let json = serde_json::to_vec_pretty(value)?;
        std::fs::write(path, json)?;
        Ok(())
      },
      |path| {
        let json = std::fs::read(path)?;
        let cfg: UserConfig = serde_json::from_slice(&json)?;
        Ok(cfg.realize())
      },
    ),
    #[cfg(feature = "toml")]
    Format::new(
      vec!["toml"],
      |path, value| {
        let toml = toml::to_string_pretty(value)?;
        std::fs::write(path, toml)?;
        Ok(())
      },
      |path| {
        let toml = std::fs::read_to_string(path)?;
        let cfg: UserConfig = toml::from_str(&toml)?;
        Ok(cfg.realize())
      },
    ),
    #[cfg(feature = "yaml")]
    Format::new(
      vec!["yaml", "yml"],
      |path, value| {
        let toml = serde_yml::to_string(value)?;
        std::fs::write(path, toml)?;
        Ok(())
      },
      |path| {
        let toml = std::fs::read_to_string(path)?;
        let cfg: UserConfig = serde_yml::from_str(&toml)?;
        Ok(cfg.realize())
      },
    ),
  ]
}

pub fn find_fmt<P: AsRef<Path>>(path: P) -> Option<(Format<Config>, PathBuf)> {
  let pext = match path.as_ref().extension().and_then(|ext| ext.to_str()) {
    Some(ext) => ext,
    None => return None,
  };
  let formats = config_formats();
  for fmt in &formats {
    for ext in &fmt.exts {
      if ext.eq_ignore_ascii_case(pext) {
        return Some((fmt.clone(), path.as_ref().with_extension(ext)));
      }
    }
  }
  None
}
