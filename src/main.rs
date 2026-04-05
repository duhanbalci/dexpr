use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let input = include_str!("../examples/basic_long.dexpr");

  let ast = parser::program(input)?;

  let mut compiler = Compiler::new();
  let bytecode = compiler.compile(ast)?;

  let num = dec!(3);
  let mut vm = VM::new(&bytecode);
  vm.set_global("test", Value::Number(num));
  let res = vm.execute();
  if res.is_err() {
    println!("Error: {:?}", res.unwrap_err());
  }

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

  println!("Error message:\n{}", err_msg);

  Ok(())
}
