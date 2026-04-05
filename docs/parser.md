# Parser Modülü

**Konum:** `src/parser/`

Parser, kaynak kodu AST'ye dönüştürür. PEG (Parsing Expression Grammar) tabanlıdır ve `peg` crate'ini kullanır.

---

## Giriş Noktaları

**Dosya:** `src/parser/mod.rs`

| Fonksiyon | Dönüş | Açıklama |
|-----------|-------|----------|
| `program(source)` | `Vec<Stmt>` | Tam programı parse eder |
| `program_with_spans(source)` | `Vec<(usize, Stmt)>` | Byte offset'leriyle birlikte parse eder |

### `offset_to_span(source, offset) -> Span`
Byte offset'ini 1-indexed satır ve sütuna dönüştürür. UTF-8 karakter sınırlarını doğru şekilde hesaplar.

---

## Gramer Kuralları

**Dosya:** `src/parser/grammar.rs`

### Deyim (Statement) Ayrıştırma

| Kural | Ürettiği | Sözdizimi |
|-------|----------|-----------|
| `statement()` | `Stmt` | Herhangi bir geçerli deyim |
| `assignment()` | `Stmt::Assignment` | `x = expr` veya `x += expr` |
| `property_assignment()` | `Stmt::PropertyAssignment` | `a.b.c = expr` |
| `expr_stmt()` | `Stmt::ExprStmt` | Bağımsız ifade |
| `if_stmt()` | `Stmt::If` | `if cond then ... [else ...] end` |

**Bileşik atamalar:** `+=`, `-=`, `*=`, `/=`, `%=` operatörleri desugar edilir:
- `x += e` → `x = x + e`

**Özellik ataması:** `identifier ("." identifier)+ "=" expression` kalıbı ile nesne özelliklerine atama yapılır (örn: `a.b.c = 42`).

**Else zincirleri:** `else if` yapısı desteklenir ve rekürsif olarak parse edilir.

### İfade (Expression) Ayrıştırma - Öncelik Sırası

En düşükten en yükseğe:

1. Method çağrıları, fonksiyon çağrıları
2. Mantıksal AND (`&&`)
3. Mantıksal OR (`||`)
4. Karşılaştırma (`==`, `!=`, `<`, `<=`, `>`, `>=`, `in`)
5. Toplama/Çıkarma (`+`, `-`)
6. Çarpma/Bölme/Mod (`*`, `/`, `%`)
7. Üs alma (`**`) - sağdan birleşimli (right-associative)
8. Tekli operatörler (`-`, `!`), atomik ifadeler

### Postfix Kuralı

`postfix()` kuralı, herhangi bir atom üzerinde `.property` (özellik erişimi) ve `.method(args)` (metod çağrısı) zincirlemesini sağlar. Atom parse edildikten sonra ardışık `.identifier` veya `.identifier(args)` kalıpları uygulanır:

- `obj.field` → `Expr::PropertyAccess(obj, "field")`
- `obj.method(args)` → `Expr::MethodCall(obj, "method", args)`
- `obj.a.b.method()` → zincirleme erişim ve çağrı

### Atomik İfadeler

`atom()` kuralı temel ifade birimlerini parse eder:

| Tür | Örnek | Ürettiği |
|-----|-------|----------|
| Tanımlayıcı | `myVar` | `Expr::Variable` |
| Sayı | `42`, `3.14` | `Expr::Value(Number)` |
| String | `"hello"`, `'world'` | `Expr::Value(String)` |
| Boolean | `true`, `false` | `Expr::Value(Boolean)` |
| Parantezli ifade | `(a + b)` | İç ifade |
| Tekli negatif | `-x` | `Expr::UnaryOp(Neg, x)` |
| Tekli NOT | `!flag` | `Expr::UnaryOp(Not, flag)` |

### Literal Ayrıştırma

- **Sayılar:** Ondalık kısım opsiyonel (`42`, `3.14`)
- **Stringler:** Çift veya tek tırnak, escape sequence destekli
- **Tanımlayıcılar:** Ayrılmış kelimelerle çakışmamalı

### Ayrılmış Kelimeler

`if`, `then`, `else`, `end`, `true`, `false`, `in`

### Boşluk ve Yorumlar

- Boşluklar: space, tab, newline
- Satır yorumu: `// ...`
- Blok yorumu: `/* ... */`

---

## Hata Yönetimi

Parser hataları `peg` crate'inden gelir ve beklenen token/kural bilgisi içerir. `program_with_spans()` kullanıldığında byte offset'leri `offset_to_span()` ile satır/sütun bilgisine dönüştürülebilir.

---

## Kullanım Örneği

```rust
use dexpr::parser;

let source = r#"
    result = (3 + 4) * 2
    log(result)
"#;

let ast = parser::program(source).expect("Parse error");
// ast: Vec<Stmt> - Assignment, ExprStmt
```
