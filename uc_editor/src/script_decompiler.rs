#![allow(dead_code)]

use crate::script_token::{ArrayToken, Token};
use l2_rw::ue2_rw::{UnrealReader, INDEX};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::io::{Read, Seek};
use strum::{Display, EnumIter};

pub struct BytecodeDecompiler<'a, R: Read + Seek> {
    size: usize,
    current_pos: usize,
    reader: &'a mut R,
}

impl<'a, R: Read + Seek> BytecodeDecompiler<'a, R> {
    pub fn deserialize_array_token(&mut self, token: ArrayToken) {
        match token {
            ArrayToken::ArrayElementToken | ArrayToken::DynamicArrayElementToken => {
                self.next(); // Key
                self.next(); // Array
            }
            ArrayToken::DynamicArrayLengthToken => {
                self.next(); // Array
            }

            ArrayToken::DynamicArrayFindStructToken
            | ArrayToken::DynamicArrayInsertItemToken
            | ArrayToken::DynamicArrayInsertToken
            | ArrayToken::DynamicArrayRemoveToken => {
                self.next(); // Array
                self.next(); // Param 1
                self.next(); // Param 2
            }

            ArrayToken::DynamicArraySortToken
            | ArrayToken::DynamicArrayRemoveItemToken
            | ArrayToken::DynamicArrayMethodToken
            | ArrayToken::DynamicArrayFindToken
            | ArrayToken::DynamicArrayAddItemToken => {
                self.next(); // Array
                self.next(); // Param 1
            }

            ArrayToken::DynamicArrayAddToken => {
                self.next(); // Array
                self.next(); // Param 1
                self.next(); // EndParms
            }
        }
    }

