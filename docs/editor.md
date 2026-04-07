# Editor Modülü (codemirror-lang-dexpr)

**Konum:** `editor/`

CodeMirror 6 için dexpr dil desteği kütüphanesi. Syntax highlighting, tip-bazlı autocomplete ve error-tolerant parsing sağlar.

---

## Mimari

```
editor/
  src/
    dexpr.grammar     ← Lezer gramer dosyası (kaynak)
    parser.js         ← Lezer tarafından üretilen parser (generated)
    parser.terms.js   ← Token tanımları (generated)
    tokens.ts         ← External tokenizer (else if → tek token)
    language.ts       ← LRLanguage tanımı + syntax highlighting tag'leri
    highlight.ts      ← Varsayılan renk teması
    completions.ts    ← Autocomplete: tip çıkarımı + metadata bazlı öneriler
    index.ts          ← Ana export: dexpr() fonksiyonu
  demo.ts             ← Test sayfası kaynak kodu
  demo.html           ← Test sayfası
  dexpr.grammar       ← (src altındaki asıl dosya)
```

### İki Parser Stratejisi

| Taraf | Parser | Amaç |
|-------|--------|------|
| **Rust** (execution) | PEG (`peg` crate) | AST üretimi → derleme → VM çalıştırma |
| **Editor** (web) | Lezer | Syntax highlighting, error recovery, autocomplete |

Dil küçük olduğundan (7 keyword, ~20 operatör) iki gramer dosyasını sync tutmak kolaydır.

---

## Lezer Grammar

**Dosya:** `src/dexpr.grammar`

### Parse Kuralları

| Kural | Açıklama |
|-------|----------|
| `Program` | `statement*` — üst düzey kural |
| `IfStatement` | `if expr then stmts (else if expr then stmts)* (else stmts)? end` |
| `Assignment` | `VariableName AssignOp expression` veya `VariableName (.PropertyName)+ AssignOp expression` |
| `PropertyAccess` | `expression.PropertyName` — nesne özellik erişimi |
| `ExprStatement` | Bağımsız ifade |
| `BinaryExpression` | İkili operatörler, öncelik sırasıyla |
| `UnaryExpression` | Tekli `-` ve `!` |
| `MethodCall` | `expression.PropertyName(args)` |
| `FunctionCall` | `VariableName(args)` |
| `ParenExpression` | `(expression)` |

### Operatör Önceliği (düşükten yükseğe)

1. `||` — mantıksal OR
2. `&&` — mantıksal AND
3. `==`, `!=`, `<`, `<=`, `>`, `>=`, `in` — karşılaştırma
4. `+`, `-` — toplama/çıkarma
5. `*`, `/`, `%` — çarpma/bölme/mod
6. `**` — üs alma (sağdan birleşimli)
7. `-`, `!` — tekli operatörler (prefix)
8. `.method()` — metod çağrısı
9. `name()` — fonksiyon çağrısı

### External Tokenizer

**Dosya:** `src/tokens.ts`

`else if` iki ayrı keyword olarak yazılır ama Lezer'da tek token olarak tanınır (`elseIf`). External tokenizer `else` + boşluk + `if` dizisini tespit edip tek token üretir. Bu sayede `else if` zinciri ile nested `if` arasındaki belirsizlik (ambiguity) ortadan kalkar.

### Keyword Yönetimi

Keyword'ler `@extend` ile tanımlanır — `identifier` token'ından türetilir ama **her zaman** keyword olarak parse edilir. dexpr'de keyword'ler reserved'dır (`if`, `then`, `else`, `end`, `in`, `true`, `false`).

### Error Recovery

Lezer GLR parser kullanır. Bozuk/yarım kod yazılırken:
- Parse edilebilen kısımlar doğru tree node'ları üretir
- Hatalı kısımlar `⚠` (error) node'ları ile sarılır
- Editör yarım kodda bile syntax highlighting ve autocomplete sunabilir

---

## Syntax Highlighting

**Dosya:** `src/language.ts` (tag eşleştirme) + `src/highlight.ts` (renk teması)

### Token → Tag Eşleştirmesi

| Token | Lezer Tag | Varsayılan Renk |
|-------|-----------|-----------------|
| `if`, `then`, `else`, `end`, `in`, `elseIf` | `keyword` | `#7c3aed` (mor) |
| `true`, `false` | `bool` | `#d97706` (turuncu) |
| `"string"`, `'string'` | `string` | `#059669` (yeşil) |
| `42`, `3.14` | `number` | `#2563eb` (mavi) |
| `// comment`, `/* comment */` | `lineComment` / `blockComment` | `#9ca3af` (gri, italic) |
| `+`, `-`, `*`, `/`, `%`, `!`, `\|\|`, `&&` | `operator` | `#dc2626` (kırmızı) |
| `==`, `!=`, `<`, `<=`, `>`, `>=`, `=`, `**` | `compareOperator` | `#dc2626` (kırmızı) |
| `myVar` | `variableName` | `#1f2937` (koyu) |
| `.method` | `propertyName` | `#0891b2` (cyan) |
| `functionName()` | `function(variableName)` | `#9333ea` (mor) |

Host uygulama `highlighting: false` vererek varsayılan temayı devre dışı bırakıp kendi temasını kullanabilir.

---

## Autocomplete

**Dosya:** `src/completions.ts`

### Veri Kaynağı

Autocomplete verileri `DexprLanguageInfo` arayüzü ile sağlanır. Bu veri Rust tarafında `LanguageInfo::to_json()` ile üretilir. Detaylar: [Language Info Modülü](language_info.md)

