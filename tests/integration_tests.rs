use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};
use indexmap::IndexMap;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use smol_str::SmolStr;

/// Helper to run code and get the value of "result" variable
fn run_and_get_result(code: &str) -> Value {
  let ast = parser::program(code).expect("Failed to parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("Failed to compile");
  let mut vm = VM::new(&bytecode);
  vm.execute().expect("Failed to execute");
  vm.get_global("result").expect("result not found").clone()
}

/// Helper to run code and return the expression result (last expression value)
fn run_expr(code: &str) -> Value {
  let ast = parser::program(code).expect("Failed to parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("Failed to compile");
  let mut vm = VM::new(&bytecode);
  vm.execute().expect("Failed to execute")
}

/// Helper to run code with globals and get "result" variable
fn run_and_get_result_with_globals(code: &str, globals: Vec<(&str, Value)>) -> Value {
  let ast = parser::program(code).expect("Failed to parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("Failed to compile");
  let mut vm = VM::new(&bytecode);
  for (name, value) in globals {
    vm.set_global(name, value);
  }
  vm.execute().expect("Failed to execute");
  vm.get_global("result").expect("result not found").clone()
}

/// Helper to run code with globals and return expression result
fn run_expr_with_globals(code: &str, globals: Vec<(&str, Value)>) -> Value {
  let ast = parser::program(code).expect("Failed to parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("Failed to compile");
  let mut vm = VM::new(&bytecode);
  for (name, value) in globals {
    vm.set_global(name, value);
  }
  vm.execute().expect("Failed to execute")
}

// ==================== MODULO OPERATOR TESTS ====================

#[test]
fn test_modulo_basic() {
  let result = run_and_get_result("result = 10 % 3");
  assert_eq!(result, Value::Number(dec!(1)));
}

#[test]
fn test_modulo_zero_remainder() {
  let result = run_and_get_result("result = 12 % 4");
  assert_eq!(result, Value::Number(dec!(0)));
}

#[test]
fn test_modulo_with_decimals() {
  let result = run_and_get_result("result = 10.5 % 3");
  assert_eq!(result, Value::Number(dec!(1.5)));
}

#[test]
fn test_modulo_negative() {
  // Rust remainder semantics: -7 % 3 = -1
  let result = run_and_get_result("result = -7 % 3");
  assert_eq!(result, Value::Number(dec!(-1)));
}

#[test]
fn test_modulo_precedence() {
  // % should have same precedence as * and /
  // 10 + 7 % 3 = 10 + 1 = 11
  let result = run_and_get_result("result = 10 + 7 % 3");
  assert_eq!(result, Value::Number(dec!(11)));
}

// ==================== POWER OPERATOR TESTS ====================

#[test]
fn test_power_basic() {
  let result = run_and_get_result("result = 2 ** 10");
  assert_eq!(result, Value::Number(dec!(1024)));
}

#[test]
fn test_power_square() {
  let result = run_and_get_result("result = 5 ** 2");
  assert_eq!(result, Value::Number(dec!(25)));
}

#[test]
fn test_power_cube() {
  let result = run_and_get_result("result = 3 ** 3");
  assert_eq!(result, Value::Number(dec!(27)));
}

#[test]
fn test_power_zero_exponent() {
  let result = run_and_get_result("result = 5 ** 0");
  assert_eq!(result, Value::Number(dec!(1)));
}

#[test]
fn test_power_one_exponent() {
  let result = run_and_get_result("result = 7 ** 1");
  assert_eq!(result, Value::Number(dec!(7)));
}

#[test]
fn test_power_right_associative() {
  // 2 ** 3 ** 2 should be 2 ** (3 ** 2) = 2 ** 9 = 512 (right-associative)
  let result = run_and_get_result("result = 2 ** 3 ** 2");
  assert_eq!(result, Value::Number(dec!(512)));
}

#[test]
fn test_power_precedence() {
  // ** should have higher precedence than * and /
  // 2 * 3 ** 2 = 2 * 9 = 18
  let result = run_and_get_result("result = 2 * 3 ** 2");
  assert_eq!(result, Value::Number(dec!(18)));
}

#[test]
fn test_power_in_expression() {
  // (2 + 3) ** 2 = 5 ** 2 = 25
  let result = run_and_get_result("result = (2 + 3) ** 2");
  assert_eq!(result, Value::Number(dec!(25)));
}

// ==================== COMBINED OPERATOR TESTS ====================

#[test]
fn test_modulo_and_power_together() {
  // 2 ** 4 % 5 = 16 % 5 = 1
  let result = run_and_get_result("result = 2 ** 4 % 5");
  assert_eq!(result, Value::Number(dec!(1)));
}

#[test]
fn test_complex_expression_with_new_ops() {
  // (3 ** 2 + 4 ** 2) % 10 = (9 + 16) % 10 = 25 % 10 = 5
  let result = run_and_get_result("result = (3 ** 2 + 4 ** 2) % 10");
  assert_eq!(result, Value::Number(dec!(5)));
}

// ==================== COMPOUND ASSIGNMENT TESTS ====================

#[test]
fn test_compound_add() {
  let result = run_and_get_result("x = 10\nx += 5\nresult = x");
  assert_eq!(result, Value::Number(dec!(15)));
}

#[test]
fn test_compound_sub() {
  let result = run_and_get_result("x = 20\nx -= 7\nresult = x");
  assert_eq!(result, Value::Number(dec!(13)));
}

#[test]
fn test_compound_mul() {
  let result = run_and_get_result("x = 3\nx *= 4\nresult = x");
  assert_eq!(result, Value::Number(dec!(12)));
}

#[test]
fn test_compound_div() {
  let result = run_and_get_result("x = 100\nx /= 4\nresult = x");
  assert_eq!(result, Value::Number(dec!(25)));
}

#[test]
fn test_compound_mod() {
  let result = run_and_get_result("x = 17\nx %= 5\nresult = x");
  assert_eq!(result, Value::Number(dec!(2)));
}

#[test]
fn test_compound_multiple_operations() {
  let result = run_and_get_result(
    "x = 10\nx += 5\nx *= 2\nx -= 10\nresult = x",
  );
  assert_eq!(result, Value::Number(dec!(20)));
}

#[test]
fn test_compound_with_expression() {
  let result = run_and_get_result("x = 10\nx += 3 * 2\nresult = x");
  assert_eq!(result, Value::Number(dec!(16)));
}

#[test]
fn test_compound_chained() {
  let result = run_and_get_result(
    "a = 5\nb = 10\na += 1\nb -= 2\nresult = a + b",
  );
  assert_eq!(result, Value::Number(dec!(14)));
}

// ==================== STRING METHOD TESTS ====================

#[test]
fn test_string_trim() {
  let result = run_and_get_result(r#"result = "  hello  ".trim()"#);
  assert_eq!(result, Value::String("hello".into()));
}

#[test]
fn test_string_trim_start() {
  let result = run_and_get_result(r#"result = "  hello  ".trimStart()"#);
  assert_eq!(result, Value::String("hello  ".into()));
}

#[test]
fn test_string_trim_end() {
  let result = run_and_get_result(r#"result = "  hello  ".trimEnd()"#);
  assert_eq!(result, Value::String("  hello".into()));
}

#[test]
fn test_string_upper() {
  let result = run_and_get_result(r#"result = "hello".upper()"#);
  assert_eq!(result, Value::String("HELLO".into()));
}

#[test]
fn test_string_lower() {
  let result = run_and_get_result(r#"result = "HELLO".lower()"#);
  assert_eq!(result, Value::String("hello".into()));
}

#[test]
fn test_string_replace() {
  let result = run_and_get_result(r#"result = "hello".replace("l", "x")"#);
  assert_eq!(result, Value::String("hexxo".into()));
}

#[test]
fn test_string_contains() {
  let result = run_and_get_result(r#"result = "hello".contains("ell")"#);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_string_contains_false() {
  let result = run_and_get_result(r#"result = "hello".contains("xyz")"#);
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_starts_with() {
  let result = run_and_get_result(r#"result = "hello".startsWith("hel")"#);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_string_ends_with() {
  let result = run_and_get_result(r#"result = "hello".endsWith("llo")"#);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_string_length() {
  let result = run_and_get_result(r#"result = "hello".length()"#);
  assert_eq!(result, Value::Number(dec!(5)));
}

#[test]
fn test_string_char_at() {
  let result = run_and_get_result(r#"result = "hello".charAt(1)"#);
  assert_eq!(result, Value::String("e".into()));
}

#[test]
fn test_string_char_at_out_of_bounds() {
  let result = run_and_get_result(r#"result = "hello".charAt(10)"#);
  assert_eq!(result, Value::Null);
}

#[test]
fn test_string_substring() {
  let result = run_and_get_result(r#"result = "hello".substring(1, 4)"#);
  assert_eq!(result, Value::String("ell".into()));
}

#[test]
fn test_string_substring_to_end() {
  let result = run_and_get_result(r#"result = "hello".substring(2)"#);
  assert_eq!(result, Value::String("llo".into()));
}

#[test]
fn test_string_split() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.length()
"#);
  assert_eq!(result, Value::Number(dec!(3)));
}

// ==================== STRING LIST METHOD TESTS ====================

#[test]
fn test_stringlist_first() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.first()
"#);
  assert_eq!(result, Value::String("a".into()));
}

#[test]
fn test_stringlist_last() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.last()
"#);
  assert_eq!(result, Value::String("c".into()));
}

#[test]
fn test_stringlist_get() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.get(1)
"#);
  assert_eq!(result, Value::String("b".into()));
}

#[test]
fn test_stringlist_get_out_of_bounds() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.get(10)
"#);
  assert_eq!(result, Value::Null);
}

#[test]
fn test_stringlist_join() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.join("-")
"#);
  assert_eq!(result, Value::String("a-b-c".into()));
}

#[test]
fn test_stringlist_join_no_delimiter() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.join()
"#);
  assert_eq!(result, Value::String("abc".into()));
}
// ==================== EXPRESSION RETURN VALUE TESTS ====================

#[test]
fn test_expr_return_single_expression() {
  // Single-line expression should return its value
  let result = run_expr("42 + 8");
  assert_eq!(result, Value::Number(dec!(50)));
}

#[test]
fn test_expr_return_formula() {
  // Formula-style expression
  let result = run_expr_with_globals(
    "toplamTutar * kdv / 100",
    vec![
      ("toplamTutar", Value::Number(dec!(1000))),
      ("kdv", Value::Number(dec!(18))),
    ],
  );
  assert_eq!(result, Value::Number(dec!(180)));
}

#[test]
fn test_expr_return_last_expression() {
  // Multi-line: last expression statement wins
  let result = run_expr("x = 10\ny = 20\nx + y");
  assert_eq!(result, Value::Number(dec!(30)));
}

#[test]
fn test_expr_return_assignment_then_expr() {
  let result = run_expr("result = 42\nresult");
  assert_eq!(result, Value::Number(dec!(42)));
}

#[test]
fn test_expr_return_null_when_only_assignments() {
  // Only assignments, no expression statement → Null
  let result = run_expr("x = 10\ny = 20");
  assert_eq!(result, Value::Null);
}

#[test]
fn test_expr_return_string() {
  let result = run_expr("\"hello\" + \" world\"");
  assert_eq!(result, Value::String("hello world".into()));
}

#[test]
fn test_expr_return_boolean() {
  let result = run_expr("10 > 5");
  assert_eq!(result, Value::Boolean(true));
}

// ==================== EXTERNAL FUNCTION TESTS ====================

#[test]
fn test_external_function_basic() {
  let ast = parser::program("result = add(3, 4)").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.register_function("add", |args| {
    match (&args[0], &args[1]) {
      (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
      _ => Err("expected numbers".to_string()),
    }
  });
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Number(dec!(7)));
}

#[test]
fn test_external_function_no_args() {
  let ast = parser::program("pi()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.register_function("pi", |_args| {
    Ok(Value::Number(dec!(3.14159)))
  });
  let result = vm.execute().expect("execute");
  assert_eq!(result, Value::Number(dec!(3.14159)));
}

#[test]
fn test_external_function_with_globals() {
  let code = "getExchangeRate('USD')";
  let ast = parser::program(code).expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.register_function("getExchangeRate", |args| {
    match &args[0] {
      Value::String(currency) if currency.as_str() == "USD" => Ok(Value::Number(dec!(34.5))),
      Value::String(currency) if currency.as_str() == "EUR" => Ok(Value::Number(dec!(37.2))),
      _ => Err("unknown currency".to_string()),
    }
  });
  let result = vm.execute().expect("execute");
  assert_eq!(result, Value::Number(dec!(34.5)));
}

#[test]
fn test_external_function_undefined_error() {
  let ast = parser::program("unknownFn()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  let err = vm.execute().unwrap_err();
  assert!(err.to_string().contains("Undefined function: unknownFn"));
}

#[test]
fn test_external_function_in_formula() {
  // Realistic pricing formula: base * exchangeRate * (1 + commission/100)
  let code = "rate = getRate('USD')\nbase * rate * (1 + commission / 100)";
  let ast = parser::program(code).expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("base", Value::Number(dec!(100)));
  vm.set_global("commission", Value::Number(dec!(5)));
  vm.register_function("getRate", |args| {
    match &args[0] {
      Value::String(s) if s.as_str() == "USD" => Ok(Value::Number(dec!(34))),
      _ => Err("unknown".to_string()),
    }
  });
  let result = vm.execute().expect("execute");
  assert_eq!(result, Value::Number(dec!(3570)));
}

// ==================== IN KEYWORD TESTS ====================

#[test]
fn test_in_string_in_stringlist() {
  let result = run_and_get_result(r#"
categories = "finans,teknoloji,saglik".split(",")
result = "finans" in categories
"#);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_in_string_not_in_stringlist() {
  let result = run_and_get_result(r#"
categories = "finans,teknoloji,saglik".split(",")
result = "spor" in categories
"#);
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_in_string_in_string_substring() {
  let result = run_expr("\"hello\" in \"hello world\"");
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_in_string_not_in_string() {
  let result = run_expr("\"xyz\" in \"hello world\"");
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_in_with_if_statement() {
  let result = run_and_get_result(r#"
categories = "finans,teknoloji".split(",")
if "finans" in categories then
  result = "found"
else
  result = "not found"
end
"#);
  assert_eq!(result, Value::String("found".into()));
}

// ==================== LIST METHOD EXPANSION TESTS ====================

// StringList methods

#[test]
fn test_stringlist_contains() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.contains("b")
"#);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_stringlist_contains_missing() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.contains("z")
"#);
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_stringlist_indexof() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.indexOf("b")
"#);
  assert_eq!(result, Value::Number(dec!(1)));
}

#[test]
fn test_stringlist_indexof_not_found() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
result = parts.indexOf("z")
"#);
  assert_eq!(result, Value::Number(dec!(-1)));
}

#[test]
fn test_stringlist_slice() {
  let result = run_and_get_result(r#"
parts = "a,b,c,d".split(",")
sliced = parts.slice(1, 3)
result = sliced.join(",")
"#);
  assert_eq!(result, Value::String("b,c".into()));
}

#[test]
fn test_stringlist_reverse() {
  let result = run_and_get_result(r#"
parts = "a,b,c".split(",")
reversed = parts.reverse()
result = reversed.join(",")
"#);
  assert_eq!(result, Value::String("c,b,a".into()));
}

#[test]
fn test_stringlist_sort() {
  let result = run_and_get_result(r#"
parts = "c,a,b".split(",")
sorted = parts.sort()
result = sorted.join(",")
"#);
  assert_eq!(result, Value::String("a,b,c".into()));
}

#[test]
fn test_stringlist_isempty() {
  let result = run_and_get_result(r#"
parts = "a,b".split(",")
result = parts.isEmpty()
"#);
  assert_eq!(result, Value::Boolean(false));
}

// NumberList methods (using globals since we can't create NumberList literals yet)

#[test]
fn test_numberlist_contains() {
  let ast = parser::program("result = nums.contains(3)").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![dec!(1), dec!(2), dec!(3), dec!(4)]));
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Boolean(true));
}

#[test]
fn test_numberlist_indexof() {
  let ast = parser::program("result = nums.indexOf(3)").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![dec!(1), dec!(2), dec!(3)]));
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Number(dec!(2)));
}

#[test]
fn test_numberlist_slice() {
  let ast = parser::program("sliced = nums.slice(1, 3)\nresult = sliced.sum()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![dec!(10), dec!(20), dec!(30), dec!(40)]));
  vm.execute().expect("execute");
  // slice(1,3) = [20, 30], sum = 50
  assert_eq!(vm.get_global("result").unwrap(), &Value::Number(dec!(50)));
}

#[test]
fn test_numberlist_reverse() {
  let ast = parser::program("reversed = nums.reverse()\nresult = reversed.first()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![dec!(1), dec!(2), dec!(3)]));
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Number(dec!(3)));
}

#[test]
fn test_numberlist_sort() {
  let ast = parser::program("sorted = nums.sort()\nresult = sorted.first()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![dec!(30), dec!(10), dec!(20)]));
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Number(dec!(10)));
}

#[test]
fn test_numberlist_isempty() {
  let ast = parser::program("result = nums.isEmpty()").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("nums", Value::NumberList(vec![]));
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::Boolean(true));
}

// ==================== EXTERNAL METHOD TESTS ====================

#[test]
fn test_external_method_on_number() {
  let ast = parser::program("result = x.format(2)").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.set_global("x", Value::Number(dec!(3.14159)));
  vm.register_method("Number", "format", |this, args| {
    match (this, &args[0]) {
      (Value::Number(n), Value::Number(decimals)) => {
        let d = decimals.to_u32().unwrap_or(2);
        let formatted = format!("{:.1$}", n, d as usize);
        Ok(Value::String(formatted.into()))
      }
      _ => Err("format requires a number".to_string()),
    }
  });
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::String("3.14".into()));
}

#[test]
fn test_external_method_on_string() {
  let ast = parser::program(r#"result = "hello".repeat(3)"#).expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");
  let mut vm = VM::new(&bytecode);
  vm.register_method("String", "repeat", |this, args| {
    match (this, &args[0]) {
      (Value::String(s), Value::Number(n)) => {
        let count = n.to_usize().unwrap_or(0);
        Ok(Value::String(s.repeat(count).into()))
      }
      _ => Err("repeat requires a number".to_string()),
    }
  });
  vm.execute().expect("execute");
  assert_eq!(vm.get_global("result").unwrap(), &Value::String("hellohellohello".into()));
}

// ==================== RAND FUNCTION TESTS ====================

#[test]
fn test_rand_returns_number_in_range() {
  let result = run_expr("rand(1, 10)");
  match result {
    Value::Number(n) => {
      assert!(n >= dec!(1) && n <= dec!(10), "rand result {} not in range 1..10", n);
    }
    _ => panic!("Expected Number, got {:?}", result),
  }
}

#[test]
fn test_rand_same_min_max() {
  let result = run_expr("rand(5, 5)");
  assert_eq!(result, Value::Number(dec!(5)));
}

#[test]
fn test_rand_in_assignment() {
  let result = run_and_get_result("result = rand(1, 100)");
  match result {
    Value::Number(n) => {
      assert!(n >= dec!(1) && n <= dec!(100));
    }
    _ => panic!("Expected Number"),
  }
}

// ==================== BYTECODE CACHE & RE-EXECUTE TESTS ====================

#[test]
fn test_bytecode_reuse_with_different_globals() {
  // Compile once, execute multiple times with different inputs
  let ast = parser::program("base * rate").expect("parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("compile");

  // First execution
  let mut vm = VM::new(&bytecode);
  vm.set_global("base", Value::Number(dec!(100)));
  vm.set_global("rate", Value::Number(dec!(1.5)));
  let result1 = vm.execute().expect("execute");
  assert_eq!(result1, Value::Number(dec!(150.0)));

  // Second execution with different values — reuse same bytecode
  let mut vm2 = VM::new(&bytecode);
  vm2.set_global("base", Value::Number(dec!(200)));
  vm2.set_global("rate", Value::Number(dec!(2.0)));
  let result2 = vm2.execute().expect("execute");
  assert_eq!(result2, Value::Number(dec!(400.0)));
}

/*
#[test]
fn expr_test() {
  let cases = vec![
      ("(18.49730)/81.6*63.0852-64.773/(32.2007)+41.2879-85.4-(13.50-56.0243)-(49.35612)+(59.8+63.30426)/(70.56281)-13.81690*(86.7057*(43.8*(86.41762)+72.6*58.35/28.62060))", "-4711911.631973682999201581558386"),
("(52.01054+91.492+69.997)*(60.42597)+20.1237/43.9-(47.94052)+82.54171*48.5715+61.48", "16924.0893449520574032"),
("(21.10121)+(21.4)+48.18/(11.31875)+(71.4+21.05*(41.412)-(64.6949)/44.1296/(24.184/90.3442*49.8813)+(46.02969)*(22.6)/(77.3720/(88.8365/55.5577)))", "1011.2692544086353375"),
("(11.204)*5.3+17.95658-40.86/0.93-91.2*58.71*(61.36106/81.8-(45.24309)*(14.735/76.41129))", "42731.423060682431273967518192"),
("(71.0626*45.04296)*(1.38)+(24.6055)-(95.7317)-(17.14)/(92.799)/(6.00*42.7395/81.6413)/(18.65)-39.71747-33.2061", "4273.1474690724873717"),
("(40.294)*(42.0572)+(6.6897-53.21*11.12257)/(38.5)-(81.02849)+(58.49)-10.5+81.53/(50.3951*(18.03091)/(30.81381)+(13.80113)*(88.8/(76.5940)*38.12896)-(66.044)+(56.9/64.7))", "1646.5577647573549686"),
("42.4322*76.024*57.70-79.228-(83.6/11.815)+(0.66)+(19.205+91.8077/39.740*(62.10317+60.054+41.0))", "186442.931938706969977122025"),
("76.98*(89.73+83.00)+11.56316-96.7821/(91.785-72.1*(51.6704*92.552/(32.9239-78.748)-(47.1707/84.408)))", "13308.3059193646290232"),
("41.25*(56.81*88.01)+68.0-(46.9003)-81.8620+71.9*14.100+(11.327/93.36124)", "207196.883149438278669"),
("(99.221)+93.1*67.70+(39.31+(2.72/21.097)-59.2016)*(53.94/23.348)+17.50364+24.95+90.45581", "6489.3434981248784293961014069820784"),
("(2.497*81.61)*60.72-(76.023)-(56.0)/(87.8+(55.44/(90.26)+(2.6925*70.1204)+(9.143)/79.8150)/(5.71574/85.39960+27.05540))", "12296.918129654197124"),
("67.2+46.9-78.01624*53.31480*(12.1)/85.4145*91.2-(63.6-45.6835*16.951)", "-52913.1074455104904888"),
("(97.4)-67.2+(54.019)*77.36*66.24/87.43608+(28.8)*47.9*75.8-(73.5)*78.007-66.2325/74.33*9.161", "102022.0053553527787263794"),
("57.42*(64.462)/63.02-(77.6516-(66.8*98.435+97.92*53.260*(51.96*(77.7742/(76.35+(7.7+(80.6245/77.98*6.5463)))))))", "238618.4626891788131077667264"),
("(73.01*14.485)/(0.4198)-(53.85/(86.5172)/(39.49)/72.4)*(28.2281)+(8.5936)+34.02-80.78209/(97.361)+(21.3635)-(26.825+66.087)", "2489.40467830588322294532"),
("(5.9+(45.805)/85.657)/(78.531*56.026+74.0075+15.9)+(50.9)*(39.8273-27.96138+47.631+(46.74843+84.12))", "9689.597748229417814"),
("7.162*66.353+66.9/43.84+97.3495-63.300*82.06213+(92.31)-11.878+98.7", "-4441.3051393503649635"),
("0.79/(77.43065-22.339-(78.8)-24.806+(85.917)+(22.0833*71.988)+69.30312+(82.91418/41.630))", "0.0004651354312546"),
("(99.0+55.06-86.646)-(54.454+33.35-(77.3)/(64.5733*(32.994*31.18645+96.5636-11.1298)-(92.3)/(13.94399/(68.3964)+67.65089)-(34.450/31.15957)))", "-20.3889257620919107"),
("3.5+79.5/(45.3103/(12.77)/12.34)/11.0+(75.0587+58.24958-45.2774)/(75.4)/(71.934*(23.72+19.81273))", "28.6356432996469259"),
("95.53+37.63780+(83.018+(72.230)*73.808)-(22.3)-35.5*43.7506*1.5+(75.1)+32.08/(78.287)+(19.9957)-47.08*72.5", "-122.4763357079719494"),
("77.7+(83.56)*(37.4)+12.0871+(87.23119*(31.86)/(36.6)/17.99570+54.3*76.9504-(62.6508*44.32)+(11.7022-40.7))", "4591.8761295565077158"),
("11.952*(92.7476+41.021/(98.62656)*76.29)/(99.1+29.589+2.559)-73.3466-91.59624", "-153.6073154918920115"),
("64.461-22.88+(22.9)+91.54*13.56/28.9252-12.1494/(66.725-85.332+8.0089)-65.4172-21.76", "21.3637039814725284"),
("92.787+(14.5723-(40.68935-3.8035/(12.61)*15.0999-28.4705)+64.3935)-6.8149+51.5883-35.3+(53.8352*89.633)", "4998.97234941522601063649"),
("83.839+(22.7557+(9.89*30.11)*(78.1989-20.077)*(96.440*7.22746)/52.9-(11.19*13.53987)*(41.137*37.409/33.22))", "221140.04372737897527175584969"),
("97.87/55.7470+93.266*(87.427)-63.19-(18.5+54.59274)*86.8917/(82.72*(89.825*16.328))", "8092.4798427571432436"),
("69.56577+10.19659*(53.9190/91.3*5.35)*(18.47)+2.30898*30.3*67.05780*(73.206)+77.82/(43.5503)-63.37332-(56.65939*54.6798)", "340951.1456145128387014845945685"),
("(20.0128)*4.11078/41.20165+(54.056)*54.17877+(40.54090)/33.53-(90.50882/(15.8201+(18.06068+50.3214)+(5.79341)*(27.1)/46.161)+(79.9/(3.727/59.92870)))", "1646.0995394189492598"),
("(43.80835)-10.839+(62.86974*18.671)*(92.474-(16.4505)/36.2*(35.1949-39.7541-98.9))", "163771.2724064544227594178868709248"),
("(60.4461)-37.71-(68.25714)/85.52189/56.16*65.06/(22.82-87.557)/(85.78)*(67.09395+(57.08)/98.169)/(23.48216-28.4621)+99.059-(85.884-57.1)", "93.0088373059094804"),
("(64.50745+34.278-31.140+(52.25116*87.741-94.3)-(93.000+(76.525)*68.8693)-21.82036-(74.2)-(21.5767+(29.58)-(67.8*26.02123)))", "811.75363106"),
("(22.558)/20.5*5.07-(81.397/81.9-50.27800)/(59.7534)-(1.214+(27.22)-19.94-(5.1)-5.74*(25.540)+(71.8-(42.79143)*31.356)-(75.8-71.2/57.72385))", "1494.14399091315859183"),
("(26.204+79.321)+(75.1655/17.009)/(68.4388+(92.299)-(91.37210)-(87.16871)+(18.89-3.914-12.98*(79.7636)+(28.92*85.5084-80.2871+27.092)+17.6564)+92.77317)-(95.97760*55.46)", "-5217.3897340542493272"),
("70.6901-(6.45-(10.7)*30.46112)+(82.64)/95.2541/(10.82)*(91.2162+47.9/82.4498)/7.1-(74.5198/86.87529)", "390.3529982272845288"),
("59.5189+70.409*62.585/(5.8410)+58.58*(38.9)+8.656*52.21-(69.29)*(80.58/11.87302)", "3074.368775273583646156"),
("55.351*(18.681-93.1)/77.04+(41.00399)+(29.5)/(31.07/(16.214-72.0-84.15313)/(98.888)+(13.93107-18.312/7.0036-14.77648))", "-20.984219520618196"),
("(66.9*39.9*61.29392)*75.85*(72.9214)-60.84/98.3528/(75.803-82.1356/48.1234)+(47.9*85.9084)/2.72-54.161+69.29430/(17.76/34.5989-36.563)", "904956477.2197517050761512"),
("(35.9+(13.05613)+20.00271-43.24/3.11)-76.3+53.85505*(19.25-(40.5604)-73.8-17.0298)", "-6060.5607749874919614"),
("(13.2+(67.5)+(81.77331-62.35)-37.9411)/(11.32/(87.70619*(69.077-(46.7)-(73.529*25.55316))))", "-894437.3132401834266139"),
("(46.1832/82.96469+97.2240-54.50*59.62)/84.11388-(20.3168*10.35291)-23.103+38.36584+(60.8)/(70.7*94.19)*(29.7735*(54.65937+(34.83763*11.485)))", "-108.91894850517456148018720811"),
("(21.00-(95.26747-28.84873-(41.7700)+82.19*95.57*(82.4241)*74.872)+(62.61/43.59))", "-48474600.0209386043871163"),
("(78.9)+(90.9746)+(88.622/(7.12-50.3430-(53.9/8.5136*63.44757+(9.46305-36.34)+(64.0+71.0940))))", "169.7143807630540127"),
("5.2*(96.47)-(84.84860-(70.951/(17.196*16.01543*29.327/9.8-(94.659)-(18.7*70.0))))", "416.6729665063680072"),
("91.6*(46.85)*(93.8273)+(34.3124)+73.482-(91.3897-(63.7819-(77.598)-(79.8)/(79.9703-38.8657)/(26.8+71.84517)*(5.22498/2.4943)))", "402658.65223186841219833803621004565091"),
("32.083/(3.3866)*34.553/(86.60+17.11649*(33.1554)*17.6+94.690)+63.418+(2.07)-(12.54225)/45.99069+(8.67)/(78.0303)*99.13520/81.036+8.464", "73.8474028679700169"),
("8.368+(12.708/57.36-69.67948-71.058)+(81.554*(65.53478)/32.251*24.576-58.81309)+34.275*20.9", "4598.111846663777242932"),
("(26.96991)/(15.313)-77.8-(54.211)/(63.9206/79.2+(35.398)+(37.0871)/6.925)+63.158+26.41", "12.2248588019904453"),
("(52.5)+(29.86)*(0.57)/(20.46534)-(27.85*(6.04183)/92.619/27.366)/77.0657*(12.397*39.057)", "52.9145631756737791057185"),
("(59.0/49.4)+(29.4310*(87.861)/56.9821)+(8.6)+(12.4982+97.82)/92.128*53.9236-37.1-20.652", "61.99268345398978546564"),
("(10.472)*7.5-(8.0348+76.886)+98.0794/(24.6+(90.903)+(13.593)*(3.1*(4.2+47.8421-40.94/42.377)*(82.1)/86.9)-(28.3393)*(66.38358*7.3453))", "-6.3892047123880735"),
("98.9/(14.49550)-22.174*(60.06002)/(28.67844)-87.3-(78.01201)*27.2/(65.24712*(38.744)/(52.4)-(0.84403)+(12.11000+1.7/75.90))", "-162.5590728690040862"),
("66.16/36.930*20.791*(50.127)*61.6-(85.5-82.02442)/(90.42+(3.3)/63.3)*62.97490*(85.1)+(40.1040+14.00162-(67.4883)+29.9688*(81.46/85.95))", "114821.36456608956795954937976"),
("85.3797+66.025+(22.2)*8.38/48.855-(4.8)-10.4595-(69.49066/79.25165-9.1)", "148.1762859019992207"),
("61.4374/22.8892/(48.8)-94.057*99.2987+(51.89039*39.5)+40.2567/71.06/(38.003+23.001-(82.22883)*7.028-(47.84255+(89.6098+1.66113-(48.98*41.63619)+68.535+11.491)))", "-7290.0119837261121902"),
("(65.66845)+48.2-(62.44*(8.3)*86.1969/64.1838/37.301)-(75.0316)*12.36*(3.86712/(41.50180/24.341-81.6)*63.2991*47.40702-(47.72*26.00311/(41.4)/95.819/31.0174))", "134805.6197977978758035622071345613888"),
("(88.719)-42.0846-(65.73638)-46.861*(79.4657)+97.45-(36.00793/99.07578-(36.2)/90.5)+57.8895/19.65274/99.176+75.85+22.95/70.9496+93.89555/(64.12-43.53)", "-3564.6941658600054532"),
("19.64337/56.96718-2.6+(59.55)*(66.910/45.478+(35.73+(82.6-(21.1305)-25.63702)+(12.2691/(1.70)+0.83995/(66.8476+(97.047/(17.54544)-26.945-(2.10-(61.3/92.3)))))))", "4777.820291065982541655"),
("(25.9338-(73.42-32.454*(54.0)-56.2936)/(25.28590/(88.28)+(65.46504)*69.73926*(84.336)/74.6121)-77.8075-(18.398)*8.96)/49.15*58.7420", "-258.6124192419976067014"),
("58.68444+(27.9771+38.9)+(6.0043)/(1.54893+(53.8248*(73.4-(0.76750)+26.29519+78.9)))", "125.5621672061336517"),
("6.26480/78.969/(32.05585)*(50.025)*89.4*26.7+(53.38076)-96.711+(25.3)*(7.47849-16.51)+29.1+(90.20164)/(60.1092/46.1)", "121.96632115022759394645"),
("(11.7)*(14.301)/(84.39*53.66)*93.8/(8.537)+(56.90072)*(8.803)*(49.63/20.1624)*67.9433*(69.967+97.229-(73.025-31.0))", "10485783.52967474232404148937388096712"),
("56.691-43.952-(32.10)-56.45673+(97.0-80.41)*91.2*(83.5671-11.51855/(15.53601)-28.08144)/19.70302-(42.2-65.3055)", "4151.1351250306809158"),
("(91.45417+(48.3132)/72.7769)+(51.50708)+(25.9703/(31.3)*(1.04)*(66.79)/47.8)*49.210/(84.367)/70.2-62.5*(16.29046)+(96.3-14.1879)", "-792.4065282171246921"),
("12.6443*(29.6)-(53.77038+(33.2-78.371*(74.10073/(2.29880/10.74854+(72.17-39.238+60.7811+50.107)/82.539)*(64.6157+29.1-(94.9886)*(89.712-37.216*93.9971)))))", "961382698.243192206397847895388770867776"),
("(38.4*(13.0848*55.053)-26.7/20.0669)-38.5*(65.6)*52.8340*21.6*3.04/46.6+(46.362+(71.05913)-28.55226)*(46.79905*(97.560+5.39-89.20))", "-103180.3262296021176548"),
("(33.5486+(98.3337-(11.063*(33.44014)*82.01)/(28.963/54.3977)*78.10826)/(50.8215+73.219))", "-35847.7923091900694923"),
("(74.99783)-(87.78+26.00356)+12.7+86.6+32.3+10.2969/46.8457-53.53427+(15.83-66.1848)", "-10.854995407476033"),
("(37.94885)-91.45941-(3.77*(82.82297-(83.076)/(37.47916*(58.91323)*67.541)*(12.2470*95.077)+90.147)-4.55-98.5)-(39.8397)*92.85+9.6273", "-4289.600780422766101115547362"),
("(27.7418*1.4960*55.51)/85.2012*68.42-(70.576/46.0*(74.068+63.04361)*(43.83113+(29.674*(38.8*86.98/(52.048-62.07-62.5)+20.79847)-(87.47*78.14))))", "1591112.334306702048791493272153652435667521408"),
("44.80585/18.6414/(42.769+(24.93)*30.321)*42.76-(90.1)/(71.16021)*10.7/(16.52649/89.184*42.730/99.88522*16.760)-94.0+45.6665-(62.374)*57.6+2.0+80.320", "-3568.824205650661715544"),
("(5.65-(91.81)+59.05*(31.927)-20.262)-(46.51542/(9.34940)/(71.2808)*(67.6013/(6.12)*11.913)-58.1*73.6247)+11.08*46.9/93.9978-41.720", "6011.08605300744308160020074732161430413"),
("(52.8*54.8099)*(42.2411/(32.56018-(58.43)*(3.5/39.339)*45.3794)+60.9)/(81.16609*85.3508)+17.716+86.23388/(51.1/50.50)", "128.2912092972705256"),
("(97.07*(31.1146+(31.29)*20.8833)/(93.49*15.31)*(79.9)/(4.61235)/(77.5129)/31.211)-(44.89769/(54.28109*12.1595)/(45.824)+20.94)+99.702/28.273+(96.592/52.19820)", "-15.232170162991231"),
("87.421/(78.11-(27.61153)-(78.1445-66.70131)/(55.5)/70.70)*32.625/33.09/(91.45273)*(42.3197)/(31.78+15.55452*15.573-(72.7587+63.652))", "0.0057404291598226"),
("(11.6757)+33.49685+(43.559)+72.80540-(26.3014*(11.17+52.0146)-(7.6)/33.3+2.42/94.7)", "-1500.1038145940315395"),
("60.273/(79.5631)-72.866-68.0664+80.659/77.57857/83.94+76.572/7.82247/51.15993-64.3118", "-204.2829282587953641"),
("23.6817/58.92240/(71.3797)+15.6402*30.8*8.185*5.935/23.767-(30.2134-6.5001/(26.5807-(69.3195)+(77.00+63.2)))", "954.4549081069203237"),
("(52.133-(46.31994*(47.631+4.676-6.5729)+(0.07)-(82.7+31.96*(51.414)+96.22890)/(91.4)+86.208)/41.0)-(4.0134+59.8961)", "-65.0629169960494209"),
("(3.76*98.61105)-(20.0617*(35.4185)+(4.8907+(18.555)-(25.17472)*(0.6227/36.8742)/(17.7785)*85.7/86.034)*88.9489)-(95.34045/80.87824)+65.472-77.7", "-2436.53507563541415315542"),
("27.011/53.45/(53.37)*1.1/(53.129+47.48254+(95.47028/64.1+86.0/81.19140))", "0.0001009662956334"),
("(20.44*33.1592)*80.1*(79.07)-(66.860)*8.896+27.24+6.043/47.15182*3.59+(15.3140*(25.5731)-(81.7570)+98.206/(7.2/(86.3)+4.782)+(59.04*(62.2660/98.68)))", "4292486.89845426863322881"),
("(74.3846/65.70273)+(78.58930*21.336)-67.2+(36.9657-(27.875*(13.09705)-26.4*8.047-(81.3)*(41.5-(70.3212-86.39)+(95.9-37.34099+13.70-94.196-19.01))))", "2846.3928276993377064"),
("(68.8553+20.60933/(17.10*21.562)-(80.980)+(55.81242-(52.3)-48.02297+(33.21/(36.68/26.78797+(10.8651)+92.46576*67.23))))", "-56.574022494248659"),
("(64.989)*32.9-(96.9)+60.4014-17.06603*(8.58+(18.48-(41.29564-29.391+44.0)))", "2593.9029915792"),
("8.96739+(83.4686/95.57831+12.158/68.1757/73.11929/(29.81572*69.0)+21.9)*(38.35)-(48.00/(46.683)-94.18665)/(54.2)+(68.03-(75.3649)/61.4930)-(99.592*55.135)", "-4540.15819935667646422"),
("(41.0+(82.2*14.0709)*(17.9189/(20.32)+90.08974-85.75252-(39.429)+25.4250)+50.8)/(84.2806-(50.55516+52.477)-(63.8)+40.7+54.4-67.324)", "183.824911350546589"),
("(8.2)+(61.31027)*(80.70)-46.72-62.4+(71.96-9.1/(51.82540)/(51.79740-(59.9*44.4)))", "4918.7788563334205222"),
("77.54*86.508+(95.63110)+93.443+22.16*96.48-(98.6136-1.76491)/(20.7910*13.23335)", "9034.5492152100066785"),
("77.35187/8.37-(49.84344+69.52151+(44.2723*87.0381-(40.50+(15.4)*49.0701*(72.0)*74.5430+50.5)/(81.005)*(86.53352*38.6/(57.03-80.610-35.8*31.834+71.6214))))", "-157169.86473751443870274723124909867384"),
("(23.7)-(77.288)/(91.9*8.904+(43.789/(46.0)+0.8*1.93290-(44.788)+(42.74)+45.2399)*(29.4)*(92.75+(38.02)+39.247+20.4827))", "23.699698934293117"),
("(6.3636*(20.7031+20.9)*56.61+(53.7145)/64.6063/53.5024-(23.0)-(43.8*39.67586/26.0938-81.45407*42.8631))", "18389.0332176857809571"),
("4.155*22.7+(75.01-22.98284+(82.62)-98.4994)-(48.3277)*96.253-(63.5951-10.199)", "-4574.6159481"),
("(22.35584)*5.744*(70.04/(80.9943)/(38.5-99.4)/69.4)/50.67159*(90.2400)-(63.7/75.7)*79.11-99.07+(73.20756*(88.95470/(44.115*(50.33195)+(80.85280-(49.22+88.592)))))", "-162.676134673564991559628"),
("55.320+(90.0*(7.32)+(49.149/(43.4-(1.4864)/(93.0026/(2.0/(31.62905+70.07+45.5)/65.67+32.289)/(22.65365/64.7)))))", "715.2571999910689178"),
("(50.37)*31.78158*59.84/13.19537/(72.38*(39.3889-(15.91*(14.288/(77.9524)*40.0427)+(45.3365/68.15663))))", "-1.2851101855344694"),
("67.34548/5.643+99.2370-(47.2-(90.0837/99.444)-(10.95663-75.4-69.73)*45.6/(42.9618)*(82.207*29.1))", "-340618.14427072036819048594"),
("(10.2910)*30.915-54.115/70.8586+(17.89031)/(80.11/27.0835-(63.24020)-35.05283)+(81.6275)+25.671-(82.7379/97.04-50.90291-39.40)+(27.87)/(63.40)/51.81*86.602", "514.6784854381998292716"),
("(53.447)-(88.1549)*(20.505)+(69.8)-(61.1982)+(88.510+(48.7213/(82.0957-(93.5-63.34-68.899))))", "-1656.6542183017804488"),
    ];

  for (expr, expected) in cases {
    let mut val1 = VM::execute(expr).unwrap();
    val1.rescale(6);

    let mut val2 = Decimal::from_str(expected).unwrap();
    val2.rescale(6);

    assert_eq!(val1, val2, "L: {} | R: {}", expr, expected);
  }
}
*/

// ==================== ERROR LOCATION TESTS ====================

#[test]
fn test_error_location_division_by_zero() {
  let code = r#"x = 10
y = 0
result = x / y"#;

  let mut compiler = Compiler::new();
  let (bytecode, debug_info) = compiler
    .compile_from_source(code)
    .expect("Failed to compile");

  let mut vm = VM::new(&bytecode);
  vm.set_debug_info(&debug_info);

  let err = vm.execute().unwrap_err();
  let err_msg = err.to_string();

  // Error should include line 3 (where the division happens)
  assert!(
    err_msg.contains("line 3"),
    "Error should contain line 3, got: {}",
    err_msg
  );
  assert!(
    err_msg.contains("Division by zero"),
    "Error should mention division by zero, got: {}",
    err_msg
  );
}

#[test]
fn test_error_location_modulo_by_zero() {
  let code = r#"x = 15
y = 0
result = x % y"#;

  let mut compiler = Compiler::new();
  let (bytecode, debug_info) = compiler
    .compile_from_source(code)
    .expect("Failed to compile");

  let mut vm = VM::new(&bytecode);
  vm.set_debug_info(&debug_info);

  let err = vm.execute().unwrap_err();
  let err_msg = err.to_string();

  assert!(
    err_msg.contains("line 3"),
    "Error should contain line 3, got: {}",
    err_msg
  );
}

#[test]
fn test_string_number_auto_coercion() {
  // string + number now auto-coerces to string concatenation
  let code = r#"x = "hello"
y = 5
x + y"#;

  let ast = parser::program(code).unwrap();
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).unwrap();
  let mut vm = VM::new(&bytecode);
  let result = vm.execute().unwrap();

  assert_eq!(result, Value::String("hello5".into()));
}

#[test]
fn test_error_location_method_not_found() {
  let code = r#"x = 42
result = x.unknownMethod()"#;

  let mut compiler = Compiler::new();
  let (bytecode, debug_info) = compiler
    .compile_from_source(code)
    .expect("Failed to compile");

  let mut vm = VM::new(&bytecode);
  vm.set_debug_info(&debug_info);

  let err = vm.execute().unwrap_err();
  let err_msg = err.to_string();

  assert!(
    err_msg.contains("line 2"),
    "Error should contain line 2, got: {}",
    err_msg
  );
  assert!(
    err_msg.contains("unknownMethod"),
    "Error should mention method name, got: {}",
    err_msg
  );
}

#[test]
fn test_no_error_location_without_debug_info() {
  // When debug info is not provided, errors should not include line numbers
  let code = "result = 1 / 0";

  let ast = parser::program(code).expect("Failed to parse");
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).expect("Failed to compile");

  // Create VM without setting debug info
  let mut vm = VM::new(&bytecode);

  let err = vm.execute().unwrap_err();
  let err_msg = err.to_string();

  // Should just be the plain error message
  assert_eq!(err_msg, "Division by zero");
}

// ==================== OBJECT TYPE TESTS ====================

fn make_object(entries: Vec<(&str, Value)>) -> Value {
  let mut map = IndexMap::new();
  for (k, v) in entries {
    map.insert(SmolStr::new(k), v);
  }
  Value::Object(map)
}

#[test]
fn test_object_property_access() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
    ("age", Value::Number(dec!(30))),
  ]);
  let result = run_expr_with_globals("customer.name", vec![("customer", obj)]);
  assert_eq!(result, Value::String("Alice".into()));
}

#[test]
fn test_object_property_access_number() {
  let obj = make_object(vec![
    ("age", Value::Number(dec!(30))),
  ]);
  let result = run_expr_with_globals("person.age", vec![("person", obj)]);
  assert_eq!(result, Value::Number(dec!(30)));
}

#[test]
fn test_object_property_access_missing() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let result = run_expr_with_globals("person.missing", vec![("person", obj)]);
  assert_eq!(result, Value::Null);
}

#[test]
fn test_object_nested_property_access() {
  let address = make_object(vec![
    ("city", Value::String("Istanbul".into())),
    ("zip", Value::String("34000".into())),
  ]);
  let person = make_object(vec![
    ("name", Value::String("Duhan".into())),
    ("address", address),
  ]);
  let result = run_expr_with_globals("person.address.city", vec![("person", person)]);
  assert_eq!(result, Value::String("Istanbul".into()));
}

#[test]
fn test_object_property_assignment() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
    ("age", Value::Number(dec!(30))),
  ]);
  let code = r#"
    person.name = "Bob"
    person.name
  "#;
  let result = run_expr_with_globals(code, vec![("person", obj)]);
  assert_eq!(result, Value::String("Bob".into()));
}

#[test]
fn test_object_nested_property_assignment() {
  let address = make_object(vec![
    ("city", Value::String("Istanbul".into())),
  ]);
  let person = make_object(vec![
    ("name", Value::String("Duhan".into())),
    ("address", address),
  ]);
  let code = r#"
    person.address.city = "Ankara"
    person.address.city
  "#;
  let result = run_expr_with_globals(code, vec![("person", person)]);
  assert_eq!(result, Value::String("Ankara".into()));
}

#[test]
fn test_object_method_keys() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
    ("age", Value::Number(dec!(30))),
  ]);
  let result = run_expr_with_globals("person.keys()", vec![("person", obj)]);
  assert_eq!(
    result,
    Value::StringList(vec![SmolStr::new("name"), SmolStr::new("age")])
  );
}

#[test]
fn test_object_method_length() {
  let obj = make_object(vec![
    ("a", Value::Number(dec!(1))),
    ("b", Value::Number(dec!(2))),
    ("c", Value::Number(dec!(3))),
  ]);
  let result = run_expr_with_globals("obj.length()", vec![("obj", obj)]);
  assert_eq!(result, Value::Number(dec!(3)));
}

#[test]
fn test_object_method_contains() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#"obj.contains("name")"#;
  let result = run_expr_with_globals(code, vec![("obj", obj)]);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_object_method_contains_missing() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#"obj.contains("missing")"#;
  let result = run_expr_with_globals(code, vec![("obj", obj)]);
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_object_method_get() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#"obj.get("name")"#;
  let result = run_expr_with_globals(code, vec![("obj", obj)]);
  assert_eq!(result, Value::String("Alice".into()));
}

#[test]
fn test_object_method_get_missing() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#"obj.get("missing")"#;
  let result = run_expr_with_globals(code, vec![("obj", obj)]);
  assert_eq!(result, Value::Null);
}

#[test]
fn test_object_method_values_strings() {
  let obj = make_object(vec![
    ("a", Value::String("x".into())),
    ("b", Value::String("y".into())),
  ]);
  let result = run_expr_with_globals("obj.values()", vec![("obj", obj)]);
  assert_eq!(
    result,
    Value::StringList(vec![SmolStr::new("x"), SmolStr::new("y")])
  );
}

#[test]
fn test_object_method_values_numbers() {
  let obj = make_object(vec![
    ("a", Value::Number(dec!(1))),
    ("b", Value::Number(dec!(2))),
  ]);
  let result = run_expr_with_globals("obj.values()", vec![("obj", obj)]);
  assert_eq!(result, Value::NumberList(vec![dec!(1), dec!(2)]));
}

#[test]
fn test_object_in_operator() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
    ("age", Value::Number(dec!(30))),
  ]);
  let code = r#""name" in person"#;
  let result = run_expr_with_globals(code, vec![("person", obj)]);
  assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_object_in_operator_missing() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#""missing" in person"#;
  let result = run_expr_with_globals(code, vec![("person", obj)]);
  assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_object_property_in_expression() {
  let obj = make_object(vec![
    ("price", Value::Number(dec!(100))),
    ("quantity", Value::Number(dec!(5))),
  ]);
  let code = "order.price * order.quantity";
  let result = run_expr_with_globals(code, vec![("order", obj)]);
  assert_eq!(result, Value::Number(dec!(500)));
}

