# dexpr Embedding Guide

dexpr'i başka projelere embed etmenin iki yolu var:

1. **Rust backend + Web frontend** — Rust tarafında çalıştırma, frontend'de editör
2. **Tamamen tarayıcıda (WASM)** — Hem çalıştırma hem editör tarayıcıda

---

## Yol 1: Rust Backend + Web Frontend

Tipik senaryo: kullanıcı editörde formül yazar, backend derler ve çalıştırır.

### Rust tarafı

```toml
# Cargo.toml
[dependencies]
dexpr = { path = "../dexpr" }  # veya git/crates.io
rust_decimal_macros = "1.40"
indexmap = "2"
smol_str = "0.3"
```

```rust
use dexpr::{ast::value::Value, compiler::Compiler, vm::VM, language_info::LanguageInfo};

// ── 1. Verileri hazırla ──
// JSON'dan (API, DB, config, vb.)
let customer = Value::from_json(r#"{
    "name": "Alice",
    "email": "alice@test.com",
    "age": 30,
    "active": true,
    "tags": ["premium", "tr"]
}"#).unwrap();

let order = Value::from_json(r#"{
    "amount": 1500,
    "currency": "TRY",
    "items": ["laptop", "mouse"]
}"#).unwrap();

// ── 2. Editör metadata'sını üret (sayfa yüklenirken, 1 kez) ──
let mut info = LanguageInfo::builtin();

// Değişkenler — tip ve field bilgisi Value'dan otomatik çıkar
info.add_value("customer", &customer, Some("Müşteri bilgisi".into()));
info.add_value("order", &order, Some("Sipariş bilgisi".into()));
info.add_value("discount", &Value::Number(rust_decimal_macros::dec!(10)), None);

// Host fonksiyonları
info.add_function("getRate", "(code: String) -> Number", Some("Döviz kuru"));
info.add_function("fetchPrice", "(sku: String) -> Number", None);

let editor_json = info.to_json();
// → bu JSON'ı bir endpoint ile frontend'e gönder

// ── 3. Kullanıcının yazdığı kodu çalıştır ──
fn execute_dexpr(
    source: &str,
    globals: &[(&str, Value)],
) -> Result<Value, String> {
    let mut compiler = Compiler::new();
    let (bytecode, debug_info) = compiler
        .compile_from_source(source)
        .map_err(|e| e.to_string())?;

    let mut vm = VM::new(&bytecode);
    vm.set_debug_info(&debug_info);

    for (name, value) in globals {
        vm.set_global(name, value.clone());
    }

    // Host fonksiyonları kaydet
    vm.register_function("getRate", |args| {
        // args[0]: currency code
        Ok(Value::Number(rust_decimal_macros::dec!(34.5)))
    });

    vm.execute().map_err(|e| e.to_string())
}

// Kullanıcının formülü:
let result = execute_dexpr(
    "order.amount * (1 - discount / 100)",
    &[
        ("customer", customer),
        ("order", order),
        ("discount", Value::Number(rust_decimal_macros::dec!(10))),
    ],
);
// → Ok(Number(1350))
```

### Frontend tarafı

```bash
npm install codemirror codemirror-lang-dexpr
```

```typescript
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { dexpr } from "codemirror-lang-dexpr";

// ── 1. Backend'den metadata al ──
const langInfo = await fetch("/api/editor-metadata").then(r => r.json());

// ── 2. Editörü oluştur ──
const editor = new EditorView({
  state: EditorState.create({
    doc: "",
    extensions: [basicSetup, dexpr(langInfo)],
  }),
  parent: document.getElementById("editor")!,
});

// ── 3. Çalıştır ──
async function run() {
  const code = editor.state.doc.toString();
  const res = await fetch("/api/execute", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code }),
  });
  const { result, error } = await res.json();
  document.getElementById("output")!.textContent =
    error ?? JSON.stringify(result);
}
```

### Akış

```
Frontend                          Backend (Rust)
───────                          ──────────────
GET /editor-metadata ──────────► LanguageInfo::to_json()
◄─────────────────────────────── JSON (functions, methods, variables+fields)

dexpr(langInfo)
  customer.  →  [name, email, age, ...]
  customer.name.  →  [upper, lower, trim, ...]

POST /execute { code } ────────► Compiler::compile_from_source()
                                  VM::new() + set_global() + execute()
◄─────────────────────────────── { result: 1350 }
```

---

## Yol 2: Tamamen Tarayıcıda (WASM)

Backend'e gerek yok. Parser, compiler, VM hepsi tarayıcıda çalışır.

### Build

```bash
cd wasm
wasm-pack build --target web --release
# → wasm/pkg/ altında JS + WASM + TypeScript types üretilir
```

