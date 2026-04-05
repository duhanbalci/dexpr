use indexmap::IndexMap;
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::fmt;

/// Value type for the dExpr language
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
  #[default]
  Null,
  Number(Decimal),
  String(SmolStr),
  Boolean(bool),
  NumberList(Vec<Decimal>),
  StringList(Vec<SmolStr>),
  Object(IndexMap<SmolStr, Value>),
}

/// Type tag constants for serialization
pub const TYPE_NULL: u8 = 0x00;
pub const TYPE_NUMBER: u8 = 0x01;
pub const TYPE_STRING: u8 = 0x02;
pub const TYPE_BOOLEAN: u8 = 0x03;
pub const TYPE_NUMBER_LIST: u8 = 0x04;
pub const TYPE_STRING_LIST: u8 = 0x05;
pub const TYPE_OBJECT: u8 = 0x06;

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Value::Null => write!(f, "null"),
      Value::Number(n) => write!(f, "{}", n),
      Value::String(s) => write!(f, "\"{}\"", s),
      Value::Boolean(b) => write!(f, "{}", b),
      Value::NumberList(list) => {
        write!(f, "[")?;
        for (i, val) in list.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}", val)?;
        }
        write!(f, "]")
      }
      Value::StringList(list) => {
        write!(f, "[")?;
        for (i, val) in list.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "\"{}\"", val)?;
        }
        write!(f, "]")
      }
      Value::Object(map) => {
        write!(f, "{{")?;
        for (i, (key, val)) in map.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{}: {}", key, val)?;
        }
        write!(f, "}}")
      }
    }
  }
}

impl Value {
  /// Get the type tag for bytecode serialization
  pub fn type_tag(&self) -> u8 {
    match self {
      Value::Null => TYPE_NULL,
      Value::Number(_) => TYPE_NUMBER,
      Value::String(_) => TYPE_STRING,
      Value::Boolean(_) => TYPE_BOOLEAN,
      Value::NumberList(_) => TYPE_NUMBER_LIST,
      Value::StringList(_) => TYPE_STRING_LIST,
      Value::Object(_) => TYPE_OBJECT,
    }
  }

  /// Get the type name as a string (for error messages)
  pub fn type_name(&self) -> &'static str {
    match self {
      Value::Null => "Null",
      Value::Number(_) => "Number",
      Value::String(_) => "String",
      Value::Boolean(_) => "Boolean",
      Value::NumberList(_) => "NumberList",
      Value::StringList(_) => "StringList",
      Value::Object(_) => "Object",
    }
  }

  /// Serialize the value to bytes for bytecode
  pub fn serialize(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(self.type_tag());

    match self {
      Value::Null => {
        // No additional data for null
      }
      Value::Number(n) => {
        bytes.extend_from_slice(&n.serialize());
      }
      Value::String(s) => {
        // String length (2 bytes)
        bytes.push((s.len() >> 8) as u8);
        bytes.push(s.len() as u8);
        // String data
        bytes.extend_from_slice(s.as_bytes());
      }
      Value::Boolean(b) => {
        bytes.push(if *b { 1 } else { 0 });
      }
      Value::NumberList(list) => {
        // List length (2 bytes)
        bytes.push((list.len() >> 8) as u8);
        bytes.push(list.len() as u8);
        // List items
        for n in list {
          bytes.extend_from_slice(&n.serialize());
        }
      }
      Value::StringList(list) => {
        // List length (2 bytes)
        bytes.push((list.len() >> 8) as u8);
        bytes.push(list.len() as u8);
        // List items
        for s in list {
          // String length (2 bytes)
          bytes.push((s.len() >> 8) as u8);
          bytes.push(s.len() as u8);
          // String data
          bytes.extend_from_slice(s.as_bytes());
        }
      }
      Value::Object(map) => {
        // Entry count (2 bytes)
        bytes.push((map.len() >> 8) as u8);
        bytes.push(map.len() as u8);
        // Entries: key (string) + value (recursive)
        for (key, val) in map {
          bytes.push((key.len() >> 8) as u8);
          bytes.push(key.len() as u8);
          bytes.extend_from_slice(key.as_bytes());
          bytes.extend_from_slice(&val.serialize());
        }
      }
    }

    bytes
  }

  pub fn is_null(&self) -> bool {
    matches!(self, Value::Null)
  }
}

