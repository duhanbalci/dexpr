use crate::ast::value::Value;
use crate::opcodes::default_fn;
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};

use super::error::VMError;
use super::vm::VM;

impl<'a> VM<'a> {
  /// Dispatch a built-in (default) function call by ID
  pub(super) fn dispatch_builtin(
    &mut self,
    dest: usize,
    fn_id: u8,
    arg_regs: &[usize],
  ) -> Result<(), VMError> {
    match fn_id {
      default_fn::RAND => self.builtin_rand(dest, arg_regs),
      default_fn::ABS => self.builtin_abs(dest, arg_regs),
      default_fn::MIN => self.builtin_min(dest, arg_regs),
      default_fn::MAX => self.builtin_max(dest, arg_regs),
      default_fn::FLOOR => self.builtin_floor(dest, arg_regs),
      default_fn::CEIL => self.builtin_ceil(dest, arg_regs),
      default_fn::ROUND => self.builtin_round(dest, arg_regs),
      default_fn::SQRT => self.builtin_sqrt(dest, arg_regs),
      default_fn::LEN => self.builtin_len(dest, arg_regs),
      default_fn::TO_STRING => self.builtin_to_string(dest, arg_regs),
      default_fn::TO_NUMBER => self.builtin_to_number(dest, arg_regs),
      _ => {
        let name = default_fn::name(fn_id)
          .map(|s| s.to_string())
          .unwrap_or_else(|| format!("unknown({})", fn_id));
        Err(VMError::RuntimeError(format!(
          "Unknown default function: {}",
          name
        )))
      }
    }
  }

  fn builtin_rand(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    use rand::RngExt;
    if arg_regs.len() < 2 {
      return Err(VMError::RuntimeError(
        "rand() requires two arguments (min, max)".to_string(),
      ));
    }
    match (&self.registers[arg_regs[0]], &self.registers[arg_regs[1]]) {
      (Value::Number(min), Value::Number(max)) => {
        let min_i64 = min.to_i64().ok_or_else(|| {
          VMError::RuntimeError("rand() min must be an integer".to_string())
        })?;
        let max_i64 = max.to_i64().ok_or_else(|| {
          VMError::RuntimeError("rand() max must be an integer".to_string())
        })?;
        if min_i64 > max_i64 {
          return Err(VMError::RuntimeError(
            "rand() min must be <= max".to_string(),
          ));
        }
        let mut rng = rand::rng();
        let result = rng.random_range(min_i64..=max_i64);
        self.registers[dest] = Value::Number(Decimal::from(result));
        Ok(())
      }
      _ => Err(VMError::RuntimeError(
        "rand() requires number arguments".to_string(),
      )),
    }
  }

  fn builtin_abs(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("abs", arg_regs, 1)?;
    let n = extract_number("abs", &self.registers[arg_regs[0]])?;
    self.registers[dest] = Value::Number(n.abs());
    Ok(())
  }

  fn builtin_min(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    if arg_regs.is_empty() {
      return Err(VMError::RuntimeError(
        "min() requires at least one argument".to_string(),
      ));
    }
    let mut result = extract_number("min", &self.registers[arg_regs[0]])?;
    for &reg in &arg_regs[1..] {
      let n = extract_number("min", &self.registers[reg])?;
      if n < result {
        result = n;
      }
    }
    self.registers[dest] = Value::Number(result);
    Ok(())
  }

  fn builtin_max(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    if arg_regs.is_empty() {
      return Err(VMError::RuntimeError(
        "max() requires at least one argument".to_string(),
      ));
    }
    let mut result = extract_number("max", &self.registers[arg_regs[0]])?;
    for &reg in &arg_regs[1..] {
      let n = extract_number("max", &self.registers[reg])?;
      if n > result {
        result = n;
      }
    }
    self.registers[dest] = Value::Number(result);
    Ok(())
  }

  fn builtin_floor(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("floor", arg_regs, 1)?;
    let n = extract_number("floor", &self.registers[arg_regs[0]])?;
    self.registers[dest] = Value::Number(n.floor());
    Ok(())
  }

  fn builtin_ceil(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("ceil", arg_regs, 1)?;
    let n = extract_number("ceil", &self.registers[arg_regs[0]])?;
    self.registers[dest] = Value::Number(n.ceil());
    Ok(())
  }

  fn builtin_round(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("round", arg_regs, 1)?;
    let n = extract_number("round", &self.registers[arg_regs[0]])?;
    // Optional second argument: decimal places (default 0)
    let places = if arg_regs.len() > 1 {
      extract_number("round", &self.registers[arg_regs[1]])?
        .to_u32()
        .unwrap_or(0)
    } else {
      0
    };
    self.registers[dest] = Value::Number(n.round_dp(places));
    Ok(())
  }

  fn builtin_sqrt(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("sqrt", arg_regs, 1)?;
    let n = extract_number("sqrt", &self.registers[arg_regs[0]])?;
    if n.is_sign_negative() {
      return Err(VMError::RuntimeError(
        "sqrt() argument must be non-negative".to_string(),
      ));
    }
    let result = n.sqrt().ok_or_else(|| {
      VMError::RuntimeError("sqrt() failed to compute".to_string())
    })?;
    self.registers[dest] = Value::Number(result);
    Ok(())
  }

  fn builtin_len(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("len", arg_regs, 1)?;
    let len = match &self.registers[arg_regs[0]] {
      Value::String(s) => Decimal::from(s.len()),
      Value::StringList(l) => Decimal::from(l.len()),
      Value::NumberList(l) => Decimal::from(l.len()),
      Value::Object(m) => Decimal::from(m.len()),
      other => {
        return Err(VMError::RuntimeError(format!(
          "len() not supported for type {}",
          other.type_name()
        )));
      }
    };
    self.registers[dest] = Value::Number(len);
    Ok(())
  }

  fn builtin_to_string(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("toString", arg_regs, 1)?;
    let s = super::vm::value_to_string(&self.registers[arg_regs[0]]);
    self.registers[dest] = Value::String(s.into_owned().into());
    Ok(())
  }

  fn builtin_to_number(&mut self, dest: usize, arg_regs: &[usize]) -> Result<(), VMError> {
    require_args("toNumber", arg_regs, 1)?;
    let result = match &self.registers[arg_regs[0]] {
      Value::Number(n) => *n,
      Value::String(s) => s.parse::<Decimal>().map_err(|_| {
        VMError::RuntimeError(format!("toNumber() cannot parse '{}'", s))
      })?,
      Value::Boolean(b) => {
        if *b {
          Decimal::from(1)
        } else {
          Decimal::from(0)
        }
      }
      other => {
        return Err(VMError::RuntimeError(format!(
          "toNumber() not supported for type {}",
          other.type_name()
        )));
      }
    };
    self.registers[dest] = Value::Number(result);
    Ok(())
  }
}

/// Helper: check minimum argument count
fn require_args(name: &str, arg_regs: &[usize], min: usize) -> Result<(), VMError> {
  if arg_regs.len() < min {
    Err(VMError::RuntimeError(format!(
      "{}() requires at least {} argument(s)",
      name, min
    )))
  } else {
    Ok(())
  }
}

/// Helper: extract a Decimal from a Value or return an error
fn extract_number(name: &str, val: &Value) -> Result<Decimal, VMError> {
  match val {
    Value::Number(n) => Ok(*n),
    other => Err(VMError::RuntimeError(format!(
      "{}() requires a number argument, got {}",
      name,
      other.type_name()
    ))),
  }
}