`pkg/` içeriği:
- `dexpr_wasm_bg.wasm` (~370KB) — WASM binary
- `dexpr_wasm.js` — JS glue code (auto-init)
- `dexpr_wasm.d.ts` — TypeScript type definitions

### Kullanım

```html
<div id="editor"></div>
<button onclick="run()">Çalıştır</button>
<pre id="output"></pre>

<script type="module">
import init, { DexprEngine } from "./pkg/dexpr_wasm.js";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { dexpr } from "codemirror-lang-dexpr";

// ── 1. WASM'ı başlat ──
await init();
const engine = new DexprEngine();

// ── 2. Global'leri JSON string olarak ver ──
engine.setGlobal("customer", JSON.stringify({
  name: "Alice",
  email: "alice@test.com",
  age: 30,
  active: true,
  tags: ["premium", "tr"],
}));

engine.setGlobal("order", JSON.stringify({
  amount: 1500,
  currency: "TRY",
  items: ["laptop", "mouse"],
}));

engine.setGlobal("discount", "10");

// ── 3. Host fonksiyon kaydet ──
engine.registerFunction("getRate", (argsJson) => {
  const args = JSON.parse(argsJson);
  const code = args[0]; // currency code string
  const rates = { USD: 34.5, EUR: 37.2 };
  return JSON.stringify(rates[code] ?? 0);
});
engine.addFunctionInfo("getRate", "(code: String) -> Number", "Döviz kuru");

// ── 4. Editör metadata'sını engine'den al ──
// setGlobal çağrıları otomatik olarak field tiplerini algılar
const langInfo = JSON.parse(engine.languageInfo());

// ── 5. CodeMirror editörünü oluştur ──
const editor = new EditorView({
  state: EditorState.create({
    doc: `// customer ve order WASM engine'e JSON olarak verildi
name = customer.name.upper()
total = order.amount * (1 - discount / 100)
log(name, total)
total`,
    extensions: [basicSetup, dexpr(langInfo)],
  }),
  parent: document.getElementById("editor"),
});

// ── 6. Çalıştır butonu ──
window.run = function() {
  try {
    const code = editor.state.doc.toString();
    const resultJson = engine.execute(code);
    const result = JSON.parse(resultJson);
    document.getElementById("output").textContent = JSON.stringify(result, null, 2);

    // Global'ler güncellendi mi kontrol et
    const customer = JSON.parse(engine.getGlobal("customer"));
    console.log("customer after:", customer);
  } catch (e) {
    document.getElementById("output").textContent = "Error: " + e.message;
  }
};
</script>
```

### API Referansı

```typescript
class DexprEngine {
  constructor()

  // Global değişken: JSON string olarak ver/al
  setGlobal(name: string, json: string): void
  getGlobal(name: string): string | undefined

  // Çalıştır: kaynak kodu → sonuç JSON string
  execute(source: string): string

  // Host fonksiyonu kaydet (args ve return JSON string olarak)
  registerFunction(name: string, fn: (argsJson: string) => string): void

  // Editör metadata (otomatik güncellenir her setGlobal'de)
  languageInfo(): string

  // Editör autocomplete için fonksiyon bilgisi
  addFunctionInfo(name: string, signature: string, doc?: string): void
}
```

### Dikkat Edilmesi Gerekenler

- **JSON string convention**: `setGlobal` ve `execute` JSON *string* alır/verir — `JSON.stringify()` ve `JSON.parse()` kullan
- **Global sync**: `execute()` sonrası property assignment'lar (`customer.city = "Ankara"`) global'lere yansır — `getGlobal()` ile güncel halini al
- **Host fonksiyonlar**: Args ve return JSON string — `(argsJson: string) => string`
- **Editör metadata**: `setGlobal()` her çağrıldığında `languageInfo()` otomatik güncellenir (field tipleri Value'dan çıkarılır)
- **WASM boyutu**: Release build ~370KB (gzip ile ~120KB)

---

## Karşılaştırma

| | Rust Backend | WASM |
|---|---|---|
| **Çalıştırma** | Server'da | Tarayıcıda |
| **Performans** | Native hız | ~2-3x native |
| **Güvenlik** | Server-side validation | Client-side (güvenilir ortam) |
| **Latency** | Network roundtrip | Anında |
| **Deployment** | Backend deploy | Static dosya |
| **Host fonksiyonlar** | Rust closure'ları | JS callback (JSON bridge) |
| **DB/API erişimi** | Doğrudan | Fetch ile |

**Öneri**: Güvenlik kritikse (fiyat hesaplama, kurallar) → Rust backend. İnteraktif preview/sandbox → WASM.
