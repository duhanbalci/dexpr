//! Language metadata for editor integration.
//!
//! Generates JSON describing built-in functions, methods per type,
//! and host-registered extensions. The frontend editor library
//! (`codemirror-lang-dexpr`) consumes this to provide type-aware autocomplete.
//!
//! # Usage
//! ```rust
//! use dexpr::language_info::LanguageInfo;
//! use dexpr::ast::value::Value;
//! use indexmap::IndexMap;
//! use smol_str::SmolStr;
//! use rust_decimal_macros::dec;
//! use std::rc::Rc;
//!
//! let mut info = LanguageInfo::builtin();
//!
//! // Add host-registered functions
//! info.add_function("getRate", "(code: String) -> Number", Some("Get exchange rate"));
//!
//! // Add host-registered methods
//! info.add_method("String", "toTitleCase", "() -> String", None);
//!
//! // Add external variables — type inferred from Value
//! let mut customer = IndexMap::new();
//! customer.insert(SmolStr::new("name"), Value::String("Alice".into()));
//! customer.insert(SmolStr::new("age"), Value::Number(dec!(30)));
//! info.add_value("customer", &Value::Object(Rc::new(customer)), None);
//! info.add_value("price", &Value::Number(dec!(100)), None);
//!
//! let json = info.to_json();
//! // Send `json` to frontend
//! ```

use crate::ast::value::Value;

/// Function metadata
pub struct FunctionInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub doc: Option<&'static str>,
}

/// Method metadata
pub struct MethodInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub doc: Option<&'static str>,
}

/// Field metadata for Object type variables
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
}

/// Variable metadata
pub struct VariableInfo {
    pub name: String,
    pub type_name: String,
    pub doc: Option<String>,
    pub fields: Option<Vec<FieldInfo>>,
}

/// Collected language metadata for editor autocomplete
pub struct LanguageInfo {
    pub functions: Vec<FunctionInfo>,
    pub methods: Vec<(&'static str, Vec<MethodInfo>)>,
    pub variables: Vec<VariableInfo>,
}

impl LanguageInfo {
    /// Create metadata with all built-in functions and methods
    pub fn builtin() -> Self {
        Self {
            functions: builtin_functions(),
            methods: builtin_methods(),
            variables: Vec::new(),
        }
    }

    /// Add a host-registered function
    pub fn add_function(&mut self, name: &'static str, signature: &'static str, doc: Option<&'static str>) {
        self.functions.push(FunctionInfo { name, signature, doc });
    }

    /// Add a host-registered method on a type
    pub fn add_method(&mut self, type_name: &'static str, name: &'static str, signature: &'static str, doc: Option<&'static str>) {
        if let Some(entry) = self.methods.iter_mut().find(|(t, _)| *t == type_name) {
            entry.1.push(MethodInfo { name, signature, doc });
        } else {
            self.methods.push((type_name, vec![MethodInfo { name, signature, doc }]));
        }
    }

    /// Add an external variable
    pub fn add_variable(&mut self, name: impl Into<String>, type_name: impl Into<String>, doc: Option<String>) {
        self.variables.push(VariableInfo {
            name: name.into(),
            type_name: type_name.into(),
            doc,
            fields: None,
        });
    }

    /// Add a variable by inspecting a Value — type and Object fields are derived automatically.
    ///
    /// This is the recommended way to register variables for editor autocomplete.
    /// It mirrors what you pass to `vm.set_global()`.
    ///
    /// ```ignore
    /// vm.set_global("customer", customer.clone());
    /// info.add_value("customer", &customer, None);
    /// ```
    pub fn add_value(&mut self, name: impl Into<String>, value: &Value, doc: Option<String>) {
        let type_name = value.type_name().to_string();
        let fields = match value {
            Value::Object(map) => {
                Some(map.iter().map(|(k, v)| FieldInfo {
                    name: k.to_string(),
                    type_name: v.type_name().to_string(),
                }).collect())
            }
            _ => None,
        };
        self.variables.push(VariableInfo {
            name: name.into(),
            type_name,
            doc,
            fields,
        });
    }

    /// Add an Object variable with manually specified field types.
    /// Use `add_value` instead when you have the actual Value.
    pub fn add_object_variable(&mut self, name: impl Into<String>, fields: Vec<(&str, &str)>, doc: Option<String>) {
        self.variables.push(VariableInfo {
            name: name.into(),
            type_name: "Object".to_string(),
            doc,
            fields: Some(fields.into_iter().map(|(n, t)| FieldInfo {
                name: n.to_string(),
                type_name: t.to_string(),
            }).collect()),
        });
    }

