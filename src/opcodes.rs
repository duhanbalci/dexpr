/// Register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Register(pub u8);

/// Default (built-in) function IDs for CallDefault opcode
pub mod default_fn {
  pub const RAND: u8 = 0;
  pub const ABS: u8 = 1;
  pub const MIN: u8 = 2;
  pub const MAX: u8 = 3;
  pub const FLOOR: u8 = 4;
  pub const CEIL: u8 = 5;
  pub const ROUND: u8 = 6;
  pub const SQRT: u8 = 7;
  pub const LEN: u8 = 8;
  pub const TO_STRING: u8 = 9;
  pub const TO_NUMBER: u8 = 10;

  /// Lookup table: function name ��� ID
  pub const NAMES: &[(&str, u8)] = &[
    ("rand", RAND),
    ("abs", ABS),
    ("min", MIN),
    ("max", MAX),
    ("floor", FLOOR),
    ("ceil", CEIL),
    ("round", ROUND),
    ("sqrt", SQRT),
    ("len", LEN),
    ("toString", TO_STRING),
    ("toNumber", TO_NUMBER),
  ];

  /// Get function name by ID
  pub fn name(id: u8) -> Option<&'static str> {
    NAMES.iter().find(|(_, i)| *i == id).map(|(n, _)| *n)
  }

  /// Get function ID by name
  pub fn id(name: &str) -> Option<u8> {
    NAMES.iter().find(|(n, _)| *n == name).map(|(_, i)| *i)
  }
}

/// Bytecode opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCodeByte {
  // Register operations
  LoadConst = 0x10, // Load constant to register
  Move = 0x11,      // Move value between registers

  // Memory operations
  LoadLocal = 0x20,   // Load local variable to register
  StoreLocal = 0x21,  // Store register to local variable
  LoadGlobal = 0x22,  // Load global variable to register
  StoreGlobal = 0x23, // Store register to global variable

  // Arithmetic operations
  Add = 0x30, // Addition
  Sub = 0x31, // Subtraction
  Mul = 0x32, // Multiplication
  Div = 0x33, // Division
  Neg = 0x34, // Negation
  Mod = 0x35, // Modulo
  Pow = 0x36, // Power

  // Comparison operations
  Lt = 0x40,  // Less than
  Lte = 0x41, // Less than or equal
  Gt = 0x42,  // Greater than
  Gte = 0x43, // Greater than or equal
  Eq = 0x44,  // Equal
  Neq = 0x45, // Not equal

  // Boolean operations
  And = 0x50, // Logical AND
  Or = 0x51,  // Logical OR
  Not = 0x52, // Logical NOT

  // Membership test
  Contains = 0x53, // Check if value is in list/string

  // Control flow
  Jump = 0x60,        // Should read 4-byte address
  JumpIfFalse = 0x61, // Should read register + 4-byte address

  // String operations
  Concat = 0x80, // String concatenation

  // Property & method calls
  GetProperty = 0x91, // Get object property: dest, obj, name
  SetProperty = 0x92, // Set object property: obj, name, value
  MethodCall = 0x90,  // Call method on object

  // Built-in functions
  Log = 0xA0,          // Print a value
  CallExternal = 0xA1, // Call external (host) function
  CallDefault = 0xA2,  // Call default (built-in) function by ID

  // Result
  SetResult = 0xB0,   // Set expression result (for return value)
  ClearResult = 0xB1, // Clear expression result (assignment resets last result)

  // End marker
  End = 0xFF, // End of program
}

impl OpCodeByte {
  /// Convert opcode to byte
  pub fn to_byte(self) -> u8 {
    self as u8
  }

