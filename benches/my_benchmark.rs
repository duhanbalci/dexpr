use criterion::{criterion_group, criterion_main, Criterion};
use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use smol_str::SmolStr;

/// Helper: compile source to bytecode
fn compile(source: &str) -> Vec<u8> {
  let ast = parser::program(source).unwrap();
  let mut compiler = Compiler::new();
  compiler.compile(ast).unwrap()
}

/// Helper: build a sample object with N fields
fn sample_object(n: usize) -> Value {
  let mut map = IndexMap::new();
  for i in 0..n {
    map.insert(SmolStr::new(format!("field{}", i)), Value::Number(Decimal::from(i)));
  }
  Value::Object(Box::new(map))
}

pub fn criterion_benchmark(c: &mut Criterion) {
  // ── Existing benchmarks ──────────────────────────────────────────────

  c.bench_function("parser_long", |b| {
    let input = include_str!("../examples/bench_long.dexpr");
    b.iter(|| {
      let _ = parser::program(input).unwrap();
    })
  });

  c.bench_function("compiler_long", |b| {
    let input = include_str!("../examples/bench_long.dexpr");
    let ast = parser::program(input).unwrap();
    b.iter(|| {
      let mut compiler = Compiler::new();
      let _ = compiler.compile(ast.clone()).unwrap();
    })
  });

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

  c.bench_function("vm_short", |b| {
    let input = "5.12 + test * 1.5";
    let bytecode = compile(input);
    let test_val = dec!(100);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("test", Value::Number(test_val));
      let _ = vm.execute().unwrap();
    })
  });

  // ── #1: Method dispatch clone overhead ───────────────────────────────

  // Object method — clone entire IndexMap per call
  c.bench_function("vm_object_method_keys", |b| {
    let bytecode = compile("obj.keys()");
    let obj = sample_object(20);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("obj", obj.clone());
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_object_method_length", |b| {
    let bytecode = compile("obj.length()");
    let obj = sample_object(20);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("obj", obj.clone());
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_object_method_contains", |b| {
    let bytecode = compile(r#"obj.contains("field10")"#);
    let obj = sample_object(20);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("obj", obj.clone());
      let _ = vm.execute().unwrap();
    })
  });

  // StringList method — clone entire Vec per call
  c.bench_function("vm_strlist_method_length", |b| {
    let bytecode = compile("items.length()");
    let items = Value::StringList(Box::new((0..50).map(|i| SmolStr::new(format!("item{}", i))).collect()));
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("items", items.clone());
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_numlist_method_sum", |b| {
    let bytecode = compile("nums.sum()");
    let nums = Value::NumberList(Box::new((0..100).map(Decimal::from).collect()));
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("nums", nums.clone());
      let _ = vm.execute().unwrap();
    })
  });

  // String method — lighter clone (SmolStr)
  c.bench_function("vm_string_method_upper", |b| {
    let bytecode = compile(r#"s.upper()"#);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("s", Value::String(SmolStr::new("hello world this is a test string")));
      let _ = vm.execute().unwrap();
    })
  });

  // ── #2 & #3: Vec alloc in method/external calls ─────────────────────

  c.bench_function("vm_method_call_with_args", |b| {
    let bytecode = compile(r#"s.replace("hello", "world")"#);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("s", Value::String(SmolStr::new("hello world hello")));
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_external_fn_call", |b| {
    let bytecode = compile("getRate(a, b)");
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("a", Value::Number(dec!(10)));
      vm.set_global("b", Value::Number(dec!(20)));
      vm.register_function("getRate", |_args| Ok(Value::Number(dec!(34.5))));
      let _ = vm.execute().unwrap();
    })
  });

  // ── #4: Value enum size (cache pressure on register ops) ─────────────

  c.bench_function("vm_arithmetic_chain", |b| {
    // Pure arithmetic — measures register read/write cache performance
    let bytecode = compile(
      "a = 1.5\nb = 2.3\nc = a + b\nd = c * a\ne = d - b\nf = e / c\ng = f + d\ng * 2.0",
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_comparison_chain", |b| {
    let bytecode = compile(
      "a = 10\nb = 20\nc = a < b\nd = b >= a\ne = a == 10\nf = b != 15\nc && d && e && f",
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  // ── #5: SmolStr alloc per opcode (string table missing) ──────────────

  c.bench_function("vm_global_read_heavy", |b| {
    // Many LoadGlobal ops → SmolStr alloc per read (split to stay within register limit)
    let bytecode = compile(
      "r1 = x1 + x2 + x3\nr2 = x4 + x5 + x6\nr = r1 + r2\nr",
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      for i in 1..=6 {
        vm.set_global(&format!("x{}", i), Value::Number(Decimal::from(i)));
      }
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_global_write_heavy", |b| {
    // Many StoreGlobal ops
    let bytecode = compile(
      "a = 1\nb = 2\nc = 3\nd = 4\ne = 5\nf = 6\nr = a + b + c\ns = d + e + f\nr + s",
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_property_access_chain", |b| {
    // GetProperty → read_string per access
    let bytecode = compile("obj.field0 + obj.field1 + obj.field2 + obj.field3 + obj.field4");
    let obj = sample_object(10);
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      vm.set_global("obj", obj.clone());
      let _ = vm.execute().unwrap();
    })
  });

  // ── #8: String concat with format! ───────────────────────────────────

  c.bench_function("vm_string_concat", |b| {
    let bytecode = compile(
      r#"a = "hello" + " " + "world"
b = a + " " + "this"
c = b + " " + "test"
c"#,
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  c.bench_function("vm_string_number_coerce", |b| {
    let bytecode = compile(
      r#"a = "value: " + 42
b = a + " and " + 3.14
b"#,
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  // ── #6: Opcode dispatch (overall loop throughput) ────────────────────

  c.bench_function("vm_opcode_throughput", |b| {
    // Many simple ops to stress the dispatch loop
    let bytecode = compile(
      "a = 1\nb = 2\nc = a + b\nd = c * 2\ne = d - 1\nf = e / 3\n\
       g = f + a\nh = g * b\ni = h - c\nj = i + d\n\
       k = j * 2\nl = k - 1\nm = l + 3\nn = m / 2\n\
       o = n + a\np = o * b\np",
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });

  // ── Combined: realistic expression with multiple issue areas ─────────

  c.bench_function("vm_realistic_mixed", |b| {
    let bytecode = compile(
      r#"
      price = 100.50
      tax = 18
      discount = 5.5
      net = price * (1 + tax / 100) - discount
      label = "Total: " + net
      if net > 100 then
        result = label + " (high)"
      else
        result = label + " (low)"
      end
      result
      "#,
    );
    b.iter(|| {
      let mut vm = VM::new(&bytecode);
      let _ = vm.execute().unwrap();
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
