pub mod instruction {
    use std::fmt::format;



    #[derive(Debug, Clone)]
    pub enum Registers {
        AL = 0,
        CL = 1,
        DL = 2,
        BL = 3,
        AH = 4,
        CH = 5,
        DH = 6,
        BH = 7,
        AX = 8,
        CX = 9,
        DX = 10,
        BX = 11,
        SP = 12,
        BP = 13,
        SI = 14,
        DI = 15,
    }

    const REGISTERS_STRINGS: [&str; 16] = [
        "al",
        "cl",
        "dl",
        "bl",
        "ah",
        "ch",
        "dh",
        "bh",
        "ax",
        "cx",
        "dx",
        "bx",
        "sp",
        "bp",
        "si",
        "di",
    ];

    pub const REGISTERS_VALUES: [Registers; 16] = [
        Registers::AL,
        Registers::CL,
        Registers::DL,
        Registers::BL,
        Registers::AH,
        Registers::CH,
        Registers::DH,
        Registers::BH,
        Registers::AX,
        Registers::CX,
        Registers::DX,
        Registers::BX,
        Registers::SP,
        Registers::BP,
        Registers::SI,
        Registers::DI,
    ];

    #[derive(Debug, Clone)]
    pub enum Mnemonics {
        Mov = 0,
        Add = 1,
        Sub = 2,
        Cmp = 3,
        JE = 4,
        JZ = 5,
        JL = 6,
        JNGE = 7,
        JLE = 8,
        JNG = 9,
        JB = 10,
        JNAE = 11,
        JBE = 12,
        JNA = 13,
        JP = 14,
        JPE = 15,
        JO = 16,
        JS = 17,
        JNE = 18,
        JNZ = 19,
        JNL = 20,
        JGE = 21,
        JNLE = 22,
        JG = 23,
        JNB = 24,
        JAE = 25,
        JNBE = 26,
        JA = 27,
        JNP = 28,
        JPO = 29,
        JNO = 30,
        JNS = 31,
        LOOP = 32,
        LOOPZ = 33,
        LOOPE = 34,
        LOOPNZ = 35,
        LOOPNE = 36,
        JCXZ = 37,
    }

    const MNEMONICS_STRINGS: [&str; 38] = [
        "mov",
        "add",
        "sub",
        "cmp",
        "je",
        "jz",
        "jl",
        "jnge",
        "jle",
        "jng",
        "jb",
        "jnae",
        "jbe",
        "jna",
        "jp",
        "jpe",
        "jo",
        "js",
        "jne",
        "jnz",
        "jnl",
        "jge",
        "jnle",
        "jg",
        "jnb",
        "jae",
        "jnbe",
        "ja",
        "jnp",
        "jpo",
        "jno",
        "jns",
        "loop",
        "loopz",
        "loope",
        "loopnz",
        "loopne",
        "jcxz",
    ];

    pub struct Instruction {
        mnemonic: Mnemonics,
        arg: Vec<Arg>
    }

    #[derive(PartialEq)]
    pub enum ImmediateSize {
        Byte,
        Word
    }
    
    pub struct ImmediateArg { 
        pub value: u16,
        pub immediate_size: Option<ImmediateSize>,
        pub is_signed: bool,
        pub is_wide: bool,
    }

    pub struct RegArg {
        pub register: Registers
    }

    pub struct AddressArg {
        pub immediate_address: u16,
        pub registers_to_add: Vec<Registers>,
        pub is_relative: bool
    }

    pub enum Arg {
        Immediate (ImmediateArg),
        Register(RegArg),
        Address(AddressArg)
    }
    
    impl Instruction {
        pub fn new_full(mnemonic: Mnemonics, arg: Vec<Arg>) -> Self {
            return Self { mnemonic: mnemonic, arg: arg};
        }
    }
    
    impl ToString for Instruction {
        fn to_string(&self) -> String {
            let arg_strs: Vec<String> = self.arg.iter().map(|it| it.to_string()).collect(); 
            return format!("{} {}", MNEMONICS_STRINGS[self.mnemonic.clone() as usize],  arg_strs.join(", "));
        }
    }
    
    impl Arg {
        
        fn calculate_signed_value(&self, value: u16, is_signed: bool, is_wide: bool) -> i32 {
            
            return if !is_signed {
                value as i32
            } else {
                let is_positive = if is_wide { value & 0b1000000000000000 == 0b1000000000000000 } else { value & 0b0000000010000000 == 0b0000000010000000 };
                let signed_number: i16 = (value as i16) * if !is_positive {1} else {-1};
                signed_number as i32
            }
             
        }
    }

    impl ToString for Arg {
        fn to_string(&self) -> String {
            return match self {
                Arg::Immediate(immediate_arg) => {
                    let immediate_clause = match &immediate_arg.immediate_size {
                        Some(it) => if *it == ImmediateSize::Byte { format!("{} ", "byte") } else {format!("{} ", "word") },
                        None => "".into(),
                    };
                    let value_clause = self.calculate_signed_value(immediate_arg.value, immediate_arg.is_signed, immediate_arg.is_wide) as u16;
                    format!("{}{}", immediate_clause, value_clause)
                },
                Arg::Register(reg_arg) => REGISTERS_STRINGS[reg_arg.register.clone() as usize].into(),
                Arg::Address(address_arg) => {
                    let mut args: Vec<String> = address_arg.registers_to_add.iter().map(|it| REGISTERS_STRINGS[it.clone() as usize].into()).collect();
                    let is_only_imm_address = args.is_empty();
                    if address_arg.immediate_address != 0 {
                        if is_only_imm_address {
                            args.push(format!("{:#x}", self.calculate_signed_value(address_arg.immediate_address, address_arg.is_relative, true) as i16));
                        } else {
                            args.push(format!("{}", self.calculate_signed_value(address_arg.immediate_address, address_arg.is_relative, true) as i16));
                        }
                    }
                    format!("[{}]", args.join(" + "))
                    
                },
            };
        }
    }
}
