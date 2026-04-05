import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { dexpr, DexprLanguageInfo } from "./src/index";

// ── Simulate: this JSON comes from Rust's LanguageInfo::to_json() ──
// In a real app:
//   Rust:  info.add_value("customer", &Value::from_json(json)?, None);
//          let metadata = info.to_json();
//   HTTP:  GET /api/editor-metadata → metadata
//   JS:    const languageInfo = await fetch(...).then(r => r.json());
//
// Here we inline it for the demo:

const languageInfo: DexprLanguageInfo = {
  functions: [
    { name: "log", signature: "(...args) -> null", doc: "Print values to output" },
    { name: "rand", signature: "(min, max) -> Number", doc: "Random integer between min and max" },
    { name: "getRate", signature: "(code: String) -> Number", doc: "Get exchange rate for currency" },
  ],
  methods: {
    String: [
      { name: "upper", signature: "() -> String" },
      { name: "lower", signature: "() -> String" },
      { name: "trim", signature: "() -> String" },
      { name: "trimStart", signature: "() -> String" },
      { name: "trimEnd", signature: "() -> String" },
      { name: "split", signature: "(delim: String) -> StringList" },
      { name: "replace", signature: "(old: String, new: String) -> String" },
      { name: "contains", signature: "(substr: String) -> Boolean" },
      { name: "startsWith", signature: "(prefix: String) -> Boolean" },
      { name: "endsWith", signature: "(suffix: String) -> Boolean" },
      { name: "length", signature: "() -> Number" },
      { name: "charAt", signature: "(index: Number) -> String" },
      { name: "substring", signature: "(start: Number, end?: Number) -> String" },
    ],
    Number: [],
    Boolean: [],
    Object: [
      { name: "keys", signature: "() -> StringList", doc: "Get all keys" },
      { name: "values", signature: "() -> StringList | NumberList", doc: "Get all values (must be same type)" },
      { name: "length", signature: "() -> Number", doc: "Number of entries" },
      { name: "len", signature: "() -> Number" },
      { name: "contains", signature: "(key: String) -> Boolean", doc: "Check if key exists" },
      { name: "get", signature: "(key: String) -> any", doc: "Get value by key" },
    ],
    NumberList: [
      { name: "length", signature: "() -> Number" },
      { name: "len", signature: "() -> Number" },
      { name: "isEmpty", signature: "() -> Boolean" },
      { name: "first", signature: "() -> Number" },
      { name: "last", signature: "() -> Number" },
      { name: "get", signature: "(index: Number) -> Number" },
      { name: "contains", signature: "(value: Number) -> Boolean" },
      { name: "indexOf", signature: "(value: Number) -> Number" },
      { name: "slice", signature: "(start: Number, end?: Number) -> NumberList" },
      { name: "reverse", signature: "() -> NumberList" },
      { name: "sort", signature: "() -> NumberList" },
      { name: "sum", signature: "() -> Number" },
      { name: "avg", signature: "() -> Number" },
      { name: "min", signature: "() -> Number" },
      { name: "max", signature: "() -> Number" },
    ],
    StringList: [
      { name: "length", signature: "() -> Number" },
      { name: "len", signature: "() -> Number" },
      { name: "isEmpty", signature: "() -> Boolean" },
      { name: "first", signature: "() -> String" },
      { name: "last", signature: "() -> String" },
      { name: "get", signature: "(index: Number) -> String" },
      { name: "contains", signature: "(value: String) -> Boolean" },
      { name: "indexOf", signature: "(value: String) -> Number" },
      { name: "slice", signature: "(start: Number, end?: Number) -> StringList" },
      { name: "reverse", signature: "() -> StringList" },
      { name: "sort", signature: "() -> StringList" },
      { name: "join", signature: "(delim?: String) -> String" },
    ],
  },
  // ── Variables: as if Rust did info.add_value("customer", &Value::from_json(...), ...) ──
  variables: [
    // info.add_value("discount", &Value::Number(10), None)
    { name: "discount", type: "Number", doc: "Discount percentage" },
    // info.add_value("customer", &Value::from_json(r#"{"name":"Alice",...}"#)?, None)
    // → from_json auto-detects field types from the JSON values
    {
      name: "customer", type: "Object", doc: "Customer — from JSON via Value::from_json()",
      fields: [
        { name: "name", type: "String" },
        { name: "email", type: "String" },
        { name: "city", type: "String" },
        { name: "age", type: "Number" },
        { name: "active", type: "Boolean" },
        { name: "tags", type: "StringList" },
      ],
    },
    // info.add_value("order", &Value::from_json(r#"{"amount":1500,...}"#)?, None)
    {
      name: "order", type: "Object", doc: "Order — from JSON via Value::from_json()",
      fields: [
        { name: "amount", type: "Number" },
        { name: "currency", type: "String" },
        { name: "status", type: "String" },
        { name: "items", type: "StringList" },
      ],
    },
  ],
};

const sampleCode = `// dexpr demo
// customer and order come from JSON via Value::from_json()
// Rust side:
//   let customer = Value::from_json(api_response)?;
//   vm.set_global("customer", customer.clone());
//   info.add_value("customer", &customer, None);

// Property access — type-aware autocomplete
name = customer.name
city = customer.city

// Object fields in expressions
net = order.amount * (1 - discount / 100)

// Method chaining: obj.field → String methods
upper_name = customer.name.upper()
domain = customer.email.split("@").last()

// Object methods
fields = customer.keys()
fieldCount = customer.length()

// in operator — check key existence
if "email" in customer then
  log(customer.email)
end

// Property assignment
customer.city = "Ankara"

// List field methods
first_tag = customer.tags.first()
tag_count = customer.tags.length()

log(upper_name, net, first_tag)
`;

new EditorView({
  state: EditorState.create({
    doc: sampleCode,
    extensions: [basicSetup, dexpr(languageInfo)],
  }),
  parent: document.getElementById("editor")!,
});