    pub fn new(reader: &'a mut R, size: usize) -> BytecodeDecompiler<'a, R> {
        Self {
            size,
            current_pos: 0,
            reader,
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.current_pos += 1;

        self.reader.read_unreal_value::<u8>()
    }

    fn read_short(&mut self) -> u16 {
        self.current_pos += 2;

        self.reader.read_unreal_value::<u16>()
    }

    fn read_index(&mut self) -> INDEX {
        self.current_pos += 4;

        self.reader.read_unreal_value::<INDEX>()
    }

    fn next(&mut self) -> Token {
        self.next_u(u8::MAX)
    }
    fn next_b(&mut self) -> Box<Token> {
        Box::new(self.next_u(u8::MAX))
    }

    fn next_u(&mut self, mut token_code: u8) -> Token {
        if token_code == u8::MAX {
            token_code = self.read_byte();
        }

        if token_code >= ExprToken::ExtendedNative.u8() {
            if token_code >= ExprToken::FirstNative.u8() {
                Token::NativeFunction {
                    index: token_code as u16,
                }
            } else {
                let ext = self.read_byte();

                let native_index =
                    (((token_code - ExprToken::ExtendedNative.u8()) as u16) << 8) | ext as u16;

                assert!(native_index < MAX_NATIVE);

                Token::NativeFunction {
                    index: native_index,
                }
            }
        } else {
            let expr = ExprToken::from_u8(token_code).unwrap();

            match expr {
                ExprToken::DynamicCast => Token::DynamicCast {
                    from: self.read_index(),
                    to: self.next_b(),
                },
                ExprToken::MetaCast => Token::MetaCast {
                    from: self.read_index(),
                    to: self.next_b(),
                },
                ExprToken::InterfaceCast => Token::InterfaceCast,
                ExprToken::PrimitiveCast => {
                    let cast_code = self.read_byte();

                    Token::PrimitiveCast {
                        from: self.next_b(),
                        to: CastToken::from_u8(cast_code).unwrap(),
                    }
                }

                ExprToken::ClassContext => Token::ClassContext {
                    start: self.next_b(),
                    size: self.read_short(),
                    property_type: self.read_byte(),
                    end: self.next_b(),
                },
                ExprToken::InterfaceContext => Token::InterfaceContext {
                    start: self.next_b(),
                },
                ExprToken::Context => Token::Context {
                    start: self.next_b(),
                    size: self.read_short(),
                    property_type: self.read_byte(),
                    end: self.next_b(),
                },

                ExprToken::StructMember => Token::StructMember {
                    meta_super_field: self.read_index(),
                    meta_next: self.read_index(),
                    member: self.next_b(),
                },

                ExprToken::Let => Token::Let {
                    left: self.next_b(),
                    right: self.next_b(),
                },
                ExprToken::LetBool => Token::LetBool {
                    left: self.next_b(),
                    right: self.next_b(),
                },
                ExprToken::LetDelegate => Token::LetDelegate {
                    left: self.next_b(),
                    right: self.next_b(),
                },
                ExprToken::Conditional | ExprToken::Eval => Token::Conditional {
                    condition: self.next_b(),
                    size_t: self.read_short(),
                    if_true: self.next_b(),
                    size_f: self.read_short(),
                    if_false: self.next_b(),
                },


                _ => unreachable!("unexpected token {:?}", expr),
            }
        }
    }

    pub fn decompile(&mut self) -> String {
        let mut res = "".to_string();

        while self.current_pos < self.size {
            self.next();
        }

        res
    }
}

trait OpsShortcut {
    fn u8(&self) -> u8;
}

impl<T: ToPrimitive> OpsShortcut for T {
    fn u8(&self) -> u8 {
        self.to_u8().unwrap()
    }
}

/// A collection of tokens describing an expression.
#[repr(u8)]
#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum ExprToken {
    // ValidateObject
    // ResizeString
    LocalVariable = 0x00,
    InstanceVariable = 0x01,
    DefaultVariable = 0x02, // default.Property
    /// UE1: ???
    /// UE2: Deprecated (Bad Expr Token)
    /// UE3: Introduced in a late UDK build.
    StateVariable = 0x03,
    Return = 0x04,    // return EXPRESSION
    Switch = 0x05,    // switch (CONDITION)
    Jump = 0x06,      // goto CODEOFFSET
    JumpIfNot = 0x07, // if( !CONDITION ) goto CODEOFFSET;
    Stop = 0x08,      // Stop (State)
    Assert = 0x09,    // assert (CONDITION)
    Case = 0x0A,      // case CONDITION:
    Nothing = 0x0B,
    LabelTable = 0x0C,
    GotoLabel = 0x0D,       // goto EXPRESSION
    EatReturnValue = 0x0E,  // Formerly known as EatString
    Let = 0x0F,             // A = B
    DynArrayElement = 0x10, // Array[EXPRESSION]
    New = 0x11,             // new(OUTER) CLASS...
    ClassContext = 0x12,    // Class'Path'.static.Function()
    MetaCast = 0x13,        // <CLASS>(CLASS)

    /// UE1: BeginFunction
    /// UE2: LetBool
    LetBool = 0x14,

    /// UE1: ???
    /// UE2: LineNumber (early UE2)?
    /// UE2X: Deprecated (Bad Expr Token)
    /// UE3: EndParmValue
    LineNumber = 0x15,
    EndFunctionParms = 0x16, // )
    Self_ = 0x17,            // Self (Renamed to avoid conflict with keyword)
    Skip = 0x18,
    Context = 0x19,         // A.B
    ArrayElement = 0x1A,    // A[x]
    VirtualFunction = 0x1B, // F(...)
    FinalFunction = 0x1C,   // F(...)
    IntConst = 0x1D,
    FloatConst = 0x1E,
    StringConst = 0x1F, // "String"
    ObjectConst = 0x20,
    NameConst = 0x21, // 'Name'
    RotationConst = 0x22,
    VectorConst = 0x23,
    ByteConst = 0x24,
    IntZero = 0x25,
    IntOne = 0x26,
    True = 0x27,
    False = 0x28,
    NativeParm = 0x29, // A (Native)
    NoObject = 0x2A,   // None
    /// UE1: A string size cast
    /// UE2: Deprecated (Bad Expr Token)
    CastStringSize = 0x2B,
    IntConstByte = 0x2C, // 0-9 (<= 255)
    BoolVariable = 0x2D, // B (Bool)
    DynamicCast = 0x2E,  // A(B)
    Iterator = 0x2F,     // ForEach
    IteratorPop = 0x30,  // Break (Implied/Explicit)
    IteratorNext = 0x31, // Continue (Implied/Explicit)
    StructCmpEq = 0x32,  // A == B
    StructCmpNE = 0x33,  // A != B
    // UnicodeStringConst
    UniStringConst = 0x34, // "UNICODE"

    // Note: These byte-codes have shifted since UE3 and have therefore incorrect values assigned.
    // FixedByteCodes
    /// UE1: ???
    /// UE2: RangeConst or Deprecated (Bad Expr Token)
    /// UE3: ???
    RangeConst = 0x35,
    StructMember = 0x36,   // Struct.Property
    DynArrayLength = 0x37, // ARRAY.Length
    GlobalFunction = 0x38, // Global.

    /// Redefined (RotatorToVector)
    ///
    /// UE1: RotatorToVector cast.
    /// UE2+: Followed by any of the CastTokens to free space for other tokens, most are unused from 0x39 to 0x3F.
    PrimitiveCast = 0x39, // TYPE(EXPRESSION)

    /// Redefined (DynArrayRemove)
    ///
    /// UE1: ByteToInt cast.
    /// UE2: ReturnNothing (Deprecated)
    /// UE3: ReturnNothing if previous token is a ReturnToken, DynArrayRemove when not.
    ReturnNothing = 0x3A,

    // UE2: ReturnNothing (Deprecated)
    DelegateCmpEq = 0x3B,
    DelegateCmpNE = 0x3C,
    DelegateFunctionCmpEq = 0x3D,
    DelegateFunctionCmpNE = 0x3E,
    NoDelegate = 0x3F,

    // FixedByteCodes
    DynArrayInsert = 0x40,
    DynArrayRemove = 0x41,
    DebugInfo = 0x42,
    DelegateFunction = 0x43,
    DelegateProperty = 0x44,
    LetDelegate = 0x45,
    /// UE3: An alternative to Else-If statements using A ? B : C;
    Conditional = 0x46, // CONDITION ? TRUE_LET : FALSE_LET
    /// Redefined (ObjectToBool, DynArrayFind)
    ///
    /// UE1: As an ObjectToBool cast.
    /// UE2: As an indicator of a function's end (unless preceded by PrimitiveCast then it is treated as an ObjectToBool).
    /// UE3: See DynArrayFind (See EndOfScript).
    ///
    /// Also, can be ARRAY.Find( EXPRESSION )
    FunctionEnd = 0x47,
    /// In some Unreal Engine 2 games, see Conditional for Unreal Engine 3.
    ///
    /// An alternative to Else-If statements using A ? B : C;.
    Eval = 0x48, // See Conditional
    /// UE3: Reference to a property with the Out modifier.
    OutVariable = 0x49,
    /// UE3: Default value of a parameter property.
    DefaultParmValue = 0x4A, // PARAMETER = EXPRESSION
    /// UE3: No parameter value was given e.g., Foo( Foo,, Foo );
    EmptyParmValue = 0x4B, // Empty argument, Call(Parm1,,Parm2)
    InstanceDelegate = 0x4C,
    VarInt = 0x4D,    // Found in Borderlands 2
    VarFloat = 0x4E,  // Found in Borderlands 2
    VarByte = 0x4F,   // Found in Borderlands 2
    VarBool = 0x50,   // Found in Borderlands 2
    VarObject = 0x51, // Found in Borderlands 2
    InterfaceContext = 0x52,
    InterfaceCast = 0x53,
    EndOfScript = 0x54,
    DynArrayAdd = 0x55,
    DynArrayAddItem = 0x56,
    DynArrayRemoveItem = 0x57,
    DynArrayInsertItem = 0x58,
    DynArrayIterator = 0x59,
    DynArraySort = 0x5A,
    FilterEditorOnly = 0x5B, // filtereditoronly { BLOCK }
    Unused5C = 0x5C,
    Unused5D = 0x5D,
    Unused5E = 0x5E,
    MaxNonNative = 0x5F,

    ExtendedNative = 0x60,
    FirstNative = 0x70,
}

const MAX_NATIVE: u16 = 0x1000;
const MAX_NON_NATIVE: u16 = 0x60 - 1; // ExtendedNative - 1
const INTERNAL_UNRESOLVED: u16 = MAX_NON_NATIVE;
const UNUSED: u16 = INTERNAL_UNRESOLVED;

/// A collection of tokens for casting operations.
#[repr(u8)]
#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum CastToken {
    None = 0x00,

