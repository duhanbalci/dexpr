# Compiler Modülü

**Konum:** `src/compiler.rs`

AST'yi bytecode'a dönüştürür. Tek geçişli (single-pass) derleme yapar. Tüm değişkenler global scope'tadır.

---

## Konfigürasyon

- `MAX_REGISTERS = 8` — Hesaplama için kullanılabilir register sayısı

---

## Hata Türleri (CompileError)

| Hata | Açıklama |
|------|----------|
| `UndefinedFunction(SmolStr)` | Tanımlanmamış fonksiyon (built-in dışı çağrı) |
| `RegisterLimitExceeded` | Register limiti aşıldı |
| `InvalidExpression(String)` | Geçersiz ifade |
| `InvalidStatement(String)` | Geçersiz deyim |
| `BytecodeError(String)` | Bytecode üretim hatası |

---

## Compiler Yapısı

```rust
struct Compiler {
    writer: BytecodeWriter,
    used_registers: Vec<bool>,

    // Jump address resolution
    pending_jumps: Vec<(usize, usize)>,
    labels: HashMap<usize, usize>,
    next_label: usize,

    // Debug info generation
    debug_info: DebugInfo,
    current_span: Span,
}
```

---

## Derleme Akışı

### Ana Derleme: `compile(statements) -> Vec<u8>`

Tek geçişli derleme süreci:

```
1. Deyimleri sırayla derle
2. End opcode'u yaz
3. Atlama adreslerini çözümle (resolve_jumps)
```

### Kaynak Koddan Derleme: `compile_from_source(source) -> (Vec<u8>, DebugInfo)`

Parse ile birlikte pozisyon bilgisi de toplar ve `DebugInfo` üretir.

---

## Deyim Derleme

### Assignment (Atama)

1. İfadeyi register'a derle
2. `StoreGlobal` emit et (tüm değişkenler global)

### If Statement (Koşullu Deyim)

```
1. Koşulu register'a derle
2. Else ve end için label oluştur
3. JumpIfFalse → else label
4. Then dalını derle
5. Jump → end label
6. Else label'ını set et, else dalını derle
7. End label'ını set et
```

---

## İfade Derleme

### Value (Sabit Değer)
- Register ayır → `LoadConst` emit et

### Variable (Değişken)
- Register ayır → `LoadGlobal` emit et

### BinaryOp (İkili Operasyon)
1. Sol operandı register'a derle
2. Sağ operandı register'a derle
3. Sonuç register'ı ayır
4. Uygun opcode'u emit et (Add, Sub, Mul, vs.)
5. Operand register'ları serbest bırak

> **Not:** String birleştirme derleme zamanında ayırt edilmez. `Op::Add` her zaman `OpCodeByte::Add` emit eder; string birleştirme ve otomatik tip dönüşümü VM tarafından çalışma zamanında (runtime) ele alınır.

### UnaryOp (Tekli Operasyon)
1. Operandı register'a derle
2. Sonuç register'ı ayır
3. `Neg` veya `Not` emit et

### FunctionCall (Fonksiyon Çağrısı)

- `log` built-in fonksiyonu: Argümanları derle → `Log` emit et → Null register döndür
- Diğer fonksiyonlar: `CallExternal` opcode emit edilir (VM tarafından runtime'da çözümlenir)

### ExprStmt (İfade Deyimi)
- İfadeyi derle → `SetResult` emit et → Register'ı serbest bırak
- `SetResult`, VM'in `execute()` dönüş değerini belirler (son ExprStmt kazanır)

### MethodCall (Metod Çağrısı)
- Nesneyi register'a derle
- Argümanları derle
- `MethodCall` emit et (sonuç, nesne, metod adı, argüman sayısı, argüman register'ları)

### PropertyAccess (Özellik Erişimi)
- Nesneyi register'a derle
- `GetProperty` emit et (hedef register, nesne register, özellik adı string)

### PropertyAssignment (Özellik Ataması)
- İç içe özellik zinciri (`a.b.c = expr`) için:
  1. Kök değişkeni `LoadGlobal` ile yükle
  2. Ara özellikler için `GetProperty` zinciri emit et
  3. Son özellik için `SetProperty` emit et
  4. Değiştirilmiş kök nesneyi `StoreGlobal` ile geri yaz

---

## Register Yönetimi

- `allocate_register() -> u8` — İlk boş register'ı bul, yoksa hata
- `free_register(reg)` — Register'ı kullanılabilir olarak işaretle
- Toplam 8 register limiti var

---

## Label ve Jump Yönetimi

| Metod | Açıklama |
|-------|----------|
| `create_label() -> usize` | Benzersiz label ID üret |
| `set_label(id)` | Label'ın bytecode pozisyonunu kaydet |
| `emit_jump_address(label) -> usize` | Placeholder yaz, çözümleme için kaydet |
| `emit_jump(label)` | Koşulsuz atlama emit et |
| `resolve_jumps()` | Tüm placeholder'ları gerçek adreslerle doldur |
