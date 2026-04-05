use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(serde::Deserialize)]
struct TestCase {
    name: String,
    code: String,
    #[serde(default)]
    globals: HashMap<String, ValueDef>,
    expected: ValueDef,
}

#[derive(serde::Deserialize)]
struct ValueDef {
    #[serde(rename = "type")]
    typ: String,
    #[serde(default)]
    value: Option<serde_json::Value>,
}

fn value_def_to_value(def: &ValueDef) -> Value {
    match def.typ.as_str() {
        "null" => Value::Null,
        "number" => {
            let s = def.value.as_ref().unwrap().as_str().unwrap();
            Value::Number(Decimal::from_str(s).unwrap())
        }
        "string" => {
            let s = def.value.as_ref().unwrap().as_str().unwrap();
            Value::String(s.into())
        }
        "boolean" => {
            let b = def.value.as_ref().unwrap().as_bool().unwrap();
            Value::Boolean(b)
        }
        "object" => {
            fn json_obj_to_value(obj: &serde_json::Map<String, serde_json::Value>) -> Value {
                let mut map = IndexMap::new();
                for (k, v) in obj {
                    let val = match v {
                        serde_json::Value::String(s) => {
                            if let Ok(d) = Decimal::from_str(s) {
                                Value::Number(d)
                            } else {
                                Value::String(SmolStr::from(s.as_str()))
                            }
                        }
                        serde_json::Value::Bool(b) => Value::Boolean(*b),
                        serde_json::Value::Object(nested) => json_obj_to_value(nested),
                        _ => Value::String(SmolStr::from(v.to_string())),
                    };
                    map.insert(SmolStr::from(k.as_str()), val);
                }
                Value::Object(Box::new(map))
            }
            let obj = def.value.as_ref().unwrap().as_object().unwrap();
            json_obj_to_value(obj)
        }
        other => panic!("Unknown type: {other}"),
    }
}

#[test]
fn test_all_cases() {
    let json = include_str!("test_cases.json");
    let cases: Vec<TestCase> =
        serde_json::from_str(json).expect("Failed to parse test_cases.json");

    let mut failures = Vec::new();
    let total = cases.len();

    for case in &cases {
        let ast = match parser::program(&case.code) {
            Ok(ast) => ast,
            Err(e) => {
                failures.push(format!(
                    "FAIL: {}\n  code:  {}\n  error: parse failed: {e}",
                    case.name,
                    case.code.replace('\n', "\\n"),
                ));
                continue;
            }
        };

        let mut compiler = Compiler::new();
        let bytecode = match compiler.compile(ast) {
            Ok(bc) => bc,
            Err(e) => {
                failures.push(format!(
                    "FAIL: {}\n  code:  {}\n  error: compile failed: {e}",
                    case.name,
                    case.code.replace('\n', "\\n"),
                ));
                continue;
            }
        };

        let mut vm = VM::new(&bytecode);

        for (name, def) in &case.globals {
            vm.set_global(name, value_def_to_value(def));
        }

        let result = match vm.execute() {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!(
                    "FAIL: {}\n  code:  {}\n  error: execute failed: {e}",
                    case.name,
                    case.code.replace('\n', "\\n"),
                ));
                continue;
            }
        };

        let expected = value_def_to_value(&case.expected);

        if result != expected {
            failures.push(format!(
                "FAIL: {}\n  code:     {}\n  expected: {:?}\n  got:      {:?}",
                case.name,
                case.code.replace('\n', "\\n"),
                expected,
                result
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n{} / {} test cases failed:\n\n{}\n",
            failures.len(),
            total,
            failures.join("\n\n")
        );
    }

    eprintln!("All {total} test cases passed.");
}