    // UE3
    InterfaceToObject = 0x36,
    InterfaceToString = 0x37,
    InterfaceToBool = 0x38,

    RotatorToVector = 0x39, // Redefined
    ByteToInt = 0x3A,       // Redefined (ReturnNothing)
    ByteToBool = 0x3B,
    ByteToFloat = 0x3C,
    IntToByte = 0x3D,
    IntToBool = 0x3E,
    IntToFloat = 0x3F,
    BoolToByte = 0x40,  // Redefined
    BoolToInt = 0x41,   // Redefined
    BoolToFloat = 0x42, // Redefined
    FloatToByte = 0x43, // Redefined
    FloatToInt = 0x44,  // Redefined
    FloatToBool = 0x45, // Redefined
    ObjectToInterface = 0x46,
    ObjectToBool = 0x47, // Redefined
    NameToBool = 0x48,   // Redefined
    StringToByte = 0x49,
    StringToInt = 0x4A,
    StringToBool = 0x4B,
    StringToFloat = 0x4C,
    StringToVector = 0x4D,
    StringToRotator = 0x4E,
    VectorToBool = 0x4F,
    VectorToRotator = 0x50,
    RotatorToBool = 0x51,
    ByteToString = 0x52,
    IntToString = 0x53,
    BoolToString = 0x54,
    FloatToString = 0x55,
    ObjectToString = 0x56,
    NameToString = 0x57,
    VectorToString = 0x58,
    RotatorToString = 0x59,

    // UE3
    DelegateToString = 0x5A,
    StringToName = 0x60,
}

/// Debug information tokens.
#[repr(u8)]
#[derive(Display, Debug, EnumIter, Eq, PartialEq, Copy, Clone, FromPrimitive, ToPrimitive)]
enum DebugInfo {
    Let = 0x00,
    SimpleIf = 0x01,
    Switch = 0x02,
    While = 0x03,
    Assert = 0x04,
    Return = 0x10,
    ReturnNothing = 0x11,
    NewStack = 0x20,
    NewStackLatent = 0x21,
    NewStackLabel = 0x22,
    PrevStack = 0x30,
    PrevStackLatent = 0x31,
    PrevStackLabel = 0x32,
    PrevStackState = 0x33,
    EFP = 0x40,
    EFPOper = 0x41,
    EFPIter = 0x42,
    ForInit = 0x50,
    ForEval = 0x51,
    ForInc = 0x52,
    BreakLoop = 0x60,
    BreakFor = 0x61,
    BreakForEach = 0x62,
    BreakSwitch = 0x63,
    ContinueLoop = 0x70,
    ContinueForeach = 0x71,
    ContinueFor = 0x72,

    Unset = 0xFF,
}