#[test]
fn test_object_property_in_if() {
  let obj = make_object(vec![
    ("active", Value::Boolean(true)),
    ("discount", Value::Number(dec!(10))),
  ]);
  let code = r#"
    if customer.active then
      result = customer.discount
    else
      result = 0
    end
  "#;
  let ast = parser::program(code).unwrap();
  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast).unwrap();
  let mut vm = VM::new(&bytecode);
  vm.set_global("customer", obj);
  vm.execute().unwrap();
  let val = vm.get_global("result").unwrap().clone();
  assert_eq!(val, Value::Number(dec!(10)));
}

#[test]
fn test_object_property_with_method_call() {
  let obj = make_object(vec![
    ("name", Value::String("hello world".into())),
  ]);
  let code = "obj.name.upper()";
  let result = run_expr_with_globals(code, vec![("obj", obj)]);
  assert_eq!(result, Value::String("HELLO WORLD".into()));
}

#[test]
fn test_object_add_new_property() {
  let obj = make_object(vec![
    ("name", Value::String("Alice".into())),
  ]);
  let code = r#"
    person.age = 25
    person.age
  "#;
  let result = run_expr_with_globals(code, vec![("person", obj)]);
  assert_eq!(result, Value::Number(dec!(25)));
}

// ==================== VALUE::FROM_JSON TESTS ====================

