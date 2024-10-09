use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::Display;

use serde::de::Visitor;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};

use crate::{Error, ErrorKind};

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
  Null,
  Bool(bool),
  Float(f64),
  Integer(i128),
  Unsigned(u128),
  String(String),
  Map(HashMap<String, Value>),
  Array(Vec<Value>),
}

impl Value {
  pub fn loose_eq(&self, other: &Value) -> bool {
    format!("{}", self).eq(&format!("{}", other))
  }
}
impl Default for Value {
  fn default() -> Self {
    Self::Null
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Null => "null".to_string(),
        Self::Bool(v) => match v {
          true => "true",
          false => "false",
        }
        .to_string(),
        Self::Float(v) => format!("{}", v),
        Self::Integer(v) => format!("{}", v),
        Self::Unsigned(v) => format!("{}", v),
        Self::String(v) => format!("{}", v),
        Self::Map(v) => format!("{:?}", v),
        Self::Array(v) => format!("{:?}", v),
      }
    )
  }
}

macro_rules! impl_value {
  ($vty: expr, $($ty: ty),+) => {
    $(
      impl From<$ty> for Value {
        fn from(value: $ty) -> Self {
          $vty(value.into())
        }
      }

      impl From<Option<$ty>> for Value {
        fn from(value: Option<$ty>) -> Self {
          if let Some(v) = value {
            $vty(v.into())
          } else {
            Self::Null
          }
        }
      }
    )+
  };
}

impl_value!(Value::Bool, bool);
impl_value!(Value::Float, f32, f64);
impl_value!(Value::Integer, i8, i16, i32, i64, i128);
impl_value!(Value::Unsigned, u8, u16, u32, u64, u128);
impl_value!(Value::String, &str, String);

impl From<HashMap<String, Value>> for Value {
  fn from(value: HashMap<String, Value>) -> Self {
    Value::Map(
      value
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<_, _>>(),
    )
  }
}

impl From<BTreeMap<String, Value>> for Value {
  fn from(value: BTreeMap<String, Value>) -> Self {
    Value::Map(
      value
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<_, _>>(),
    )
  }
}

impl<const N: usize> From<&[Value; N]> for Value {
  fn from(value: &[Value; N]) -> Self {
    Value::Array(value.iter().map(|v| v.clone()).collect::<Vec<_>>())
  }
}

impl<const N: usize> From<[Value; N]> for Value {
  fn from(value: [Value; N]) -> Self {
    Value::Array(value.iter().map(|v| v.clone()).collect::<Vec<_>>())
  }
}

impl From<Vec<Value>> for Value {
  fn from(value: Vec<Value>) -> Self {
    Value::Array(value.iter().map(|v| v.clone()).collect::<Vec<_>>())
  }
}

impl From<VecDeque<Value>> for Value {
  fn from(value: VecDeque<Value>) -> Self {
    Value::Array(value.iter().map(|v| v.clone()).collect::<Vec<_>>())
  }
}

#[cfg(feature = "json")]
impl TryFrom<serde_json::Value> for Value {
  type Error = crate::Error;

  fn try_from(value: serde_json::Value) -> crate::Result<Self> {
    Self::try_from_json(value)
  }
}

#[cfg(feature = "toml")]
impl TryFrom<toml::Value> for Value {
  type Error = crate::Error;

  fn try_from(value: toml::Value) -> crate::Result<Self> {
    Self::try_from_toml(value)
  }
}

#[cfg(feature = "yaml")]
impl TryFrom<serde_yml::Value> for Value {
  type Error = crate::Error;

  fn try_from(value: serde_yml::Value) -> crate::Result<Self> {
    Self::try_from_yaml(value)
  }
}

