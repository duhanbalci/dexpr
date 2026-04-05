use criterion::{criterion_group, criterion_main, Criterion};
use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};
use rust_decimal_macros::dec;



pub fn criterion_benchmark(c: &mut Criterion) {
  // 1. Parser Benchmark
  c.bench_function("parser_long", |b| {
    let input = include_str!("../examples/bench_long.dexpr");
    b.iter(|| {
      let _ = parser::program(input).unwrap();
    })
  });

  // 2. Compiler Benchmark
  c.bench_function("compiler_long", |b| {
    let input = include_str!("../examples/bench_long.dexpr");
    let ast = parser::program(input).unwrap();
    b.iter(|| {
      let mut compiler = Compiler::new();
      let _ = compiler.compile(ast.clone()).unwrap();
    })
  });

  // 3. VM Benchmarks

  // basic_long.dexpr benchmark
  c.bench_function("vm_basic_long", |b| {
    let input = include_str!("../examples/basic_long.dexpr");
    let ast = parser::program(input).unwrap();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(ast).unwrap();
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("test", Value::Number(dec!(3)));
      let _ = vm.execute().unwrap();
    })
  });

  // Long code benchmark (using bench_long.dexpr)
  c.bench_function("vm_long", |b| {
    let input = include_str!("../examples/bench_long.dexpr");
    let ast = parser::program(input).unwrap();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(ast).unwrap();
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  // Short code benchmark
  c.bench_function("vm_short", |b| {
    let input = "5.12 + test * 1.5";
    let ast = parser::program(input).unwrap();
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(ast).unwrap();
    let test_val = dec!(100);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("test", Value::Number(test_val));
      let _ = vm.execute().unwrap();
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