    /// Serialize to JSON string for the frontend editor
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(2048);
        out.push_str("{\n  \"functions\": [");
        for (i, f) in self.functions.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push_str("\n    {\"name\":\"");
            out.push_str(f.name);
            out.push_str("\",\"signature\":\"");
            out.push_str(f.signature);
            out.push('"');
            if let Some(doc) = f.doc {
                out.push_str(",\"doc\":\"");
                out.push_str(&escape_json(doc));
                out.push('"');
            }
            out.push('}');
        }
        out.push_str("\n  ],\n  \"methods\": {");
        for (i, (type_name, methods)) in self.methods.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push_str("\n    \"");
            out.push_str(type_name);
            out.push_str("\": [");
            for (j, m) in methods.iter().enumerate() {
                if j > 0 { out.push(','); }
                out.push_str("\n      {\"name\":\"");
                out.push_str(m.name);
                out.push_str("\",\"signature\":\"");
                out.push_str(m.signature);
                out.push('"');
                if let Some(doc) = m.doc {
                    out.push_str(",\"doc\":\"");
                    out.push_str(&escape_json(doc));
                    out.push('"');
                }
                out.push('}');
            }
            out.push_str("\n    ]");
        }
        out.push_str("\n  },\n  \"variables\": [");
        for (i, v) in self.variables.iter().enumerate() {
            if i > 0 { out.push(','); }
            out.push_str("\n    {\"name\":\"");
            out.push_str(&escape_json(&v.name));
            out.push_str("\",\"type\":\"");
            out.push_str(&v.type_name);
            out.push('"');
            if let Some(doc) = &v.doc {
                out.push_str(",\"doc\":\"");
                out.push_str(&escape_json(doc));
                out.push('"');
            }
            if let Some(fields) = &v.fields {
                out.push_str(",\"fields\":[");
                for (j, f) in fields.iter().enumerate() {
                    if j > 0 { out.push(','); }
                    out.push_str("{\"name\":\"");
                    out.push_str(&escape_json(&f.name));
                    out.push_str("\",\"type\":\"");
                    out.push_str(&f.type_name);
                    out.push_str("\"}");
                }
                out.push(']');
            }
            out.push('}');
        }
        out.push_str("\n  ]\n}");
        out
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn builtin_functions() -> Vec<FunctionInfo> {
    vec![
        FunctionInfo { name: "log", signature: "(...args) -> null", doc: Some("Print values to output") },
        FunctionInfo { name: "rand", signature: "(min, max) -> Number", doc: Some("Random integer between min and max (inclusive)") },
    ]
}

fn builtin_methods() -> Vec<(&'static str, Vec<MethodInfo>)> {
    vec![
        ("String", vec![
            MethodInfo { name: "upper", signature: "() -> String", doc: None },
            MethodInfo { name: "lower", signature: "() -> String", doc: None },
            MethodInfo { name: "trim", signature: "() -> String", doc: None },
            MethodInfo { name: "trimStart", signature: "() -> String", doc: None },
            MethodInfo { name: "trimEnd", signature: "() -> String", doc: None },
            MethodInfo { name: "split", signature: "(delim: String) -> StringList", doc: None },
            MethodInfo { name: "replace", signature: "(old: String, new: String) -> String", doc: None },
            MethodInfo { name: "contains", signature: "(substr: String) -> Boolean", doc: None },
            MethodInfo { name: "startsWith", signature: "(prefix: String) -> Boolean", doc: None },
            MethodInfo { name: "endsWith", signature: "(suffix: String) -> Boolean", doc: None },
            MethodInfo { name: "length", signature: "() -> Number", doc: None },
            MethodInfo { name: "charAt", signature: "(index: Number) -> String", doc: None },
            MethodInfo { name: "substring", signature: "(start: Number, end?: Number) -> String", doc: None },
        ]),
        ("Number", vec![]),
        ("Boolean", vec![]),
        ("NumberList", vec![
            MethodInfo { name: "length", signature: "() -> Number", doc: None },
            MethodInfo { name: "len", signature: "() -> Number", doc: None },
            MethodInfo { name: "isEmpty", signature: "() -> Boolean", doc: None },
            MethodInfo { name: "first", signature: "() -> Number", doc: None },
            MethodInfo { name: "last", signature: "() -> Number", doc: None },
            MethodInfo { name: "get", signature: "(index: Number) -> Number", doc: None },
            MethodInfo { name: "contains", signature: "(value: Number) -> Boolean", doc: None },
            MethodInfo { name: "indexOf", signature: "(value: Number) -> Number", doc: None },
            MethodInfo { name: "slice", signature: "(start: Number, end?: Number) -> NumberList", doc: None },
            MethodInfo { name: "reverse", signature: "() -> NumberList", doc: None },
            MethodInfo { name: "sort", signature: "() -> NumberList", doc: None },
            MethodInfo { name: "sum", signature: "() -> Number", doc: None },
            MethodInfo { name: "avg", signature: "() -> Number", doc: None },
            MethodInfo { name: "min", signature: "() -> Number", doc: None },
            MethodInfo { name: "max", signature: "() -> Number", doc: None },
        ]),
        ("Object", vec![
            MethodInfo { name: "keys", signature: "() -> StringList", doc: Some("Get all keys") },
            MethodInfo { name: "values", signature: "() -> StringList | NumberList", doc: Some("Get all values (must be same type)") },
            MethodInfo { name: "length", signature: "() -> Number", doc: Some("Number of entries") },
            MethodInfo { name: "len", signature: "() -> Number", doc: None },
            MethodInfo { name: "contains", signature: "(key: String) -> Boolean", doc: Some("Check if key exists") },
            MethodInfo { name: "get", signature: "(key: String) -> any", doc: Some("Get value by key") },
        ]),
        ("StringList", vec![
            MethodInfo { name: "length", signature: "() -> Number", doc: None },
            MethodInfo { name: "len", signature: "() -> Number", doc: None },
            MethodInfo { name: "isEmpty", signature: "() -> Boolean", doc: None },
            MethodInfo { name: "first", signature: "() -> String", doc: None },
            MethodInfo { name: "last", signature: "() -> String", doc: None },
            MethodInfo { name: "get", signature: "(index: Number) -> String", doc: None },
            MethodInfo { name: "contains", signature: "(value: String) -> Boolean", doc: None },
            MethodInfo { name: "indexOf", signature: "(value: String) -> Number", doc: None },
            MethodInfo { name: "slice", signature: "(start: Number, end?: Number) -> StringList", doc: None },
            MethodInfo { name: "reverse", signature: "() -> StringList", doc: None },
            MethodInfo { name: "sort", signature: "() -> StringList", doc: None },
            MethodInfo { name: "join", signature: "(delim?: String) -> String", doc: None },
        ]),
    ]
}