#[test]
fn test_from_json_object() {
  let val = Value::from_json(r#"{"name": "Alice", "age": 30, "active": true}"#).unwrap();
  let code = r#"
    result = person.name
  "#;
  let result = run_expr_with_globals(code, vec![("person", val)]);
  assert_eq!(result, Value::Null); // result is in global
  // use the helper that reads globals
  let val2 = Value::from_json(r#"{"name": "Alice", "age": 30, "active": true}"#).unwrap();
  let r = run_and_get_result_with_globals("result = person.name", vec![("person", val2)]);
  assert_eq!(r, Value::String("Alice".into()));
}

#[test]
fn test_from_json_nested_object() {
  let json = r#"{"address": {"city": "Istanbul", "zip": "34000"}}"#;
  let val = Value::from_json(json).unwrap();
  let result = run_expr_with_globals("person.address.city", vec![("person", val)]);
  assert_eq!(result, Value::String("Istanbul".into()));
}

#[test]
fn test_from_json_with_arrays() {
  let json = r#"{"tags": ["vip", "tr"], "scores": [10, 20, 30]}"#;
  let val = Value::from_json(json).unwrap();
  if let Value::Object(map) = &val {
    assert!(matches!(map.get("tags").unwrap(), Value::StringList(_)));
    assert!(matches!(map.get("scores").unwrap(), Value::NumberList(_)));
  } else {
    panic!("Expected Object");
  }
}

#[test]
fn test_from_json_primitives() {
  assert_eq!(Value::from_json("null").unwrap(), Value::Null);
  assert_eq!(Value::from_json("true").unwrap(), Value::Boolean(true));
  assert_eq!(Value::from_json("42").unwrap(), Value::Number(dec!(42)));
  assert_eq!(Value::from_json(r#""hello""#).unwrap(), Value::String("hello".into()));
}

#[test]
fn test_from_json_full_workflow() {
  // Simulate: JSON from API → Value → VM execution
  let customer_json = r#"{
    "name": "Duhan",
    "age": 30,
    "active": true,
    "tags": ["premium", "tr"]
  }"#;
  let customer = Value::from_json(customer_json).unwrap();

  let code = r#"
    if customer.active then
      result = customer.tags.first()
    else
      result = "none"
    end
  "#;
  let r = run_and_get_result_with_globals(code, vec![("customer", customer)]);
  assert_eq!(r, Value::String("premium".into()));
}