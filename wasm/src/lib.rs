use dexpr::ast::value::Value;
use dexpr::compiler::Compiler;
use dexpr::language_info::LanguageInfo;
use dexpr::vm::VM;
use indexmap::IndexMap;
use smol_str::SmolStr;
use wasm_bindgen::prelude::*;

/// Convert a dexpr Value to a serde_json::Value for JS interop.
fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => {
            if n.scale() == 0 {
                if let Ok(i) = n.to_string().parse::<i64>() {
                    return serde_json::Value::Number(i.into());
                }
            }
            if let Some(f) = serde_json::Number::from_f64(n.to_string().parse::<f64>().unwrap_or(0.0)) {
                serde_json::Value::Number(f)
            } else {
                serde_json::Value::String(n.to_string())
            }
        }
        Value::String(s) => serde_json::Value::String(s.to_string()),
        Value::NumberList(list) => {
            serde_json::Value::Array(list.iter().map(|n| value_to_json(&Value::Number(*n))).collect())
        }
        Value::StringList(list) => {
            serde_json::Value::Array(list.iter().map(|s| serde_json::Value::String(s.to_string())).collect())
        }
        Value::Object(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.to_string(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}

/// dexpr engine for browser use via WASM.
///
/// Holds global variables and language metadata.
/// Compile + execute expressions in one call via `execute()`.
#[wasm_bindgen]
pub struct DexprEngine {
    globals: IndexMap<SmolStr, Value>,
    language_info: LanguageInfo,
    external_fns: Vec<(String, js_sys::Function)>,
}

#[wasm_bindgen]
impl DexprEngine {
    /// Create a new engine instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            globals: IndexMap::new(),
            language_info: LanguageInfo::builtin(),
            external_fns: Vec::new(),
        }
    }

    /// Set a global variable from a JSON value string.
    ///
    /// ```js
    /// engine.setGlobal("customer", '{"name": "Alice", "age": 30}');
    /// engine.setGlobal("price", "100");
    /// engine.setGlobal("name", '"hello"');
    /// ```
    #[wasm_bindgen(js_name = setGlobal)]
    pub fn set_global(&mut self, name: &str, json: &str) -> Result<(), JsError> {
        let value = Value::from_json(json).map_err(|e| JsError::new(&e))?;
        self.update_language_info(name, &value);
        self.globals.insert(SmolStr::new(name), value);
        Ok(())
    }

    /// Get a global variable as JSON string. Returns undefined if not found.
    #[wasm_bindgen(js_name = getGlobal)]
    pub fn get_global(&self, name: &str) -> Option<String> {
        self.globals
            .get(name)
            .map(|v| serde_json::to_string(&value_to_json(v)).unwrap_or_default())
    }

    /// Register an external function callable from dexpr code.
    ///
    /// The JS function receives arguments as a JSON array string
    /// and must return a JSON value string.
    ///
    /// ```js
    /// engine.registerFunction("getRate", (argsJson) => {
    ///   const args = JSON.parse(argsJson);
    ///   return JSON.stringify(34.5);
    /// });
    /// ```
    #[wasm_bindgen(js_name = registerFunction)]
    pub fn register_function(&mut self, name: &str, f: js_sys::Function) {
        self.external_fns.push((name.to_string(), f));
    }

    /// Compile and execute dexpr source code. Returns the result as JSON string.
    ///
    /// ```js
    /// const result = engine.execute('customer.name.upper()');
    /// const parsed = JSON.parse(result);
    /// // → "ALICE"
    /// ```
    pub fn execute(&mut self, source: &str) -> Result<String, JsError> {
        // Compile
        let mut compiler = Compiler::new();
        let (bytecode, debug_info) = compiler
            .compile_from_source(source)
            .map_err(|e| JsError::new(&e.to_string()))?;

        // Create VM
        let mut vm = VM::new(&bytecode);
        vm.set_debug_info(&debug_info);

        // Apply globals
        for (name, value) in &self.globals {
            vm.set_global(name, value.clone());
        }

        // Apply external functions
        for (name, js_fn) in &self.external_fns {
            let js_fn = js_fn.clone();
            vm.register_function(name, move |args: &[Value]| {
                let json_args: Vec<serde_json::Value> = args.iter().map(value_to_json).collect();
                let args_str = serde_json::to_string(&json_args).unwrap_or_default();

                let this = JsValue::NULL;
                let js_arg = JsValue::from_str(&args_str);
                let result = js_fn
                    .call1(&this, &js_arg)
                    .map_err(|e| format!("JS function error: {:?}", e))?;

                let result_str = result
                    .as_string()
                    .ok_or_else(|| "External function must return a JSON string".to_string())?;
                Value::from_json(&result_str)
            });
        }

        // Execute
        let result = vm.execute().map_err(|e| JsError::new(&e.to_string()))?;

        // Collect modified globals back
        for (name, _) in self.globals.clone().iter() {
            if let Some(val) = vm.get_global(name) {
                self.globals.insert(name.clone(), val.clone());
            }
        }

        // Return result as JSON
        let json = serde_json::to_string(&value_to_json(&result))
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(json)
    }

    /// Get language metadata JSON for the CodeMirror editor.
    /// Includes built-in + registered functions/methods and variables with field types.
    /// Variables are auto-populated from setGlobal calls.
    ///
    /// ```js
    /// const metadata = JSON.parse(engine.languageInfo());
    /// const extensions = [basicSetup, dexpr(metadata)];
    /// ```
    #[wasm_bindgen(js_name = languageInfo)]
    pub fn language_info(&self) -> String {
        self.language_info.to_json()
    }

    /// Add a host function to the language metadata (for editor autocomplete).
    /// Call this alongside registerFunction so the editor knows about it.
    #[wasm_bindgen(js_name = addFunctionInfo)]
    pub fn add_function_info(&mut self, name: &str, signature: &str, doc: Option<String>) {
        let name: &'static str = Box::leak(name.to_string().into_boxed_str());
        let signature: &'static str = Box::leak(signature.to_string().into_boxed_str());
        let doc: Option<&'static str> = doc.map(|d| &*Box::leak(d.into_boxed_str()));
        self.language_info.add_function(name, signature, doc);
    }

    /// Internal: update language info when a global is set.
    fn update_language_info(&mut self, name: &str, value: &Value) {
        self.language_info.variables.retain(|v| v.name != name);
        self.language_info.add_value(name, value, None);
    }
}
