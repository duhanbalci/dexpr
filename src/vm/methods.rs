use crate::ast::value::Value;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use smol_str::{SmolStr, StrExt};
use std::rc::Rc;

use super::error::VMError;
use super::vm::VM;

impl<'a> VM<'a> {
  /// Dispatch a method call on a value.
  ///
  /// Uses `std::mem::take` to temporarily move the value out of the register,
  /// avoiding clones when dispatching to type-specific handlers.
  pub(super) fn dispatch_method(
    &mut self,
    dest: usize,
    obj: usize,
    method: &str,
    args: &[Value],
  ) -> Result<(), VMError> {
    // Take the value out to avoid borrow conflicts (register read + write).
    let obj_val = std::mem::take(&mut self.registers[obj]);

    let result = match &obj_val {
      Value::String(_) => self.dispatch_string_method_inner(&obj_val, method, args),
      Value::StringList(_) => self.dispatch_string_list_method_inner(&obj_val, method, args),
      Value::NumberList(_) => self.dispatch_number_list_method_inner(&obj_val, method, args),
      Value::Object(_) => self.dispatch_object_method_inner(&obj_val, method, args),
      Value::List(_) => self.dispatch_list_method_inner(&obj_val, method, args),
      _ => {
        // Try external methods for any type
        let type_name: SmolStr = obj_val.type_name().into();
        let key = (type_name, SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(&obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: obj_val.type_name(),
            method: SmolStr::from(method),
          })
        }
      }
    };

