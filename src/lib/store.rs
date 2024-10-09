use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;
use std::{
  collections::HashMap,
  io::{Read, Write},
  path::{Path, PathBuf},
  sync::Arc,
};

use log::error;

use crate::{Error, ErrorKind, Status, Value};

pub struct Store {
  path: PathBuf,
  items: Vec<HashMap<String, Value>>,
  identifier: String,
  serializer: Arc<dyn Fn(&Vec<HashMap<String, Value>>, &mut dyn Write) -> crate::Result<()>>,
  deserializer: Arc<dyn Fn(&mut dyn Read) -> crate::Result<Vec<HashMap<String, Value>>>>,
}

fn convert_items<V: Clone, R, F: Fn(V) -> crate::Result<R>>(
  items: &Vec<HashMap<String, V>>,
  f: F,
) -> crate::Result<Vec<HashMap<String, R>>> {
  let mut ret = Vec::new();
  for obj in items {
    let mut new_obj = HashMap::new();
    for (key, val) in obj {
      new_obj.insert(key.clone(), f(val.clone())?);
    }
    ret.push(new_obj);
  }
  Ok(ret)
}

#[cfg(feature = "json")]
impl Store {
  fn json_deserialize(r: &mut dyn Read) -> crate::Result<Vec<HashMap<String, Value>>> {
    let data: Vec<HashMap<String, serde_json::Value>> = serde_json::from_reader(r)?;
    Ok(convert_items(&data, |val| Value::try_from_json(val))?)
  }

  fn json_serialize(
    items: &Vec<HashMap<String, Value>>,
    writer: &mut dyn Write,
  ) -> crate::Result<()> {
    let ret = convert_items(items, |val| Ok(val.to_json()))?;
    serde_json::to_writer_pretty(writer, &ret)?;
    Ok(())
  }

  pub fn json<P: AsRef<Path>, I: AsRef<str>>(path: P, identifier: I) -> Self {
    Self::new(
      path,
      identifier,
      Self::json_serialize,
      Self::json_deserialize,
    )
  }
}

#[cfg(feature = "toml")]
impl Store {
  fn toml_deserialize(r: &mut dyn Read) -> crate::Result<Vec<HashMap<String, Value>>> {
    let mut buf = String::new();
    r.read_to_string(&mut buf);
    let data: Vec<HashMap<String, toml::Value>> = toml::from_str(&buf)?;
    Ok(convert_items(&data, |val| Value::try_from_toml(val))?)
  }

  fn toml_serialize(
    items: &Vec<HashMap<String, Value>>,
    writer: &mut dyn Write,
  ) -> crate::Result<()> {
    let ret = convert_items(items, |val| val.to_toml())?;
    let buf = toml::to_string_pretty(&ret)?;
    writer.write(buf.as_bytes())?;
    Ok(())
  }

  pub fn toml<P: AsRef<Path>, I: AsRef<str>>(path: P, identifier: I) -> Self {
    Self::new(
      path,
      identifier,
      Self::toml_serialize,
      Self::toml_deserialize,
    )
  }
}

#[cfg(feature = "yaml")]
impl Store {
  fn yaml_deserialize(r: &mut dyn Read) -> crate::Result<Vec<HashMap<String, Value>>> {
    let data: Vec<HashMap<String, serde_yml::Value>> = serde_yml::from_reader(r)?;
    Ok(convert_items(&data, |val| Value::try_from_yaml(val))?)
  }

  fn yaml_serialize(
    items: &Vec<HashMap<String, Value>>,
    writer: &mut dyn Write,
  ) -> crate::Result<()> {
    let ret = convert_items(items, |val| Ok(val.to_yaml()))?;
    serde_yml::to_writer(writer, &ret)?;
    Ok(())
  }

  pub fn yaml<P: AsRef<Path>, I: AsRef<str>>(path: P, identifier: I) -> Self {
    Self::new(
      path,
      identifier,
      Self::yaml_serialize,
      Self::yaml_deserialize,
    )
  }
}

impl Store {
  pub fn new<
    P: AsRef<Path>,
    I: AsRef<str>,
    S: Fn(&Vec<HashMap<String, Value>>, &mut dyn Write) -> crate::Result<()> + 'static,
    D: Fn(&mut dyn Read) -> crate::Result<Vec<HashMap<String, Value>>> + 'static,
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

  pub fn items(&self) -> &Vec<HashMap<String, Value>> {
    &self.items
  }

  pub fn identifier(&self) -> &String {
    &self.identifier
  }

  pub fn path_mut(&mut self) -> &mut PathBuf {
    &mut self.path
  }

  pub fn items_mut(&mut self) -> &mut Vec<HashMap<String, Value>> {
    &mut self.items
  }

  pub fn identifier_mut(&mut self) -> &mut String {
    &mut self.identifier
  }

  pub fn id_field<'a>(
    &'a self,
    obj: &'a HashMap<String, Value>,
  ) -> Option<(&'a String, &'a Value)> {
    for (k, v) in obj {
      if k.eq_ignore_ascii_case(&self.identifier) {
        return Some((k, v));
      }
    }
    None
  }

  pub fn contains(&self, id: &Value) -> bool {
    return self.find(id).is_some();
  }

  pub fn find(&self, id: &Value) -> Option<&HashMap<String, Value>> {
    for item in &self.items {
      if let Some((_id_key, id_val)) = self.id_field(item) {
        if id_val.loose_eq(id) {
          return Some(item);
        }
      }
    }
    None
  }

  pub fn create(&mut self, obj: HashMap<String, Value>) -> crate::Result<usize> {
    let id_value = match self.id_field(&obj) {
      Some((_id_key, id_val)) => id_val,
      None => {
        return Err(Error::new(
          ErrorKind::Api(Status::BadRequest),
          Some(format!("missing `{}` field in object", self.identifier)),
          None,
        ));
      }
    };
    if let Some(_) = self.find(id_value) {
      return Err(Error::new(
        ErrorKind::Api(Status::Conflict),
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

  pub fn remove(&mut self, id: &Value) -> Option<HashMap<String, Value>> {
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

impl Debug for Store {
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

#[cfg(test)]
mod tests {
  use crate::Value;

  use super::Store;

  #[test]
  fn find() {
    use std::collections::HashMap;

    let mut store = Store::json("/tmp/test.json", "id");
    store
      .create(HashMap::from([
        ("id".to_string(), Value::from(42)),
        ("name".to_string(), Value::from("Joe Garcia")),
      ]))
      .unwrap();
    store
      .create(HashMap::from([
        ("id".to_string(), Value::from(84)),
        ("name".to_string(), Value::from("Daffy duck")),
      ]))
      .unwrap();
    let found = store.find(&Value::from(84));
    assert_eq!(found, Some(&store.items[1]));
    println!("{:#?}", store);
  }
}
