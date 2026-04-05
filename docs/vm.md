# VM (Virtual Machine) Modülü

**Konum:** `src/vm/`

Register tabanlı sanal makine. Bytecode'u çalıştırır, 8 register ve global değişken deposu içerir.

---

## Alt Modüller

| Dosya | İçerik |
|-------|--------|
| `vm/mod.rs` | Modül export'ları |
| `vm/vm.rs` | Ana VM implementasyonu (core çalıştırma döngüsü) |
| `vm/methods.rs` | Metod dispatch (String, StringList, NumberList, Object metodları) |
| `vm/builtins.rs` | Built-in fonksiyon implementasyonları (abs, min, max, floor, ceil, round, sqrt, len, toString, toNumber) |
| `vm/error.rs` | Hata türleri (VMError) |
| `vm/debug_info.rs` | Bytecode offset → kaynak konum eşleştirme |

---

## VMError (Hata Türleri)

**Dosya:** `src/vm/error.rs`

| Hata | Açıklama |
|------|----------|
| `TypeMismatch { expected, got }` | Tip uyuşmazlığı |
| `UndefinedVariable(SmolStr)` | Tanımlanmamış değişken |
| `DivisionByZero` | Sıfıra bölme |
| `BytecodeError(String)` | Bozuk bytecode |
| `MethodNotFound { type_name, method }` | Metod bulunamadı |
| `RuntimeError(String)` | Genel çalışma zamanı hatası |
| `InvalidOperation { operation, left_type, right_type }` | Desteklenmeyen operasyon |
| `WithLocation { span, message }` | Kaynak konum bilgili hata |

`with_span(span)` metodu hatayı kaynak konum bilgisi ile sarar (çift sarmalamayı önler).

---

## DebugInfo

**Dosya:** `src/vm/debug_info.rs`

Bytecode offset'lerini kaynak kod konumlarına (Span) eşleştirir. Run-length encoded yapıda: her girdi bir sonraki girdiye kadar geçerlidir.

```rust
struct DebugInfo {
    entries: Vec<(u32, Span)>,  // sıralı (offset, span) çiftleri
}
```

**Metodlar:**
- `add_entry(offset, span)` — Eşleştirme ekle (artan offset sırasında)
- `get_span(offset) -> Option<Span>` — Binary search ile konum bul

---

## VM Yapısı

```rust
struct VM<'a> {
    // Bytecode durumu
    bytecode: &'a [u8],
    reader: BytecodeReader<'a>,
    pc: usize,

    // Hesaplama
    registers: [Value; 8],      // 8 register

    // Değişkenler
    globals: Map<SmolStr, Value, 64>, // Global kapsam (max 64 giriş)

    // Kaynaklar
    heap: Bump,                 // Bump allocator

    // Debug
    debug_info: Option<DebugInfo>,
    debug: bool,                // Debug çıktısı (debug build'lerde)
    opcode_counts: [usize; 256], // Profiling (debug build'lerde)
}
```

---

## Başlatma ve API

| Metod | Açıklama |
|-------|----------|
| `new(bytecode)` | VM oluştur |
| `set_debug_info(debug_info)` | Kaynak konum eşleştirmesi sağla |
| `set_global(name, value)` | Global değişken ata |
| `get_global(name) -> Option<&Value>` | Global değişken oku |
| `register_function(name, fn)` | Harici (host) fonksiyon kaydet |
| `register_method(type_name, method_name, fn)` | Tipe özel harici metod kaydet |
| `reset()` | Durumu sıfırla, yeniden çalıştırmaya hazırla |

---

## Çalıştırma Döngüsü

