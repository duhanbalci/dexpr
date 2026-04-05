# Language Info Modülü

**Konum:** `src/language_info.rs`

Editör entegrasyonu için dil metadata'sı üretir. Built-in fonksiyonlar, tipe göre metodlar ve host-kayıtlı genişletmeleri JSON formatında dışa aktarır. Frontend editör kütüphanesi (`codemirror-lang-dexpr`) bu JSON'u alarak tip-bazlı autocomplete sağlar.

---

## Yapılar

### FunctionInfo

| Alan | Tip | Açıklama |
|------|-----|----------|
| `name` | `&'static str` | Fonksiyon adı |
| `signature` | `&'static str` | İmza (örn: `"(min, max) -> Number"`) |
| `doc` | `Option<&'static str>` | Opsiyonel açıklama |

### MethodInfo

| Alan | Tip | Açıklama |
|------|-----|----------|
| `name` | `&'static str` | Metod adı |
| `signature` | `&'static str` | İmza (örn: `"() -> String"`) |
| `doc` | `Option<&'static str>` | Opsiyonel açıklama |

### VariableInfo

| Alan | Tip | Açıklama |
|------|-----|----------|
| `name` | `String` | Değişken adı |
| `type_name` | `String` | Tip adı: `String`, `Number`, `Boolean`, `NumberList`, `StringList`, `Object` |
| `doc` | `Option<String>` | Opsiyonel açıklama |

### LanguageInfo

Tüm metadata'yı toplayan ana yapı.

| Alan | Tip | Açıklama |
|------|-----|----------|
| `functions` | `Vec<FunctionInfo>` | Fonksiyon listesi |
| `methods` | `Vec<(&'static str, Vec<MethodInfo>)>` | Tipe göre metod listesi |
| `variables` | `Vec<VariableInfo>` | Değişken listesi |

---

## Metodlar

### `LanguageInfo::builtin() -> Self`

Tüm built-in fonksiyon ve metodları içeren yeni bir `LanguageInfo` oluşturur.

**Built-in fonksiyonlar:** `log`, `rand`

**Built-in metodlar (tipe göre):**

| Tip | Metodlar |
|-----|----------|
| `String` | `upper`, `lower`, `trim`, `trimStart`, `trimEnd`, `split`, `replace`, `contains`, `startsWith`, `endsWith`, `length`, `charAt`, `substring` |
| `Number` | *(yok)* |
| `Boolean` | *(yok)* |
| `NumberList` | `length`, `len`, `isEmpty`, `first`, `last`, `get`, `contains`, `indexOf`, `slice`, `reverse`, `sort`, `sum`, `avg`, `min`, `max` |
| `StringList` | `length`, `len`, `isEmpty`, `first`, `last`, `get`, `contains`, `indexOf`, `slice`, `reverse`, `sort`, `join` |
| `Object` | `keys`, `values`, `length`, `len`, `contains`, `get` |

### `add_function(name, signature, doc)`

Host-kayıtlı fonksiyon ekler (VM'deki `register_function` ile eşleşir).

### `add_method(type_name, name, signature, doc)`

Host-kayıtlı metod ekler (VM'deki `register_method` ile eşleşir). Belirtilen tipe ait metod listesine eklenir.

### `add_variable(name, type_name, doc)`

External değişken ekler (VM'deki `set_global` ile eşleşir). Editörde autocomplete ve tip-bazlı metod önerileri için kullanılır.

### `to_json() -> String`

Tüm metadata'yı JSON formatında serileştirir. Frontend editör kütüphanesine gönderilecek çıktıyı üretir.

---

## JSON Formatı

`to_json()` çıktısı:

```json
{
  "functions": [
    {"name": "log", "signature": "(...args) -> null", "doc": "Print values to output"},
    {"name": "getRate", "signature": "(code: String) -> Number", "doc": "Get exchange rate"}
  ],
  "methods": {
    "String": [
      {"name": "upper", "signature": "() -> String"},
      {"name": "toTitleCase", "signature": "() -> String", "doc": "Custom host method"}
    ],
    "Number": [],
    "NumberList": [
      {"name": "sum", "signature": "() -> Number"}
    ],
    "StringList": [
      {"name": "join", "signature": "(delim?: String) -> String"}
    ]
  },
  "variables": [
    {"name": "price", "type": "Number"},
    {"name": "category", "type": "String", "doc": "Product category"}
  ]
}
```

---

## Kullanım Örneği

```rust
use dexpr::language_info::LanguageInfo;

// 1. Built-in metadata oluştur
let mut info = LanguageInfo::builtin();

// 2. Host fonksiyonları ekle (register_function ile eşleşmeli)
info.add_function("getRate", "(code: String) -> Number", Some("Get exchange rate"));
info.add_function("fetchPrice", "(sku: String) -> Number", None);

// 3. Host metodları ekle (register_method ile eşleşmeli)
info.add_method("String", "toTitleCase", "() -> String", None);

// 4. External değişkenleri ekle (set_global ile eşleşmeli)
info.add_variable("price", "Number", None);
info.add_variable("category", "String", Some("Ürün kategorisi".to_string()));
info.add_variable("items", "StringList", None);

// 5. JSON üret ve frontend'e gönder
let json = info.to_json();
// json -> HTTP response, WebSocket message, vb.
```

---

## Frontend Entegrasyonu

Üretilen JSON doğrudan `codemirror-lang-dexpr` kütüphanesine verilir:

```typescript
import { dexpr } from "codemirror-lang-dexpr";

// Rust'tan gelen JSON parse edilir
const languageInfo = JSON.parse(jsonFromRust);

const extensions = [basicSetup, dexpr(languageInfo)];
```

Detaylar için: [Editor Modülü](editor.md)