#[cfg(feature = "json")]
impl Value {
  pub fn try_from_json(value: serde_json::Value) -> crate::Result<Self> {
    Ok(match value {
      serde_json::Value::Null => Self::Null,
      serde_json::Value::Bool(v) => Self::Bool(v),
      serde_json::Value::Number(v) => {
        if v.is_u64() {
          Self::Unsigned(v.as_u64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid unsigned value: {}", v)),
              None,
            )
          })? as u128)
        } else if v.is_i64() {
          Self::Integer(v.as_i64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid signed value: {}", v)),
              None,
            )
          })? as i128)
        } else {
          Self::Float(v.as_f64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid floating value: {}", v)),
              None,
            )
          })? as f64)
        }
      }
      serde_json::Value::String(v) => Self::String(v),
      serde_json::Value::Array(v) => {
        let mut ret = vec![];
        for val in v {
          ret.push(Value::try_from_json(val)?);
        }
        Self::Array(ret)
      }
      serde_json::Value::Object(v) => {
        let mut ret = HashMap::new();
        for (key, val) in v {
          ret.insert(key, Value::try_from_json(val)?);
        }
        Self::Map(ret)
      }
    })
  }

  pub fn to_json(&self) -> serde_json::Value {
    match self {
      Self::Null => serde_json::Value::Null,
      Self::Bool(v) => serde_json::Value::Bool(v.clone()),
      Self::Float(v) => serde_json::Value::Number(serde_json::Number::from_f64(v.clone()).unwrap()),
      Self::Integer(v) => serde_json::Value::Number(serde_json::Number::from(v.clone() as i64)),
      Self::Unsigned(v) => serde_json::Value::Number(serde_json::Number::from(v.clone() as u64)),
      Self::String(v) => serde_json::Value::String(v.clone()),
      Self::Map(v) => serde_json::Value::Object(serde_json::Map::from_iter(
        v.iter()
          .map(|(k, v)| (k.clone(), v.to_json()))
          .collect::<HashMap<_, _>>(),
      )),
      Self::Array(v) => serde_json::Value::Array(Vec::from_iter(
        v.iter().map(|v| v.to_json()).collect::<Vec<_>>(),
      )),
    }
  }
}

#[cfg(feature = "toml")]
impl Value {
  pub fn try_from_toml(value: toml::Value) -> crate::Result<Self> {
    Ok(match value {
      toml::Value::Boolean(v) => Self::Bool(v),
      toml::Value::Integer(v) => Self::Integer(v as i128),
      toml::Value::Float(v) => Self::Float(v),
      toml::Value::String(v) => Self::String(v),
      toml::Value::Datetime(v) => Self::String(v.to_string()),
      toml::Value::Array(v) => {
        let mut ret = vec![];
        for val in v {
          ret.push(Value::try_from(val)?);
        }
        Self::Array(ret)
      }
      toml::Value::Table(v) => {
        let mut ret = HashMap::new();
        for (key, val) in v {
          ret.insert(key, Value::try_from(val)?);
        }
        Self::Map(ret)
      }
    })
  }

  pub fn to_toml(&self) -> crate::Result<toml::Value> {
    Ok(match self {
      Self::Null => {
        return Err(Error::new(
          ErrorKind::Parse,
          Some(format!("null values do not exist in toml")),
          None,
        ))
      }
      Self::Bool(v) => toml::Value::Boolean(v.clone()),
      Self::Float(v) => toml::Value::Float(*v),
      Self::Integer(v) => toml::Value::Integer(*v as i64),
      Self::Unsigned(v) => toml::Value::Integer(*v as i64),
      Self::String(v) => toml::Value::String(v.clone()),
      Self::Map(v) => {
        let mut ret = toml::Table::new();
        for (k, v) in v {
          ret.insert(k.clone(), v.to_toml()?);
        }
        toml::Value::Table(ret)
      }
      Self::Array(v) => {
        let mut ret = Vec::new();
        for v in v {
          ret.push(v.to_toml()?);
        }
        toml::Value::Array(ret)
      }
    })
  }
}

