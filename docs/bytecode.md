# Bytecode Modülü

**Konum:** `src/bytecode.rs`

Bytecode'un serileştirilmesi (yazma) ve deserileştirilmesi (okuma) işlemlerini sağlar. Compiler bytecode üretirken `BytecodeWriter`'ı, VM çalıştırırken `BytecodeReader`'ı kullanır.

---

## BytecodeWriter

Compiler tarafından bytecode üretmek için kullanılır.

| Metod | Açıklama |
|-------|----------|
| `new()` | Boş writer oluştur |
| `write_byte(u8)` | Tek byte yaz |
| `write_u16(u16)` | Big-endian 16-bit tamsayı yaz |
| `write_u32(u32)` | Big-endian 32-bit tamsayı yaz |
| `write_register(u8)` | Register indeksi yaz |
| `write_string(SmolStr)` | String yaz (2-byte uzunluk + UTF-8) |
| `write_value(Value)` | Serileştirilmiş Value yaz |
| `position() -> usize` | Geçerli bytecode pozisyonu |
| `bytecode() -> &[u8]` | Bytecode'un immutable view'ı |
| `into_bytecode() -> Vec<u8>` | Writer'ı tüket, byte vektörü döndür |

---

## BytecodeReader

VM tarafından bytecode'u okumak için kullanılır.

| Metod | Açıklama |
|-------|----------|
| `new(bytecode)` | Bytecode'dan reader oluştur |
| `read_byte() -> Result<u8>` | Tek byte oku |
| `read_u16() -> Result<u16>` | Big-endian 16-bit tamsayı oku |
| `read_u32() -> Result<u32>` | Big-endian 32-bit tamsayı oku |
| `read_register() -> Result<u8>` | Register indeksi oku |
| `read_string() -> Result<SmolStr>` | String oku |
| `read_value() -> Result<Value>` | Value deserileştir ve oku |
| `position() -> usize` | Geçerli okuma pozisyonu |
| `set_position(usize)` | Pozisyona atla (bounds check ile) |
| `remaining() -> usize` | Kalan byte sayısı |

---

## Veri Formatı

Tüm çok-byte değerler **big-endian** formatında saklanır.

### String Formatı
```
[2 bytes: uzunluk (u16)] [N bytes: UTF-8 veri]
```

### Value Formatı
```
[1 byte: tip etiketi] [N bytes: tipe göre veri]
```

Detaylı Value serileştirme formatı için [AST dokümantasyonuna](ast.md#serileştirme) bakın.

---

## Bytecode Dump (Disassembler)

**Konum:** `src/bytecode_dump.rs`

`disassemble_bytecode(bytecode) -> Vec<String>` fonksiyonu bytecode'u insan okunabilir formata çevirir:

```
0000: LoadConst r0, 42
0008: StoreGlobal "x", r0
000d: LoadGlobal r0, "x"
0014: Log r0
0016: End
```

Bu araç debug ve geliştirme sürecinde bytecode'u incelemek için kullanılır.
