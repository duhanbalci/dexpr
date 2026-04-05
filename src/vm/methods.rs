use crate::ast::value::Value;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use smol_str::{SmolStr, StrExt};

use super::error::VMError;
use super::vm::VM;

impl<'a> VM<'a> {
  /// Dispatch a method call on a value
  pub(super) fn dispatch_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &SmolStr,
    args: &[Value],
  ) -> Result<(), VMError> {
    match &self.registers[obj] {
      Value::String(_) => self.dispatch_string_method(dest, obj, method, args),
      Value::StringList(_) => self.dispatch_string_list_method(dest, obj, method, args),
      Value::NumberList(_) => self.dispatch_number_list_method(dest, obj, method, args),
      Value::Object(_) => self.dispatch_object_method(dest, obj, method, args),
      _ => {
        // Try external methods for any type
        let obj_val = &self.registers[obj];
        let type_name: SmolStr = obj_val.type_name().into();
        let key = (type_name.clone(), method.clone());
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          let result = ext_method(obj_val, args).map_err(VMError::RuntimeError)?;
          self.registers[dest] = result;
          Ok(())
        } else {
          Err(VMError::MethodNotFound {
            type_name: obj_val.type_name(),
            method: method.clone(),
          })
        }
      }
    }
  }

  fn dispatch_string_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &SmolStr,
    args: &[Value],
  ) -> Result<(), VMError> {
    let s = match &self.registers[obj] {
      Value::String(s) => s.clone(),
      _ => unreachable!(),
    };

    match method.as_str() {
      "upper" => {
        self.registers[dest] = Value::String(s.to_uppercase_smolstr());
      }
      "lower" => {
        self.registers[dest] = Value::String(s.to_lowercase_smolstr());
      }
      "trim" => {
        self.registers[dest] = Value::String(SmolStr::new(s.trim()));
      }
      "trimStart" => {
        self.registers[dest] = Value::String(SmolStr::new(s.trim_start()));
      }
      "trimEnd" => {
        self.registers[dest] = Value::String(SmolStr::new(s.trim_end()));
      }
      "split" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "split() requires a delimiter argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(delim) => {
            let parts: Vec<SmolStr> = s.split(delim.as_str()).map(SmolStr::new).collect();
            self.registers[dest] = Value::StringList(parts);
          }
          _ => {
            return Err(VMError::RuntimeError(
              "split() requires a string delimiter".to_string(),
            ));
          }
        }
      }
      "replace" => {
        if args.len() < 2 {
          return Err(VMError::RuntimeError(
            "replace() requires two arguments (old, new)".to_string(),
          ));
        }
        match (&args[0], &args[1]) {
          (Value::String(old), Value::String(new)) => {
            let result = SmolStr::new(s.replace(old.as_str(), new.as_str()));
            self.registers[dest] = Value::String(result);
          }
          _ => {
            return Err(VMError::RuntimeError(
              "replace() requires string arguments".to_string(),
            ));
          }
        }
      }
      "startsWith" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "startsWith() requires a prefix argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(prefix) => {
            self.registers[dest] = Value::Boolean(s.starts_with(prefix.as_str()));
          }
          _ => {
            return Err(VMError::RuntimeError(
              "startsWith() requires a string prefix".to_string(),
            ));
          }
        }
      }
      "endsWith" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "endsWith() requires a suffix argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(suffix) => {
            self.registers[dest] = Value::Boolean(s.ends_with(suffix.as_str()));
          }
          _ => {
            return Err(VMError::RuntimeError(
              "endsWith() requires a string suffix".to_string(),
            ));
          }
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "contains() requires a substring argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(substr) => {
            self.registers[dest] = Value::Boolean(s.contains(substr.as_str()));
          }
          _ => {
            return Err(VMError::RuntimeError(
              "contains() requires a string substring".to_string(),
            ));
          }
        }
      }
      "length" => {
        self.registers[dest] = Value::Number(Decimal::from(s.len()));
      }
      "charAt" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "charAt() requires an index argument".to_string(),
          ));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_u64().unwrap_or(u64::MAX) as usize;
            match s.chars().nth(index) {
              Some(c) => {
                self.registers[dest] = Value::String(SmolStr::new(c.to_string()));
              }
              None => {
                self.registers[dest] = Value::Null;
              }
            }
          }
          _ => {
            return Err(VMError::RuntimeError(
              "charAt() requires a number index".to_string(),
            ));
          }
        }
      }
      "substring" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "substring() requires at least a start index".to_string(),
          ));
        }
        match &args[0] {
          Value::Number(start_idx) => {
            let start = start_idx.to_usize().unwrap_or(0);
            let chars: Vec<char> = s.chars().collect();
            let end = if args.len() > 1 {
              match &args[1] {
                Value::Number(end_idx) => end_idx.to_usize().unwrap_or(chars.len()),
                _ => chars.len(),
              }
            } else {
              chars.len()
            };

            if start >= chars.len() || start >= end {
              self.registers[dest] = Value::String(SmolStr::new(""));
            } else {
              let end = end.min(chars.len());
              let result: String = chars[start..end].iter().collect();
              self.registers[dest] = Value::String(SmolStr::new(result));
            }
          }
          _ => {
            return Err(VMError::RuntimeError(
              "substring() requires a number start index".to_string(),
            ));
          }
        }
      }
      _ => {
        let key = (SmolStr::new_static("String"), method.clone());
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          let obj_val = &self.registers[obj];
          let result = ext_method(obj_val, args).map_err(VMError::RuntimeError)?;
          self.registers[dest] = result;
        } else {
          return Err(VMError::MethodNotFound {
            type_name: "String",
            method: method.clone(),
          });
        }
      }
    }
    Ok(())
  }

  fn dispatch_string_list_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &SmolStr,
    args: &[Value],
  ) -> Result<(), VMError> {
    let list = match &self.registers[obj] {
      Value::StringList(l) => l.clone(),
      _ => unreachable!(),
    };

    match method.as_str() {
      "length" | "len" => {
        self.registers[dest] = Value::Number(Decimal::from(list.len()));
      }
      "isEmpty" => {
        self.registers[dest] = Value::Boolean(list.is_empty());
      }
      "first" => {
        self.registers[dest] = list
          .first()
          .map(|s| Value::String(s.clone()))
          .unwrap_or(Value::Null);
      }
      "last" => {
        self.registers[dest] = list
          .last()
          .map(|s| Value::String(s.clone()))
          .unwrap_or(Value::Null);
      }
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("get() requires an index".to_string()));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_usize().unwrap_or(usize::MAX);
            self.registers[dest] = list
              .get(index)
              .map(|s| Value::String(s.clone()))
              .unwrap_or(Value::Null);
          }
          _ => return Err(VMError::RuntimeError("get() requires a number index".to_string())),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("contains() requires an argument".to_string()));
        }
        match &args[0] {
          Value::String(s) => {
            self.registers[dest] = Value::Boolean(list.contains(s));
          }
          _ => return Err(VMError::RuntimeError("contains() requires a string argument".to_string())),
        }
      }
      "indexOf" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("indexOf() requires an argument".to_string()));
        }
        match &args[0] {
          Value::String(s) => {
            let idx = list.iter().position(|item| item == s);
            self.registers[dest] = idx
              .map(|i| Value::Number(Decimal::from(i)))
              .unwrap_or(Value::Number(Decimal::from(-1)));
          }
          _ => return Err(VMError::RuntimeError("indexOf() requires a string argument".to_string())),
        }
      }
      "slice" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("slice() requires at least a start index".to_string()));
        }
        match &args[0] {
          Value::Number(start_idx) => {
            let start = start_idx.to_usize().unwrap_or(0).min(list.len());
            let end = if args.len() > 1 {
              match &args[1] {
                Value::Number(end_idx) => end_idx.to_usize().unwrap_or(list.len()).min(list.len()),
                _ => list.len(),
              }
            } else {
              list.len()
            };
            self.registers[dest] = Value::StringList(list[start..end].to_vec());
          }
          _ => return Err(VMError::RuntimeError("slice() requires a number index".to_string())),
        }
      }
      "reverse" => {
        let mut reversed = list;
        reversed.reverse();
        self.registers[dest] = Value::StringList(reversed);
      }
      "sort" => {
        let mut sorted = list;
        sorted.sort();
        self.registers[dest] = Value::StringList(sorted);
      }
      "join" => {
        let delim = if args.is_empty() {
          ""
        } else {
          match &args[0] {
            Value::String(s) => s.as_str(),
            _ => return Err(VMError::RuntimeError("join() requires a string delimiter".to_string())),
          }
        };
        let result: String = list.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(delim);
        self.registers[dest] = Value::String(SmolStr::new(result));
      }
      _ => {
        let key = (SmolStr::new_static("StringList"), method.clone());
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          let obj_val = &self.registers[obj];
          let result = ext_method(obj_val, args).map_err(VMError::RuntimeError)?;
          self.registers[dest] = result;
        } else {
          return Err(VMError::MethodNotFound {
            type_name: "StringList",
            method: method.clone(),
          });
        }
      }
    }
    Ok(())
  }

  fn dispatch_number_list_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &SmolStr,
    args: &[Value],
  ) -> Result<(), VMError> {
    let list = match &self.registers[obj] {
      Value::NumberList(l) => l.clone(),
      _ => unreachable!(),
    };

    match method.as_str() {
      "length" | "len" => {
        self.registers[dest] = Value::Number(Decimal::from(list.len()));
      }
      "isEmpty" => {
        self.registers[dest] = Value::Boolean(list.is_empty());
      }
      "first" => {
        self.registers[dest] = list
          .first()
          .map(|n| Value::Number(*n))
          .unwrap_or(Value::Null);
      }
      "last" => {
        self.registers[dest] = list
          .last()
          .map(|n| Value::Number(*n))
          .unwrap_or(Value::Null);
      }
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("get() requires an index".to_string()));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_usize().unwrap_or(usize::MAX);
            self.registers[dest] = list
              .get(index)
              .map(|n| Value::Number(*n))
              .unwrap_or(Value::Null);
          }
          _ => return Err(VMError::RuntimeError("get() requires a number index".to_string())),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("contains() requires an argument".to_string()));
        }
        match &args[0] {
          Value::Number(n) => {
            self.registers[dest] = Value::Boolean(list.contains(n));
          }
          _ => return Err(VMError::RuntimeError("contains() requires a number argument".to_string())),
        }
      }
      "indexOf" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("indexOf() requires an argument".to_string()));
        }
        match &args[0] {
          Value::Number(n) => {
            let idx = list.iter().position(|item| item == n);
            self.registers[dest] = idx
              .map(|i| Value::Number(Decimal::from(i)))
              .unwrap_or(Value::Number(Decimal::from(-1)));
          }
          _ => return Err(VMError::RuntimeError("indexOf() requires a number argument".to_string())),
        }
      }
      "slice" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("slice() requires at least a start index".to_string()));
        }
        match &args[0] {
          Value::Number(start_idx) => {
            let start = start_idx.to_usize().unwrap_or(0).min(list.len());
            let end = if args.len() > 1 {
              match &args[1] {
                Value::Number(end_idx) => end_idx.to_usize().unwrap_or(list.len()).min(list.len()),
                _ => list.len(),
              }
            } else {
              list.len()
            };
            self.registers[dest] = Value::NumberList(list[start..end].to_vec());
          }
          _ => return Err(VMError::RuntimeError("slice() requires a number index".to_string())),
        }
      }
      "reverse" => {
        let mut reversed = list;
        reversed.reverse();
        self.registers[dest] = Value::NumberList(reversed);
      }
      "sort" => {
        let mut sorted = list;
        sorted.sort();
        self.registers[dest] = Value::NumberList(sorted);
      }
      "sum" => {
        let sum: Decimal = list.iter().sum();
        self.registers[dest] = Value::Number(sum);
      }
      "avg" => {
        if list.is_empty() {
          self.registers[dest] = Value::Null;
        } else {
          let sum: Decimal = list.iter().sum();
          let avg = sum / Decimal::from(list.len());
          self.registers[dest] = Value::Number(avg);
        }
      }
      "min" => {
        self.registers[dest] = list
          .iter()
          .min()
          .map(|n| Value::Number(*n))
          .unwrap_or(Value::Null);
      }
      "max" => {
        self.registers[dest] = list
          .iter()
          .max()
          .map(|n| Value::Number(*n))
          .unwrap_or(Value::Null);
      }
      _ => {
        let key = (SmolStr::new_static("NumberList"), method.clone());
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          let obj_val = &self.registers[obj];
          let result = ext_method(obj_val, args).map_err(VMError::RuntimeError)?;
          self.registers[dest] = result;
        } else {
          return Err(VMError::MethodNotFound {
            type_name: "NumberList",
            method: method.clone(),
          });
        }
      }
    }
    Ok(())
  }

  fn dispatch_object_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &SmolStr,
    args: &[Value],
  ) -> Result<(), VMError> {
    let map = match &self.registers[obj] {
      Value::Object(m) => m.clone(),
      _ => unreachable!(),
    };

    match method.as_str() {
      "keys" => {
        let keys: Vec<SmolStr> = map.keys().cloned().collect();
        self.registers[dest] = Value::StringList(keys);
      }
      "values" => {
        let vals: Vec<Value> = map.values().cloned().collect();
        if vals.is_empty() {
          self.registers[dest] = Value::StringList(Vec::new());
        } else if vals.iter().all(|v| matches!(v, Value::String(_))) {
          let strings: Vec<SmolStr> = vals
            .into_iter()
            .map(|v| match v {
              Value::String(s) => s,
              _ => unreachable!(),
            })
            .collect();
          self.registers[dest] = Value::StringList(strings);
        } else if vals.iter().all(|v| matches!(v, Value::Number(_))) {
          let numbers: Vec<Decimal> = vals
            .into_iter()
            .map(|v| match v {
              Value::Number(n) => n,
              _ => unreachable!(),
            })
            .collect();
          self.registers[dest] = Value::NumberList(numbers);
        } else {
          return Err(VMError::RuntimeError(
            "values() only works when all values are the same type (String or Number)".to_string(),
          ));
        }
      }
      "length" | "len" => {
        self.registers[dest] = Value::Number(Decimal::from(map.len()));
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "contains() requires a key argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(key) => {
            self.registers[dest] = Value::Boolean(map.contains_key(key));
          }
          _ => {
            return Err(VMError::RuntimeError(
              "contains() requires a string key".to_string(),
            ))
          }
        }
      }
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "get() requires a key argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(key) => {
            self.registers[dest] = map.get(key).cloned().unwrap_or(Value::Null);
          }
          _ => {
            return Err(VMError::RuntimeError(
              "get() requires a string key".to_string(),
            ))
          }
        }
      }
      _ => {
        let key = (SmolStr::new_static("Object"), method.clone());
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          let obj_val = &self.registers[obj];
          let result = ext_method(obj_val, args).map_err(VMError::RuntimeError)?;
          self.registers[dest] = result;
        } else {
          return Err(VMError::MethodNotFound {
            type_name: "Object",
            method: method.clone(),
          });
        }
      }
    }
    Ok(())
  }
}