#[cfg(feature = "yaml")]
impl Value {
  pub fn try_from_yaml(value: serde_yml::Value) -> crate::Result<Self> {
    Ok(match value {
      serde_yml::Value::Bool(v) => Self::Bool(v),
      serde_yml::Value::Null => Self::Null,
      serde_yml::Value::Number(v) => {
        if v.is_f64() {
          Self::Float(v.as_f64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid floating value: {}", v)),
              None,
            )
          })?)
        } else if v.is_i64() {
          Self::Integer(v.as_i64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid signed value: {}", v)),
              None,
            )
          })? as i128)
        } else {
          Self::Unsigned(v.as_u64().ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!("invalid unsigned value: {}", v)),
              None,
            )
          })? as u128)
        }
      }
      serde_yml::Value::String(v) => Self::String(v),
      serde_yml::Value::Tagged(v) => {
        return Err(Error::new(
          ErrorKind::Parse,
          Some(format!("unknown tagged value type '{:?}'", v)),
          None,
        ))
      }

      serde_yml::Value::Sequence(v) => {
        let mut ret = vec![];
        for val in v {
          ret.push(Value::try_from(val)?);
        }
        Self::Array(ret)
      }
      serde_yml::Value::Mapping(v) => {
        let mut ret = HashMap::new();
        for (key, val) in v {
          ret.insert(Value::try_from(key)?.to_string(), Value::try_from(val)?);
        }
        Self::Map(ret)
      }
    })
  }

  pub fn to_yaml(&self) -> serde_yml::Value {
    match self {
      Self::Null => serde_yml::Value::Null,
      Self::Bool(v) => serde_yml::Value::Bool(v.clone()),
      Self::Float(v) => serde_yml::Value::Number(serde_yml::Number::from(v.clone())),
      Self::Integer(v) => serde_yml::Value::Number(serde_yml::Number::from(v.clone() as i64)),
      Self::Unsigned(v) => serde_yml::Value::Number(serde_yml::Number::from(v.clone() as u64)),
      Self::String(v) => serde_yml::Value::String(v.clone()),
      Self::Map(v) => serde_yml::Value::Mapping(serde_yml::Mapping::from_iter(
        v.iter()
          .map(|(k, v)| (Self::from(k.clone()).to_yaml(), v.to_yaml()))
          .collect::<HashMap<_, _>>(),
      )),
      Self::Array(v) => serde_yml::Value::Sequence(Vec::from_iter(
        v.iter().map(|v| v.to_yaml()).collect::<Vec<_>>(),
      )),
    }
  }
}

// impl_value!(Value::Map, HashMap<String, Value>); //, BTreeMap<String, Box<Value>>);
// impl_value!(Value::Array, &[Value], Vec<Value>, VecDeque<Value>);

impl Serialize for Value {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Self::Null => serializer.serialize_none(),
      Self::Bool(v) => serializer.serialize_bool(*v),
      Self::Float(v) => serializer.serialize_f64(*v),
      Self::Integer(v) => serializer.serialize_i128(*v),
      Self::Unsigned(v) => serializer.serialize_u128(*v),
      Self::String(v) => serializer.serialize_str(v.as_str()),
      Self::Map(v) => {
        let mut map = serializer.serialize_map(Some(v.len()))?;
        for (k, v) in v {
          map.serialize_entry(k, v)?;
        }
        map.end()
      }
      Self::Array(vec) => {
        let mut seq = serializer.serialize_seq(Some(vec.len()))?;
        for v in vec {
          seq.serialize_element(v)?;
        }
        seq.end()
      }
    }
  }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
  type Value = crate::Value;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("an integer between -2^31 and 2^31")
  }

  fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::from(value))
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut v = vec![];
    while let Some(elem) = seq.next_element()? {
      v.push(elem);
    }
    Ok(Value::from(v))
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut m = HashMap::new();
    while let Some((key, value)) = map.next_entry()? {
      m.insert(key, value);
    }
    Ok(Value::from(m))
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Value::Null)
  }
  // Similar for other methods:
  //   - visit_i16
  //   - visit_u8
  //   - visit_u16
  //   - visit_u32
  //   - visit_u64
}

impl<'de> Deserialize<'de> for Value {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_any(ValueVisitor)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::{BTreeMap, HashMap, VecDeque};

  use crate::Value;

  macro_rules! impl_from_test {
    ($ty:ty, $exp_v:expr$(, $from_v:expr )+ ) => {
      paste::paste! {
        #[test]
        fn [<from_ $ty:lower>]() {
          $(
            assert_eq!(Value::from($from_v), Value::$ty($exp_v));
          )+
        }
      }
    };
  }

  impl_from_test!(Bool, true, true);
  impl_from_test!(Float, 42f64, 42f32, 42f64);
  impl_from_test!(Integer, 42i128, 42i8, 42i16, 42i32, 42i64, 42i128);
  impl_from_test!(Unsigned, 42u128, 42u8, 42u16, 42u32, 42u64, 42u128);
  impl_from_test!(String, String::from("test"), "test", String::from("test"));
  impl_from_test!(
    Map,
    HashMap::from([(String::from("key"), Value::Integer(42))]),
    HashMap::from([(String::from("key"), Value::Integer(42))]),
    BTreeMap::from([(String::from("key"), Value::Integer(42))])
  );
  impl_from_test!(
    Array,
    Vec::from([Value::Integer(42)]),
    Vec::from([Value::Integer(42)]),
    VecDeque::from([Value::Integer(42)]),
    &[Value::Integer(42)],
    [Value::Integer(42)]
  );
}