```typescript
interface DexprLanguageInfo {
  functions: FunctionInfo[];
  methods: Partial<Record<DexprType, MethodInfo[]>>;
  variables?: VariableInfo[];
}
```

### Tip Çıkarımı (Type Inference)

Completions modülü Lezer syntax tree'sini kullanarak değişken tiplerini çıkarır:

1. **Config'den gelen tipler:** `variables` dizisindeki her değişkenin tipi bilinir
2. **Assignment analizi:** `x = "hello"` → `x: String`, `y = 42` → `y: Number`
3. **Method dönüş tipi:** `z = name.split(",")` → `z: StringList` (`split` dönüş tipi bilinir)
4. **Binary expression:** `a = x + y` → string varsa `String`, number varsa `Number`

Bu analiz her autocomplete tetiklendiğinde Lezer tree üzerinde yapılır. Bozuk kodda bile parse edilmiş assignment'lar doğru tip bilgisi verir.

### Tetikleme Kuralları

| Durum | Davranış |
|-------|----------|
| `identifier` yazılırken | Keyword, fonksiyon, değişken önerileri |
| `.` yazıldığında | Dot öncesi ifadenin tipine göre metod önerileri |
| `.` + tip bilinmiyor | Tüm metodlar (fallback) |
| `.` + `Number` tipi | Öneri yok (ondalık yazımıyla karışmaz) |
| String / comment içinde | Öneri yok |
| Ctrl+Space | Explicit tetikleme |

### Metod Önerileri (tipe göre)

| Dot Öncesi | Gösterilen Metodlar |
|------------|---------------------|
| `"hello".` | String metodları |
| `category.` (config'de `String`) | String metodları |
| `x.` (assignment'tan `String` çıkarıldı) | String metodları |
| `items.` (config'de `StringList`) | StringList metodları |
| `scores.` (config'de `NumberList`) | NumberList metodları |
| `obj.` (config'de `Object`) | Object field'ları + Object metodları |
| `kalemler.` (config'de `List`) | Element field'ları (property projection) + List metodları |
| `kalemler.tutar.` (List projection → `NumberList`) | NumberList metodları (sum, avg, min, max...) |
| `result.` (tip bilinmiyor) | Tüm metodlar |
| `42.` | Öneri yok |

---

## Kurulum ve Build

```bash
cd editor

# Bağımlılıkları kur
bun install

# Lezer parser'ı grammar'dan üret
npx lezer-generator src/dexpr.grammar -o src/parser.js

# Kütüphaneyi build et
bun run build       # → dist/index.js, dist/index.cjs, dist/index.d.ts

# Demo'yu build et (test için)
bun run demo        # → dist/demo.global.js

# Demo'yu çalıştır
bunx serve . -p 3456   # → http://localhost:3456/demo.html
```

### Grammar Değişikliği Yapıldığında

1. `src/dexpr.grammar` dosyasını düzenle
2. `npx lezer-generator src/dexpr.grammar -o src/parser.js` çalıştır
3. `bun run build` ile yeniden derle

---

## Host Uygulama Entegrasyonu

### 1. Rust Tarafı: Metadata Üretimi

```rust
use dexpr::language_info::LanguageInfo;

let mut info = LanguageInfo::builtin();

// VM'de register edilen her fonksiyon için:
info.add_function("getRate", "(code: String) -> Number", Some("Kur bilgisi"));

// VM'de register edilen her metod için:
info.add_method("String", "toTitleCase", "() -> String", None);

// VM'de set_global ile verilen her değişken için:
info.add_variable("price", "Number", None);
info.add_variable("category", "String", None);

let json = info.to_json();
```

### 2. Frontend Tarafı: Editör Oluşturma

```typescript
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { dexpr } from "codemirror-lang-dexpr";

// Rust'tan gelen JSON
const languageInfo = JSON.parse(jsonFromRust);

new EditorView({
  state: EditorState.create({
    doc: "",
    extensions: [basicSetup, dexpr(languageInfo)],
  }),
  parent: document.getElementById("editor")!,
});
```

### 3. Dinamik Güncelleme

Eğer host uygulama çalışma sırasında yeni fonksiyon/değişken eklerse, editörü yeni `languageInfo` ile yeniden oluşturmak gerekir. CodeMirror'un `EditorView.dispatch` ile extension'ları güncellemek mümkündür ama en basit yol editörü yeniden oluşturmaktır.

---

## Export'lar

### Ana Export

| Export | Tip | Açıklama |
|--------|-----|----------|
| `dexpr(config)` | `Extension` | All-in-one: language + autocomplete + highlighting |

### Granüler Export'lar

| Export | Açıklama |
|--------|----------|
| `dexprLanguage` | Sadece `LRLanguage` tanımı |
| `dexprCompletion(info)` | Sadece autocomplete extension'ı |
| `dexprHighlighting()` | Sadece varsayılan renk teması |
| `dexprHighlightStyle` | `HighlightStyle` nesnesi (özelleştirme için) |
| `KEYWORDS` | Keyword completion listesi |

### Tip Export'ları

| Tip | Açıklama |
|-----|----------|
| `DexprLanguageInfo` | Metadata arayüzü (JSON yapısı) |
| `DexprType` | `"String" \| "Number" \| "Boolean" \| "NumberList" \| "StringList" \| "Object" \| "List"` |
| `FunctionInfo` | Fonksiyon metadata'sı |
| `MethodInfo` | Metod metadata'sı |
| `VariableInfo` | Değişken metadata'sı |
