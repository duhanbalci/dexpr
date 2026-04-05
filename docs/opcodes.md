# Opcodes Modülü

**Konum:** `src/opcodes.rs`

Bytecode komut setini (instruction set) tanımlar. Her opcode bir `u8` değerine karşılık gelir.

---

## OpCodeByte Enum

### Register Operasyonları

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `LoadConst` | `0x10` | Sabit değeri register'a yükle |
| `Move` | `0x11` | Register'dan register'a kopyala |

### Bellek Operasyonları

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `LoadLocal` | `0x20` | Yerel değişkeni register'a yükle (ayrılmış) |
| `StoreLocal` | `0x21` | Register'ı yerel değişkene kaydet (ayrılmış) |
| `LoadGlobal` | `0x22` | Global değişkeni register'a yükle |
| `StoreGlobal` | `0x23` | Register'ı global değişkene kaydet |

> **Not:** `LoadLocal`/`StoreLocal` opcode'ları tanımlıdır ancak şu an compiler tarafından emit edilmez. Tüm değişkenler global scope'tadır.

### Aritmetik

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `Add` | `0x30` | Toplama |
| `Sub` | `0x31` | Çıkarma |
| `Mul` | `0x32` | Çarpma |
| `Div` | `0x33` | Bölme |
| `Neg` | `0x34` | Negatif (tekli) |
| `Mod` | `0x35` | Mod alma |
| `Pow` | `0x36` | Üs alma |

### Karşılaştırma

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `Lt` | `0x40` | Küçüktür |
| `Lte` | `0x41` | Küçük eşit |
| `Gt` | `0x42` | Büyüktür |
| `Gte` | `0x43` | Büyük eşit |
| `Eq` | `0x44` | Eşit |
| `Neq` | `0x45` | Eşit değil |

### Boolean Mantık

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `And` | `0x50` | Mantıksal VE |
| `Or` | `0x51` | Mantıksal VEYA |
| `Not` | `0x52` | Mantıksal DEĞİL |

### Kontrol Akışı

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `Jump` | `0x60` | Koşulsuz atlama |
| `JumpIfFalse` | `0x61` | Register false ise atla |

### Üyelik Testi

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `Contains` | `0x53` | Değer listede/string'de var mı kontrolü (`in` operatörü) |

### Özel Operasyonlar

| Opcode | Değer | Açıklama |
|--------|-------|----------|
| `Concat` | `0x80` | String birleştirme |
| `MethodCall` | `0x90` | Metod çağrısı |
| `GetProperty` | `0x91` | Nesne özelliği oku (dest, obj_reg, property_name_string) |
| `SetProperty` | `0x92` | Nesne özelliği yaz (obj_reg, property_name_string, val_reg) |
| `Log` | `0xA0` | Değer yazdır (built-in) |
| `CallExternal` | `0xA1` | Harici (host) fonksiyon çağrısı |
| `SetResult` | `0xB0` | İfade sonucunu kaydet (return value için) |
| `End` | `0xFF` | Program sonu |

---

## Hızlı Lookup Tablosu

`LOOKUP[256]` statik dizisi, O(1) karmaşıklıkta byte-to-opcode dönüşümü sağlar. `from_byte(u8)` metodu bu tabloyu kullanır.

**Metodlar:**
- `to_byte() -> u8` — Opcode'u byte'a dönüştür
- `from_byte(u8) -> Option<OpCodeByte>` — Byte'ı opcode'a dönüştür (lookup tablosu ile)
- `name() -> &str` — İnsan okunabilir opcode adı