impl From<Decimal> for Value {
  fn from(n: Decimal) -> Self {
    Value::Number(n)
  }
}

impl From<i64> for Value {
  fn from(n: i64) -> Self {
    Value::Number(Decimal::from(n))
  }
}

impl From<i32> for Value {
  fn from(n: i32) -> Self {
    Value::Number(Decimal::from(n))
  }
}

impl From<f64> for Value {
  fn from(n: f64) -> Self {
    Value::Number(Decimal::try_from(n).unwrap_or_default())
  }
}

impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Value::Boolean(b)
  }
}

impl From<&str> for Value {
  fn from(s: &str) -> Self {
    Value::String(SmolStr::new(s))
  }
}

impl From<String> for Value {
  fn from(s: String) -> Self {
    Value::String(SmolStr::new(&s))
  }
}

impl From<SmolStr> for Value {
  fn from(s: SmolStr) -> Self {
    Value::String(s)
  }
}

impl From<Vec<Decimal>> for Value {
  fn from(v: Vec<Decimal>) -> Self {
    Value::NumberList(v)
  }
}

impl From<Vec<SmolStr>> for Value {
  fn from(v: Vec<SmolStr>) -> Self {
    Value::StringList(v)
  }
}

impl From<IndexMap<SmolStr, Value>> for Value {
  fn from(m: IndexMap<SmolStr, Value>) -> Self {
    Value::Object(m)
  }
}

impl Value {
  /// Deserialize a value from bytes
  pub fn deserialize(bytes: &[u8]) -> Result<(Value, usize), String> {
    if bytes.is_empty() {
      return Err("Empty buffer".to_string());
    }

    let type_tag = bytes[0];
    let mut pos = 1;

    match type_tag {
      TYPE_NULL => Ok((Value::Null, pos)),
      TYPE_NUMBER => {
        if bytes.len() < pos + 16 {
          return Err("Insufficient bytes for Number".to_string());
        }
        let mut decimal_bytes = [0u8; 16];
        decimal_bytes.copy_from_slice(&bytes[pos..pos + 16]);
        pos += 16;
        Ok((Value::Number(Decimal::deserialize(decimal_bytes)), pos))
      }
      TYPE_STRING => {
        if bytes.len() < pos + 2 {
          return Err("Insufficient bytes for String length".to_string());
        }
        let len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;

        if bytes.len() < pos + len {
          return Err("Insufficient bytes for String data".to_string());
        }
        let s = match std::str::from_utf8(&bytes[pos..pos + len]) {
          Ok(s) => s,
          Err(_) => return Err("Invalid UTF-8 in String".to_string()),
        };
        pos += len;

        Ok((Value::String(s.into()), pos))
      }
      TYPE_BOOLEAN => {
        if bytes.len() < pos + 1 {
          return Err("Insufficient bytes for Boolean".to_string());
        }
        let b = bytes[pos] != 0;
        pos += 1;
        Ok((Value::Boolean(b), pos))
      }
      TYPE_NUMBER_LIST => {
        if bytes.len() < pos + 2 {
          return Err("Insufficient bytes for NumberList length".to_string());
        }
        let len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;

        let total_bytes = len * 16;
        if bytes.len() < pos + total_bytes {
          return Err("Insufficient bytes for NumberList items".to_string());
        }

        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
          let mut decimal_bytes = [0u8; 16];
          decimal_bytes.copy_from_slice(&bytes[pos..pos + 16]);
          pos += 16;
          list.push(Decimal::deserialize(decimal_bytes));
        }

        Ok((Value::NumberList(list), pos))
      }
      TYPE_STRING_LIST => {
        if bytes.len() < pos + 2 {
          return Err("Insufficient bytes for StringList length".to_string());
        }
        let len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;

        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
          if bytes.len() < pos + 2 {
            return Err("Insufficient bytes for StringList item length".to_string());
          }
          let str_len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
          pos += 2;

          if bytes.len() < pos + str_len {
            return Err("Insufficient bytes for StringList item data".to_string());
          }
          let s = match std::str::from_utf8(&bytes[pos..pos + str_len]) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in StringList item".to_string()),
          };
          pos += str_len;

          list.push(s.into());
        }

