use std::{
  collections::HashMap,
  io::{Read, Write},
  path::{Path, PathBuf},
  sync::Arc,
};

use crate::{Error, ErrorKind};

pub struct Store<V> {
  path: PathBuf,
  items: Vec<HashMap<String, V>>,
  identifier: String,
  serializer: Arc<dyn Fn(&Vec<HashMap<String, V>>, &mut dyn Write) -> crate::Result<()>>,
  deserializer: Arc<dyn Fn(&mut dyn Read) -> crate::Result<Vec<HashMap<String, V>>>>,
}
use std::fmt::Debug;
impl<V: Debug> Debug for Store<V> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Store")
      .field("path", &self.path)
      .field("items", &self.items)
      .field("identifier", &self.identifier)
      .field("serializer", &"Fn")
      .field("deserializer", &"Fn")
      .finish()
  }
}

impl<V: PartialEq + Clone + std::fmt::Display> Store<V> {
  pub fn new<
    P: AsRef<Path>,
    I: AsRef<str>,
    S: Fn(&Vec<HashMap<String, V>>, &mut dyn Write) -> crate::Result<()> + 'static,
    D: Fn(&mut dyn Read) -> crate::Result<Vec<HashMap<String, V>>> + 'static,
  >(
    path: P,
    identifier: I,
    serializer: S,
    deserializer: D,
  ) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
      items: vec![],
      identifier: identifier.as_ref().to_string(),
      serializer: Arc::new(serializer),
      deserializer: Arc::new(deserializer),
    }
  }

  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  pub fn items(&self) -> &Vec<HashMap<String, V>> {
    &self.items
  }

  pub fn identifier(&self) -> &String {
    &self.identifier
  }

  pub fn path_mut(&mut self) -> &mut PathBuf {
    &mut self.path
  }

  pub fn items_mut(&mut self) -> &mut Vec<HashMap<String, V>> {
    &mut self.items
  }

  pub fn identifier_mut(&mut self) -> &mut String {
    &mut self.identifier
  }

  pub fn id_field<'a>(&'a self, obj: &'a HashMap<String, V>) -> Option<(&'a String, &'a V)> {
    for (k, v) in obj {
      if k.eq_ignore_ascii_case(&self.identifier) {
        return Some((k, v));
      }
    }
    None
  }

  pub fn contains(&self, id: &V) -> bool {
    return self.find(id).is_some();
  }

  pub fn find(&self, id: &V) -> Option<&HashMap<String, V>> {
    for item in &self.items {
      if let Some((_id_key, id_val)) = self.id_field(item) {
        if *id_val == *id {
          return Some(item);
        }
      }
    }
    None
  }

  pub fn create(&mut self, obj: HashMap<String, V>) -> crate::Result<usize> {
    let id_value = match self.id_field(&obj) {
      Some((_id_key, id_val)) => id_val,
      None => {
        return Err(Error::new(
          ErrorKind::Unknown,
          Some(format!("missing `{}` field in object", self.identifier)),
          None,
        ));
      }
    };
    if let Some(_) = self.find(id_value) {
      return Err(Error::new(
        ErrorKind::Unknown,
        Some(format!(
          "entity with `{}`={} already exists",
          self.identifier, id_value
        )),
        None,
      ));
    }
    let ret = self.items.len();
    self.items.push(obj);
    Ok(ret)
  }

  pub fn remove(&mut self, id: &V) -> Option<HashMap<String, V>> {
    let found = self.items.iter().enumerate().find(|(item_id, item)| {
      if let Some((_id_key, id_val)) = self.id_field(item) {
        if *id_val == *id {
          return true;
        }
      }
      false
    });
    match found {
      Some((item_id, _item)) => Some(self.items.remove(item_id)),
      None => None,
    }
  }

  pub fn load(&mut self) -> crate::Result<usize> {
    let mut f = std::fs::File::open(&self.path)?;
    self.items = (self.deserializer)(&mut f)?;
    Ok(self.items.len())
  }

  pub fn save(&self) -> crate::Result<()> {
    let mut f = std::fs::File::create(&self.path)?;
    (self.serializer)(&self.items, &mut f)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::Store;

  #[test]
  #[cfg(feature = "json")]
  fn find() {
    use std::collections::HashMap;

    let mut store: Store<serde_json::Value> = Store::new(
      "/tmp/test.json",
      "id",
      |items, writer| {
        serde_json::to_writer_pretty(writer, items)?;
        Ok(())
      },
      |reader| {
        let data = serde_json::from_reader(reader)?;
        Ok(data)
      },
    );
    store
      .create(HashMap::from([
        ("id".to_string(), serde_json::to_value(42).unwrap()),
        (
          "name".to_string(),
          serde_json::to_value("Joe Garcia").unwrap(),
        ),
      ]))
      .unwrap();
    store
      .create(HashMap::from([
        ("id".to_string(), serde_json::to_value(84).unwrap()),
        (
          "name".to_string(),
          serde_json::to_value("Daffy duck").unwrap(),
        ),
      ]))
      .unwrap();
    let found = store.find(&serde_json::to_value(84).unwrap());
    assert_eq!(found, Some(&store.items[1]));
    println!("{:#?}", store);
  }
}
