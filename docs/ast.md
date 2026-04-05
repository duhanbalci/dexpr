# AST (Abstract Syntax Tree) Modülü

**Konum:** `src/ast/`

AST modülü, parser tarafından üretilen ve compiler tarafından tüketilen ağaç yapısını tanımlar. Üç alt modülden oluşur: `expr.rs`, `stmt.rs`, `value.rs`.

---

## Span & Spanned

**Dosya:** `src/ast/expr.rs`

`Span`, kaynak koddaki bir konumu (satır, sütun) temsil eder. 1-indexed'tir.

```rust
struct Span { line: u32, column: u32 }
```

`Spanned<T>`, herhangi bir AST node'unu kaynak kod konumuyla eşleştirir:

```rust
struct Spanned<T> { node: T, span: Span }
```

`dummy()` metodu test ve internal kullanım için varsayılan konum oluşturur.

---

## Expr (İfadeler)

**Dosya:** `src/ast/expr.rs`  
**Type alias:** `SpannedExpr = Spanned<Expr>`

`Expr` enum'u tüm ifade türlerini kapsar:

| Variant | Açıklama | Örnek |
|---------|----------|-------|
| `Value(Value)` | Sabit değer (literal) | `42`, `"hello"`, `true` |
| `Variable(SmolStr)` | Değişken referansı | `x`, `result` |
| `BinaryOp(Box<Expr>, Op, Box<Expr>)` | İkili operasyon | `a + b`, `x > 5` |
| `UnaryOp(Op, Box<Expr>)` | Tekli operasyon | `-x`, `!flag` |
| `FunctionCall(SmolStr, Vec<Expr>)` | Built-in fonksiyon çağrısı | `log(x)` |
| `MethodCall(Box<Expr>, SmolStr, Vec<Expr>)` | Metod çağrısı | `name.upper()`, `text.split(",")` |
| `PropertyAccess(Box<Expr>, SmolStr)` | Özellik erişimi | `obj.field`, `person.name` |

> **Not:** `FunctionCall` hem built-in (`log`) hem de harici (host) fonksiyonlar için kullanılır. Built-in olmayan fonksiyonlar `CallExternal` opcode'u ile derlenir ve VM tarafından runtime'da çözümlenir.

---

## Op (Operatörler)

**Dosya:** `src/ast/expr.rs`

| Kategori | Operatörler |
|----------|-------------|
| Aritmetik | `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Pow` |
| Karşılaştırma | `Lt`, `Lte`, `Gt`, `Gte`, `Eq`, `Neq` |
| Boolean | `And`, `Or`, `Not` |
| Üyelik | `In` (değer listede/string'de var mı) |
| Tekli | `Neg` (negatif) |

---

## Stmt (İfadeler/Deyimler)

**Dosya:** `src/ast/stmt.rs`  
**Type alias:** `SpannedStmt = Spanned<Stmt>`

| Variant | Açıklama | Sözdizimi |
|---------|----------|-----------|
| `Assignment(SmolStr, Box<Expr>)` | Değişken ataması | `x = 5` |
| `ExprStmt(Box<Expr>)` | İfade deyimi | `log(x)` |
| `If(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>)` | Koşullu deyim | `if x > 0 then ... end` |
| `PropertyAssignment(SmolStr, Vec<SmolStr>, Box<Expr>)` | Özellik ataması | `a.b.c = expr` |

---

## Value (Çalışma Zamanı Değerleri)

**Dosya:** `src/ast/value.rs`

| Variant | Rust Tipi | Açıklama |
|---------|-----------|----------|
| `Null` | - | Boş değer |
| `Number(Decimal)` | `rust_decimal::Decimal` | Keyfi hassasiyetli ondalık sayı |
| `String(SmolStr)` | `smol_str::SmolStr` | Optimize edilmiş string |
| `Boolean(bool)` | `bool` | Mantıksal değer |
| `NumberList(Vec<Decimal>)` | `Vec<Decimal>` | Sayı listesi |
| `StringList(Vec<SmolStr>)` | `Vec<SmolStr>` | String listesi |
| `Object(IndexMap<SmolStr, Value>)` | `IndexMap<SmolStr, Value>` | Anahtar-değer nesnesi |

### Serileştirme

Her `Value` bytecode'a gömülebilir. Serileştirme formatı:

1. **Tip etiketi** (1 byte): `NULL=0x00`, `NUMBER=0x01`, `STRING=0x02`, `BOOLEAN=0x03`, `NUMBER_LIST=0x04`, `STRING_LIST=0x05`, `OBJECT=0x06`
2. **Veri:**
   - Number: 16 byte (Decimal serialization)
   - String: 2-byte uzunluk + UTF-8 bytes
   - Boolean: 1 byte (0 veya 1)
   - NumberList: 2-byte count + her sayı için 16 byte
   - StringList: 2-byte count + her string için (2-byte uzunluk + bytes)
   - Object: 2-byte entry count + her girdi için (anahtar: 2-byte uzunluk + bytes, değer: rekürsif serialize)

`serialize()` ve `deserialize()` metodları bu dönüşümü gerçekleştirir.

---

## Modüller Arası İlişki

```
Parser --> Expr, Stmt, Value (AST üretir)
Compiler <-- Expr, Stmt, Value (AST tüketir, bytecode üretir)
VM <-- Value (çalışma zamanında değer olarak kullanılır)
```