  /// Static lookup table for fast byte to opcode conversion
  const LOOKUP: [Option<OpCodeByte>; 256] = {
    let mut table = [None; 256];
    let mut i = 0;
    while i < 256 {
      table[i] = match i as u8 {
        0x10 => Some(OpCodeByte::LoadConst),
        0x11 => Some(OpCodeByte::Move),
        0x20 => Some(OpCodeByte::LoadLocal),
        0x21 => Some(OpCodeByte::StoreLocal),
        0x22 => Some(OpCodeByte::LoadGlobal),
        0x23 => Some(OpCodeByte::StoreGlobal),
        0x30 => Some(OpCodeByte::Add),
        0x31 => Some(OpCodeByte::Sub),
        0x32 => Some(OpCodeByte::Mul),
        0x33 => Some(OpCodeByte::Div),
        0x34 => Some(OpCodeByte::Neg),
        0x35 => Some(OpCodeByte::Mod),
        0x36 => Some(OpCodeByte::Pow),
        0x40 => Some(OpCodeByte::Lt),
        0x41 => Some(OpCodeByte::Lte),
        0x42 => Some(OpCodeByte::Gt),
        0x43 => Some(OpCodeByte::Gte),
        0x44 => Some(OpCodeByte::Eq),
        0x45 => Some(OpCodeByte::Neq),
        0x50 => Some(OpCodeByte::And),
        0x51 => Some(OpCodeByte::Or),
        0x52 => Some(OpCodeByte::Not),
        0x53 => Some(OpCodeByte::Contains),
        0x60 => Some(OpCodeByte::Jump),
        0x61 => Some(OpCodeByte::JumpIfFalse),
        0x80 => Some(OpCodeByte::Concat),
        0x90 => Some(OpCodeByte::MethodCall),
        0x91 => Some(OpCodeByte::GetProperty),
        0x92 => Some(OpCodeByte::SetProperty),
        0xA0 => Some(OpCodeByte::Log),
        0xA1 => Some(OpCodeByte::CallExternal),
        0xA2 => Some(OpCodeByte::CallDefault),
        0xB0 => Some(OpCodeByte::SetResult),
        0xB1 => Some(OpCodeByte::ClearResult),
        0xFF => Some(OpCodeByte::End),
        _ => None,
      };
      i += 1;
    }
    table
  };

  /// Convert byte to opcode
  #[inline(always)]
  pub fn from_byte(byte: u8) -> Option<Self> {
    Self::LOOKUP[byte as usize]
  }

  /// Get opcode name
  pub fn name(&self) -> &'static str {
    match self {
      OpCodeByte::LoadConst => "LoadConst",
      OpCodeByte::Move => "Move",
      OpCodeByte::LoadLocal => "LoadLocal",
      OpCodeByte::StoreLocal => "StoreLocal",
      OpCodeByte::LoadGlobal => "LoadGlobal",
      OpCodeByte::StoreGlobal => "StoreGlobal",
      OpCodeByte::Add => "Add",
      OpCodeByte::Sub => "Sub",
      OpCodeByte::Mul => "Mul",
      OpCodeByte::Div => "Div",
      OpCodeByte::Neg => "Neg",
      OpCodeByte::Mod => "Mod",
      OpCodeByte::Pow => "Pow",
      OpCodeByte::Lt => "Lt",
      OpCodeByte::Lte => "Lte",
      OpCodeByte::Gt => "Gt",
      OpCodeByte::Gte => "Gte",
      OpCodeByte::Eq => "Eq",
      OpCodeByte::Neq => "Neq",
      OpCodeByte::And => "And",
      OpCodeByte::Or => "Or",
      OpCodeByte::Not => "Not",
      OpCodeByte::Contains => "Contains",
      OpCodeByte::Jump => "Jump",
      OpCodeByte::JumpIfFalse => "JumpIfFalse",
      OpCodeByte::Concat => "Concat",
      OpCodeByte::MethodCall => "MethodCall",
      OpCodeByte::GetProperty => "GetProperty",
      OpCodeByte::SetProperty => "SetProperty",
      OpCodeByte::Log => "Log",
      OpCodeByte::CallExternal => "CallExternal",
      OpCodeByte::CallDefault => "Rand",
      OpCodeByte::SetResult => "SetResult",
      OpCodeByte::ClearResult => "ClearResult",
      OpCodeByte::End => "End",
    }
  }
}