        Ok((Value::StringList(list), pos))
      }
      TYPE_OBJECT => {
        if bytes.len() < pos + 2 {
          return Err("Insufficient bytes for Object length".to_string());
        }
        let len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos += 2;

        let mut map = IndexMap::with_capacity(len);
        for _ in 0..len {
          // Read key
          if bytes.len() < pos + 2 {
            return Err("Insufficient bytes for Object key length".to_string());
          }
          let key_len = u16::from_be_bytes([bytes[pos], bytes[pos + 1]]) as usize;
          pos += 2;
          if bytes.len() < pos + key_len {
            return Err("Insufficient bytes for Object key data".to_string());
          }
          let key = match std::str::from_utf8(&bytes[pos..pos + key_len]) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 in Object key".to_string()),
          };
          pos += key_len;

          // Read value (recursive)
          let (val, val_bytes) = Value::deserialize(&bytes[pos..])?;
          pos += val_bytes;

          map.insert(key.into(), val);
        }

        Ok((Value::Object(map), pos))
      }
      _ => Err(format!("Unknown type tag: {}", type_tag)),
    }
  }

  /// Create a Value from a JSON string.
  ///
  /// Mapping:
  /// - `null` → `Null`
  /// - `true`/`false` → `Boolean`
  /// - number → `Number` (Decimal)
  /// - string → `String`
  /// - array of numbers → `NumberList`
  /// - array of strings → `StringList`
  /// - object → `Object` (recursive)
  ///
  /// ```
  /// use dexpr::ast::value::Value;
  /// use rust_decimal_macros::dec;
  ///
  /// let val = Value::from_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
  /// if let Value::Object(map) = &val {
  ///     assert_eq!(map.get("name").unwrap(), &Value::String("Alice".into()));
  ///     assert_eq!(map.get("age").unwrap(), &Value::Number(dec!(30)));
  /// }
  /// ```
  pub fn from_json(json: &str) -> Result<Value, String> {
    let v: serde_json::Value = serde_json::from_str(json)
      .map_err(|e| format!("JSON parse error: {}", e))?;
    Self::from_json_value(&v)
  }

  /// Convert a serde_json::Value to a dexpr Value.
  pub fn from_json_value(v: &serde_json::Value) -> Result<Value, String> {
    match v {
      serde_json::Value::Null => Ok(Value::Null),
      serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
      serde_json::Value::Number(n) => {
        // Try integer first, then float
        if let Some(i) = n.as_i64() {
          Ok(Value::Number(Decimal::from(i)))
        } else if let Some(f) = n.as_f64() {
          Decimal::try_from(f)
            .map(Value::Number)
            .map_err(|e| format!("Cannot convert {} to Decimal: {}", f, e))
        } else {
          Err(format!("Unsupported JSON number: {}", n))
        }
      }
      serde_json::Value::String(s) => Ok(Value::String(SmolStr::new(s))),
      serde_json::Value::Array(arr) => {
        if arr.is_empty() {
          return Ok(Value::StringList(Vec::new()));
        }
        // Check if all elements are the same type
        let first = &arr[0];
        if first.is_number() && arr.iter().all(|v| v.is_number()) {
          let mut nums = Vec::with_capacity(arr.len());
          for item in arr {
            if let Value::Number(n) = Self::from_json_value(item)? {
              nums.push(n);
            }
          }
          Ok(Value::NumberList(nums))
        } else if first.is_string() && arr.iter().all(|v| v.is_string()) {
          let strings: Vec<SmolStr> = arr.iter()
            .filter_map(|v| v.as_str().map(SmolStr::new))
            .collect();
          Ok(Value::StringList(strings))
        } else {
          Err("Arrays must contain all numbers or all strings".to_string())
        }
      }
      serde_json::Value::Object(obj) => {
        let mut map = IndexMap::with_capacity(obj.len());
        for (k, v) in obj {
          map.insert(SmolStr::new(k), Self::from_json_value(v)?);
        }
        Ok(Value::Object(map))
      }
    }
  }
}