    // Put the object back, then set dest (if dest == obj, result overwrites it — that's fine).
    self.registers[obj] = obj_val;
    self.registers[dest] = result?;
    Ok(())
  }

  fn dispatch_string_method_inner(
    &self,
    obj_val: &Value,
    method: &str,
    args: &[Value],
  ) -> Result<Value, VMError> {
    let s = match obj_val {
      Value::String(s) => s,
      _ => unreachable!(),
    };

    match method {
      "upper" => Ok(Value::String(s.to_uppercase_smolstr())),
      "lower" => Ok(Value::String(s.to_lowercase_smolstr())),
      "trim" => Ok(Value::String(SmolStr::new(s.trim()))),
      "trimStart" => Ok(Value::String(SmolStr::new(s.trim_start()))),
      "trimEnd" => Ok(Value::String(SmolStr::new(s.trim_end()))),
      "split" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "split() requires a delimiter argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(delim) => {
            let parts: Vec<SmolStr> = s.split(delim.as_str()).map(SmolStr::new).collect();
            Ok(Value::StringList(Rc::new(parts)))
          }
          _ => Err(VMError::RuntimeError(
            "split() requires a string delimiter".to_string(),
          )),
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
            Ok(Value::String(SmolStr::new(s.replace(old.as_str(), new.as_str()))))
          }
          _ => Err(VMError::RuntimeError(
            "replace() requires string arguments".to_string(),
          )),
        }
      }
      "startsWith" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "startsWith() requires a prefix argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(prefix) => Ok(Value::Boolean(s.starts_with(prefix.as_str()))),
          _ => Err(VMError::RuntimeError(
            "startsWith() requires a string prefix".to_string(),
          )),
        }
      }
      "endsWith" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "endsWith() requires a suffix argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(suffix) => Ok(Value::Boolean(s.ends_with(suffix.as_str()))),
          _ => Err(VMError::RuntimeError(
            "endsWith() requires a string suffix".to_string(),
          )),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "contains() requires a substring argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(substr) => Ok(Value::Boolean(s.contains(substr.as_str()))),
          _ => Err(VMError::RuntimeError(
            "contains() requires a string substring".to_string(),
          )),
        }
      }
      "length" => Ok(Value::Number(Decimal::from(s.len()))),
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
              Some(c) => Ok(Value::String(SmolStr::new(c.to_string()))),
              None => Ok(Value::Null),
            }
          }
          _ => Err(VMError::RuntimeError(
            "charAt() requires a number index".to_string(),
          )),
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
              Ok(Value::String(SmolStr::new("")))
            } else {
              let end = end.min(chars.len());
              let result: String = chars[start..end].iter().collect();
              Ok(Value::String(SmolStr::new(result)))
            }
          }
          _ => Err(VMError::RuntimeError(
            "substring() requires a number start index".to_string(),
          )),
        }
      }
      _ => {
        let key = (SmolStr::new_static("String"), SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: "String",
            method: SmolStr::from(method),
          })
        }
      }
    }
  }

  fn dispatch_string_list_method_inner(
    &self,
    obj_val: &Value,
    method: &str,
    args: &[Value],
  ) -> Result<Value, VMError> {
    let list = match obj_val {
      Value::StringList(l) => l,
      _ => unreachable!(),
    };

    match method {
      "length" | "len" => Ok(Value::Number(Decimal::from(list.len()))),
      "isEmpty" => Ok(Value::Boolean(list.is_empty())),
      "first" => Ok(list.first().map(|s| Value::String(s.clone())).unwrap_or(Value::Null)),
      "last" => Ok(list.last().map(|s| Value::String(s.clone())).unwrap_or(Value::Null)),
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("get() requires an index".to_string()));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_usize().unwrap_or(usize::MAX);
            Ok(list.get(index).map(|s| Value::String(s.clone())).unwrap_or(Value::Null))
          }
          _ => Err(VMError::RuntimeError("get() requires a number index".to_string())),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("contains() requires an argument".to_string()));
        }
        match &args[0] {
          Value::String(s) => Ok(Value::Boolean(list.contains(s))),
          _ => Err(VMError::RuntimeError("contains() requires a string argument".to_string())),
        }
      }
      "indexOf" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("indexOf() requires an argument".to_string()));
        }
        match &args[0] {
          Value::String(s) => {
            let idx = list.iter().position(|item| item == s);
            Ok(idx.map(|i| Value::Number(Decimal::from(i))).unwrap_or(Value::Number(Decimal::from(-1))))
          }
          _ => Err(VMError::RuntimeError("indexOf() requires a string argument".to_string())),
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
            Ok(Value::StringList(Rc::new(list[start..end].to_vec())))
          }
          _ => Err(VMError::RuntimeError("slice() requires a number index".to_string())),
        }
      }
      "reverse" => {
        let mut reversed = list.to_vec();
        reversed.reverse();
        Ok(Value::StringList(Rc::new(reversed)))
      }
      "sort" => {
        let mut sorted = list.to_vec();
        sorted.sort();
        Ok(Value::StringList(Rc::new(sorted)))
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
        Ok(Value::String(SmolStr::new(result)))
      }
      _ => {
        let key = (SmolStr::new_static("StringList"), SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: "StringList",
            method: SmolStr::from(method),
          })
        }
      }
    }
  }

  fn dispatch_number_list_method_inner(
    &self,
    obj_val: &Value,
    method: &str,
    args: &[Value],
  ) -> Result<Value, VMError> {
    let list = match obj_val {
      Value::NumberList(l) => l,
      _ => unreachable!(),
    };

    match method {
      "length" | "len" => Ok(Value::Number(Decimal::from(list.len()))),
      "isEmpty" => Ok(Value::Boolean(list.is_empty())),
      "first" => Ok(list.first().map(|n| Value::Number(*n)).unwrap_or(Value::Null)),
      "last" => Ok(list.last().map(|n| Value::Number(*n)).unwrap_or(Value::Null)),
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("get() requires an index".to_string()));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_usize().unwrap_or(usize::MAX);
            Ok(list.get(index).map(|n| Value::Number(*n)).unwrap_or(Value::Null))
          }
          _ => Err(VMError::RuntimeError("get() requires a number index".to_string())),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("contains() requires an argument".to_string()));
        }
        match &args[0] {
          Value::Number(n) => Ok(Value::Boolean(list.contains(n))),
          _ => Err(VMError::RuntimeError("contains() requires a number argument".to_string())),
        }
      }
      "indexOf" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("indexOf() requires an argument".to_string()));
        }
        match &args[0] {
          Value::Number(n) => {
            let idx = list.iter().position(|item| item == n);
            Ok(idx.map(|i| Value::Number(Decimal::from(i))).unwrap_or(Value::Number(Decimal::from(-1))))
          }
          _ => Err(VMError::RuntimeError("indexOf() requires a number argument".to_string())),
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
            Ok(Value::NumberList(Rc::new(list[start..end].to_vec())))
          }
          _ => Err(VMError::RuntimeError("slice() requires a number index".to_string())),
        }
      }
      "reverse" => {
        let mut reversed = list.to_vec();
        reversed.reverse();
        Ok(Value::NumberList(Rc::new(reversed)))
      }
      "sort" => {
        let mut sorted = list.to_vec();
        sorted.sort();
        Ok(Value::NumberList(Rc::new(sorted)))
      }
      "sum" => {
        let sum: Decimal = list.iter().sum();
        Ok(Value::Number(sum))
      }
      "avg" => {
        if list.is_empty() {
          Ok(Value::Null)
        } else {
          let sum: Decimal = list.iter().sum();
          let avg = sum / Decimal::from(list.len());
          Ok(Value::Number(avg))
        }
      }
      "min" => Ok(list.iter().min().map(|n| Value::Number(*n)).unwrap_or(Value::Null)),
      "max" => Ok(list.iter().max().map(|n| Value::Number(*n)).unwrap_or(Value::Null)),
      _ => {
        let key = (SmolStr::new_static("NumberList"), SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: "NumberList",
            method: SmolStr::from(method),
          })
        }
      }
    }
  }

  fn dispatch_object_method_inner(
    &self,
    obj_val: &Value,
    method: &str,
    args: &[Value],
  ) -> Result<Value, VMError> {
    let map = match obj_val {
      Value::Object(m) => m,
      _ => unreachable!(),
    };

    match method {
      "keys" => {
        let keys: Vec<SmolStr> = map.keys().cloned().collect();
        Ok(Value::StringList(Rc::new(keys)))
      }
      "values" => {
        let vals: Vec<Value> = map.values().cloned().collect();
        if vals.is_empty() {
          Ok(Value::StringList(Rc::new(Vec::new())))
        } else if vals.iter().all(|v| matches!(v, Value::String(_))) {
          let strings: Vec<SmolStr> = vals
            .into_iter()
            .map(|v| match v {
              Value::String(s) => s,
              _ => unreachable!(),
            })
            .collect();
          Ok(Value::StringList(Rc::new(strings)))
        } else if vals.iter().all(|v| matches!(v, Value::Number(_))) {
          let numbers: Vec<Decimal> = vals
            .into_iter()
            .map(|v| match v {
              Value::Number(n) => n,
              _ => unreachable!(),
            })
            .collect();
          Ok(Value::NumberList(Rc::new(numbers)))
        } else {
          Ok(Value::List(Rc::new(vals)))
        }
      }
      "length" | "len" => Ok(Value::Number(Decimal::from(map.len()))),
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "contains() requires a key argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(key) => Ok(Value::Boolean(map.contains_key(key))),
          _ => Err(VMError::RuntimeError(
            "contains() requires a string key".to_string(),
          )),
        }
      }
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError(
            "get() requires a key argument".to_string(),
          ));
        }
        match &args[0] {
          Value::String(key) => Ok(map.get(key).cloned().unwrap_or(Value::Null)),
          _ => Err(VMError::RuntimeError(
            "get() requires a string key".to_string(),
          )),
        }
      }
      _ => {
        let key = (SmolStr::new_static("Object"), SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: "Object",
            method: SmolStr::from(method),
          })
        }
      }
    }
  }

  fn dispatch_list_method_inner(
    &self,
    obj_val: &Value,
    method: &str,
    args: &[Value],
  ) -> Result<Value, VMError> {
    let list = match obj_val {
      Value::List(l) => l,
      _ => unreachable!(),
    };

    match method {
      "length" | "len" => Ok(Value::Number(Decimal::from(list.len()))),
      "isEmpty" => Ok(Value::Boolean(list.is_empty())),
      "first" => Ok(list.first().cloned().unwrap_or(Value::Null)),
      "last" => Ok(list.last().cloned().unwrap_or(Value::Null)),
      "get" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("get() requires an index".to_string()));
        }
        match &args[0] {
          Value::Number(idx) => {
            let index = idx.to_usize().unwrap_or(usize::MAX);
            Ok(list.get(index).cloned().unwrap_or(Value::Null))
          }
          _ => Err(VMError::RuntimeError("get() requires a number index".to_string())),
        }
      }
      "contains" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("contains() requires an argument".to_string()));
        }
        Ok(Value::Boolean(list.contains(&args[0])))
      }
      "indexOf" => {
        if args.is_empty() {
          return Err(VMError::RuntimeError("indexOf() requires an argument".to_string()));
        }
        let idx = list.iter().position(|item| item == &args[0]);
        Ok(idx.map(|i| Value::Number(Decimal::from(i))).unwrap_or(Value::Number(Decimal::from(-1))))
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
            Ok(Value::List(Rc::new(list[start..end].to_vec())))
          }
          _ => Err(VMError::RuntimeError("slice() requires a number index".to_string())),
        }
      }
      "reverse" => {
        let mut reversed = list.to_vec();
        reversed.reverse();
        Ok(Value::List(Rc::new(reversed)))
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
        let strings: Vec<String> = list.iter().map(|v| format!("{}", v)).collect();
        Ok(Value::String(SmolStr::new(strings.join(delim))))
      }
      "map" => {
        // Property shorthand: list.map("fieldName") extracts a field from each Object element
        if args.is_empty() {
          return Err(VMError::RuntimeError("map() requires a field name argument".to_string()));
        }
        match &args[0] {
          Value::String(field) => {
            let mut values: Vec<Value> = Vec::with_capacity(list.len());
            for item in list.iter() {
              match item {
                Value::Object(map) => {
                  values.push(map.get(field.as_str()).cloned().unwrap_or(Value::Null));
                }
                _ => {
                  return Err(VMError::RuntimeError(format!(
                    "map() requires all elements to be Objects, got {}",
                    item.type_name()
                  )));
                }
              }
            }
            // Smart return type: if all values are the same primitive type, return typed list
            if values.is_empty() {
              return Ok(Value::List(Rc::new(values)));
            }
            if values.iter().all(|v| matches!(v, Value::Number(_))) {
              let nums: Vec<Decimal> = values
                .into_iter()
                .map(|v| match v {
                  Value::Number(n) => n,
                  _ => unreachable!(),
                })
                .collect();
              Ok(Value::NumberList(Rc::new(nums)))
            } else if values.iter().all(|v| matches!(v, Value::String(_))) {
              let strings: Vec<SmolStr> = values
                .into_iter()
                .map(|v| match v {
                  Value::String(s) => s,
                  _ => unreachable!(),
                })
                .collect();
              Ok(Value::StringList(Rc::new(strings)))
            } else {
              Ok(Value::List(Rc::new(values)))
            }
          }
          _ => Err(VMError::RuntimeError("map() requires a string field name".to_string())),
        }
      }
      "filter" => {
        // Property shorthand:
        //   list.filter("active")           — filter by truthy boolean field
        //   list.filter("field", value)     — filter where field == value
        if args.is_empty() {
          return Err(VMError::RuntimeError("filter() requires a field name argument".to_string()));
        }
        match &args[0] {
          Value::String(field) => {
            let filtered: Result<Vec<Value>, VMError> = list
              .iter()
              .filter_map(|item| {
                match item {
                  Value::Object(map) => {
                    let field_val = map.get(field.as_str()).cloned().unwrap_or(Value::Null);
                    if args.len() > 1 {
                      // filter("field", value) — equality check
                      if field_val == args[1] {
                        Some(Ok(item.clone()))
                      } else {
                        None
                      }
                    } else {
                      // filter("field") — truthy check
                      match &field_val {
                        Value::Boolean(b) => if *b { Some(Ok(item.clone())) } else { None },
                        Value::Null => None,
                        _ => Some(Ok(item.clone())),
                      }
                    }
                  }
                  _ => Some(Err(VMError::RuntimeError(format!(
                    "filter() requires all elements to be Objects, got {}",
                    item.type_name()
                  )))),
                }
              })
              .collect();
            Ok(Value::List(Rc::new(filtered?)))
          }
          _ => Err(VMError::RuntimeError("filter() requires a string field name".to_string())),
        }
      }
      "find" => {
        // Property shorthand: list.find("field", value) — first element where field == value
        if args.is_empty() {
          return Err(VMError::RuntimeError("find() requires a field name argument".to_string()));
        }
        match &args[0] {
          Value::String(field) => {
            for item in list.iter() {
              match item {
                Value::Object(map) => {
                  let field_val = map.get(field.as_str()).cloned().unwrap_or(Value::Null);
                  if args.len() > 1 {
                    if field_val == args[1] {
                      return Ok(item.clone());
                    }
                  } else {
                    // find("field") — first with truthy field
                    match &field_val {
                      Value::Boolean(b) => if *b { return Ok(item.clone()); },
                      Value::Null => {},
                      _ => return Ok(item.clone()),
                    }
                  }
                }
                _ => {
                  return Err(VMError::RuntimeError(format!(
                    "find() requires all elements to be Objects, got {}",
                    item.type_name()
                  )));
                }
              }
            }
            Ok(Value::Null)
          }
          _ => Err(VMError::RuntimeError("find() requires a string field name".to_string())),
        }
      }
      "sort" => {
        // Property shorthand: list.sort("field") — sort by field value
        if args.is_empty() {
          return Err(VMError::RuntimeError("sort() on List requires a field name argument".to_string()));
        }
        match &args[0] {
          Value::String(field) => {
            let mut sorted = list.to_vec();
            sorted.sort_by(|a, b| {
              let a_val = match a {
                Value::Object(map) => map.get(field.as_str()).cloned().unwrap_or(Value::Null),
                _ => Value::Null,
              };
              let b_val = match b {
                Value::Object(map) => map.get(field.as_str()).cloned().unwrap_or(Value::Null),
                _ => Value::Null,
              };
              match (&a_val, &b_val) {
                (Value::Number(a), Value::Number(b)) => a.cmp(b),
                (Value::String(a), Value::String(b)) => a.cmp(b),
                _ => std::cmp::Ordering::Equal,
              }
            });
            Ok(Value::List(Rc::new(sorted)))
          }
          _ => Err(VMError::RuntimeError("sort() requires a string field name".to_string())),
        }
      }
      _ => {
        let key = (SmolStr::new_static("List"), SmolStr::from(method));
        if let Some(ext_method) = self.external_methods.as_ref().and_then(|m| m.get(&key)) {
          ext_method(obj_val, args).map_err(VMError::RuntimeError)
        } else {
          Err(VMError::MethodNotFound {
            type_name: "List",
            method: SmolStr::from(method),
          })
        }
      }
    }
  }
}
