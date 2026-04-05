# Genel Mimari

dexpr, kaynak kodu parse edip bytecode'a derleyen ve register tabanlı bir VM üzerinde çalıştıran bir ifade değerlendirici (expression evaluator) ve bytecode VM'dir.

---

## Pipeline

```
Kaynak Kod (.dexpr)
       ↓
   [PARSER]  ← PEG tabanlı gramer kuralları
       ↓
   AST (Expr, Stmt, Value)
       ↓
  [COMPILER] ← Tek geçişli derleme
       ↓
   Bytecode (ham byte dizisi)
       ↓
     [VM]    ← 8 register, global'ler
       ↓
   Sonuç (veya kaynak konum bilgili hata)
```

---

## Temel Tasarım Kararları

1. **Register tabanlı VM:** Stack tabanlı VM'lere göre daha az komut, daha hızlı çalışma
2. **8 register limiti:** Basitlik ve hız arasında denge
3. **Tek geçişli derleme:** Fonksiyon desteği kaldırıldığı için iki geçişe gerek yok
4. **Label tabanlı atlamalar:** Son geçişte çözümlenir
5. **Span izleme:** Derleme ve çalışma zamanı hataları kaynak kod konumunu gösterir
6. **Value serileştirme:** Sabitler doğrudan bytecode'a gömülür
7. **Object tipi:** `IndexMap<SmolStr, Value>` tabanlı anahtar-değer nesneleri; özellik erişimi (`obj.field`), iç içe atama (`a.b.c = expr`) ve built-in metodlar (`keys`, `values`, `contains`, `get`, `length`) desteklenir
8. **Sadece global değişkenler:** Tüm değişkenler global scope'ta; host uygulamadan `set_global` ile değer aktarılır
9. **Bytecode cache:** Compile edilmiş bytecode saklanıp farklı global değerlerle tekrar çalıştırılabilir
10. **Harici fonksiyonlar:** Host fonksiyonları VM'de isimle kayıt edilir, runtime'da HashMap lookup ile çözümlenir
11. **Harici metodlar:** Tipe özel host metodları eklenebilir, built-in metodlar bulunamazsa kontrol edilir
12. **Expression return:** `execute()` son ExprStmt'ın değerini döndürür — tek satır formüller doğrudan sonuç verir

---

## Tipik Kullanım

```rust
use dexpr::{ast::value::Value, compiler::Compiler, parser, vm::VM};

// 1. Parse
let ast = parser::program(source_code)?;

// 2. Compile
let mut compiler = Compiler::new();
let bytecode = compiler.compile(ast)?;

// 3. Execute (bytecode cache'lenip tekrar kullanılabilir)
let mut vm = VM::new(&bytecode);
vm.set_global("input", Value::Number(dec!(42)));
vm.register_function("getRate", |args| { Ok(Value::Number(dec!(34.5))) });
let result = vm.execute()?; // Son expression'ın değerini döndürür
// veya: let output = vm.get_global("output");
```

---

## Bağımlılıklar

| Crate | Kullanım |
|-------|----------|
| `rust_decimal` | Keyfi hassasiyetli ondalık aritmetik |
| `peg` | PEG tabanlı parser üreteci |
| `smol_str` | Kısa string'ler için optimize edilmiş tip |
| `micromap` | Kompakt hashmap (global değişkenler için) |
| `bumpalo` | Bump allocator (hızlı bellek ayırma) |
| `thiserror` | Hata türleri için derive makrosu |
| `rustc-hash` | Hızlı hash fonksiyonu |
| `smallvec` | Stack-allocated vektörler |
| `indexmap` | Sıralı hashmap (Object tipi için) |

---

## Modül Referansları

- [AST Modülü](ast.md) — İfadeler, deyimler, değer türleri
- [Parser Modülü](parser.md) — Gramer kuralları, ayrıştırma
- [Opcodes Modülü](opcodes.md) — Bytecode komut seti
- [Bytecode Modülü](bytecode.md) — Serileştirme, disassembler
- [Compiler Modülü](compiler.md) — AST → bytecode dönüşümü
- [VM Modülü](vm.md) — Sanal makine, çalıştırma
- [Language Info Modülü](language_info.md) — Editör metadata üretimi
- [Editor Modülü](editor.md) — CodeMirror 6 dil desteği (codemirror-lang-dexpr)