`execute() -> Result<Value, VMError>` (son expression'ın değerini döndürür):

1. VM durumunu sıfırla
2. Bytecode kaldığı sürece döngüde çalış:
   - Opcode byte'ını oku
   - İlgili handler'a yönlendir
   - Hataları kaynak konum bilgisi ile sar
   - Profiling verilerini güncelle (debug build)
   - `End` opcode'unda dur

---

## Opcode Handler'ları

### Register ve Bellek
- **`handle_load_const()`** — Bytecode'dan değer oku, register'a koy
- **`handle_move()`** — Register → register kopyala
- **`handle_load_global()`** — Global map → register
- **`handle_store_global()`** — Register → global map

### Aritmetik
- **`binary_op(f, name)`** — Genel handler: iki operand register'ı oku, fonksiyonu uygula, sonucu kaydet
- Sıfıra bölme kontrolü yapılır
- **`Add` opcode:** Sayısal toplama yanında string birleştirmeyi de destekler. Otomatik tip dönüşümü (auto-coercion) yapılır: String+String, String+Number, Number+String, String+Boolean kombinasyonları birleştirme olarak çalışır
- **`handle_neg()`** — Sadece Number tipinde tekli negatif

### Karşılaştırma
- **`compare_op(f, name)`** — Decimal değerler üzerinde karşılaştırma, Boolean döndürür

### Boolean
- **`handle_and()`**, **`handle_or()`**, **`handle_not()`** — Boolean register'lar üzerinde mantık operasyonları

### Kontrol Akışı
- **`handle_jump()`** — 4-byte adres oku, reader pozisyonunu ayarla
- **`handle_jump_if_false()`** — Register `Boolean(false)` ise atla

### String, Nesne ve Metodlar
- **`handle_concat()`** — İki register'ı birleştir (karışık tip dönüşümü destekler: String, Number, Boolean otomatik olarak String'e dönüştürülür)
- **`handle_get_property()`** — Object register'ından alan oku, alan yoksa `Null` döndür
- **`handle_set_property()`** — Object register'ında alan değerini ayarla
- **`handle_method_call()`** — Nesne register'ı, metod adı, argümanlar
  - **String metodları:** `upper`, `lower`, `trim`, `trimStart`, `trimEnd`, `split(delimiter)`, `replace(old, new)`, `startsWith(prefix)`, `endsWith(suffix)`, `contains(substr)`, `length`, `charAt(index)`, `substring(start, end?)`
  - **StringList metodları:** `length`/`len`, `isEmpty`, `first`, `last`, `get(index)`, `contains(value)`, `indexOf(value)`, `slice(start, end?)`, `reverse()`, `sort()`, `join(delimiter?)`
  - **NumberList metodları:** `length`/`len`, `isEmpty`, `first`, `last`, `get(index)`, `contains(value)`, `indexOf(value)`, `slice(start, end?)`, `reverse()`, `sort()`, `sum`, `avg`, `min`, `max`
  - **Object metodları:** `keys()`, `values()`, `length`/`len()`, `contains(key)`, `get(key)`
  - **Harici metodlar:** Yukarıdaki built-in metodlar bulunamazsa `external_methods` HashMap'inde aranır

### Üyelik Testi
- **`handle_contains()`** — `in` operatörü: String in StringList, Number in NumberList, String in String (substring), String in Object (anahtar varlığı kontrolü)

### Harici Fonksiyonlar ve Sonuç
- **`handle_call_external()`** — İsimle harici fonksiyon çağır (HashMap lookup)
- **`handle_set_result()`** — ExprStmt sonucunu `last_result`'a kaydet

### Built-in
- **`handle_log()`** — Register değerini stdout'a yazdır
- **`rand(min, max)`** — min ile max arasında rastgele tamsayı üret (varsayılan harici fonksiyon)
- **`abs(n)`** — Mutlak değer
- **`min(a, b, ...)`** — Verilen değerlerin minimumu
- **`max(a, b, ...)`** — Verilen değerlerin maksimumu
- **`floor(n)`** — Aşağı yuvarlama
- **`ceil(n)`** — Yukarı yuvarlama
- **`round(n[, places])`** — Yuvarlama (opsiyonel ondalık basamak sayısı)
- **`sqrt(n)`** — Karekök
- **`len(v)`** — Değerin uzunluğu (String, List, Object)
- **`toString(v)`** — Değeri String'e dönüştür
- **`toNumber(v)`** — Değeri Number'a dönüştür

> **Not:** Built-in fonksiyon implementasyonları `vm/builtins.rs` dosyasında, sabit ID tanımları `src/opcodes.rs` içindeki `default_fn` modülündedir.

---

## Yardımcı Metodlar

| Metod | Açıklama |
|-------|----------|
| `read_register_checked()` | Register oku ve doğrula |
| `validate_register()` | Register indeks sınır kontrolü |
| `read_jump_address()` | 4-byte adres oku ve sınır kontrolü yap |
| `set_position(addr)` | Reader pozisyonunu doğrulayarak ayarla |
| `wrap_error(err)` | Hataya kaynak konum bilgisi ekle |
| `debug_print_state()` | Debug çıktısı (debug build) |
| `print_profile_summary()` | Opcode çalışma sayıları (debug build) |

---

## Harici Fonksiyon ve Metod Kaydı

```rust
// Harici fonksiyon
vm.register_function("getRate", |args| {
    match &args[0] {
        Value::String(currency) => Ok(Value::Number(dec!(34.5))),
        _ => Err("expected string".to_string()),
    }
});

// Tipe özel harici metod
vm.register_method("Number", "format", |this, args| {
    // this: &Value, args: &[Value]
    Ok(Value::String("formatted".into()))
});
```

**Tip isimleri:** `"Number"`, `"String"`, `"Boolean"`, `"NumberList"`, `"StringList"`, `"Object"`, `"Null"`

---

## Çalışma Modeli Özeti

- **Register tabanlı:** 8 register ile hesaplama
- **Global depo:** SmolStr → Value map'i
- **Harici fonksiyonlar:** İsimle çözümlenen host fonksiyonları (HashMap lookup)
- **Harici metodlar:** Tipe özel host metodları
- **Expression return:** Son ExprStmt'ın değeri `execute()` dönüş değeri olarak verilir
- **Dinamik tip sistemi:** Tip uyuşmazlıklarında çalışma zamanı hatası
