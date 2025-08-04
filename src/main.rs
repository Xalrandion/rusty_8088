use std::{fs::File};

use octet_reader::octet_reader::OctetReader;
mod octet_reader;

static FILE_PATH: &str = "/home/alexg/dev/rusty_8088/fixture/listing_0041_add_sub_cmp_jnz";

static REG_CODE_BH: u8 = 0b00000111;

const REGISTER_TABLE: [&str; 16] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh", 
                                    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];

const CONDITIONAL_JUMP_OP_TABLE: [(u8, &str); 35] = [
(0b01110100, "JE"),
(0b01110100, "JE"),
(0b01110100, "JZ"),
(0b01111100, "JL"),
(0b01111100, "JNGE"),
(0b01111110, "JLE"),
(0b01111110, "JNG"),
(0b01110010, "JB"),
(0b01110010, "JNAE"),
(0b01110110, "JBE"),
(0b01110110, "JNA"),
(0b01111010, "JP"),
(0b01111010, "JPE"),
(0b01110000, "JO"),
(0b01111000, "JS"),
(0b01110101, "JNE"),
(0b01110101, "JNZ"),
(0b01111101, "JNL"),
(0b01111101, "JGE"),
(0b01111111, "JNLE"),
(0b01111111, "JG"),
(0b01110011, "JNB"),
(0b01110011, "JAE"),
(0b01110111, "JNBE"),
(0b01110111, "JA"),
(0b01111011, "JNP"),
(0b01111011, "JPO"),
(0b01110001, "JNO"),
(0b01111001, "JNS"),
(0b11100010, "LOOP"),
(0b11100001, "LOOPZ"),
(0b11100001, "LOOPE"),
(0b11100000, "LOOPNZ"),
(0b11100000, "LOOPNE"),
(0b11100011, "JCXZ"),
];
struct Instruction {
    mnemonic: String,
    arg: Vec<Arg>
}

#[derive(PartialEq)]
enum ImmediateSize {
    Byte,
    Word
}

enum DecoderType {

    ImmToMemReg,
    RegMemoryToEither,
    ImmediateToAcc,
    Jump,
    MemoryToAcc,
    ImmToReg
}

#[derive(PartialEq)]
enum MnemonicType {
    DataTransfer,
    Arithmetic,
    ControlTransfer
}


struct Arg {
    name: String,
    is_address_calc: bool,
    immediate_size: Option<ImmediateSize>,
    addrs_calc_param: Vec<String>
}

impl Instruction {
    fn new_full(mnemonic: String, arg: Vec<Arg>) -> Self {
        return Self { mnemonic: mnemonic, arg: arg};
    }
}

impl Arg {
    fn new_register(name: &str) -> Self {
        return  Self {name: name.into(), is_address_calc: false, addrs_calc_param: Vec::new(), immediate_size: None };
    }

    fn new_immediate(name: String, immediate_size: Option<ImmediateSize>) -> Self {
        return Self{ name: name, is_address_calc: false, addrs_calc_param: Vec::new(), immediate_size: immediate_size};
    }

    fn new_addrs_calc(calc_params: Vec<String>) -> Self {
        return  Self {name: String::new(), is_address_calc: true, addrs_calc_param: calc_params, immediate_size: None };
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        let arg_strs: Vec<String> = self.arg.iter().map(|it| it.to_string()).collect(); 
        return format!("{} {}", self.mnemonic,  arg_strs.join(", "));
    }
}

impl ToString for Arg {
    fn to_string(&self) -> String {
        let mut result = String::new();

        if self.immediate_size.is_some() {
            let imm_size_str = if *self.immediate_size.as_ref().unwrap() == ImmediateSize::Byte { "byte" } else { "word" };
            result += &*format!("{} ", imm_size_str)
        }
        if self.is_address_calc {
            result += &*format!("[{}]", self.addrs_calc_param.join(" + "));
            return result
        }
        result += &*self.name.clone();
        return result
    }
}


fn find_register_name(word: &u8, start_byte: u8, is_wide: bool) ->&str {

    let clean_word = word.clone() << start_byte >> 5;
    return get_register_name(clean_word, is_wide);
}

fn get_register_name(registe_code: u8, is_wide: bool)  -> &'static str { 
    return REGISTER_TABLE[usize::from( registe_code + (if is_wide { REG_CODE_BH +1 } else { 0 } ))]
}

fn decode_non_reg_rm_field(r_m: u8, disp: &Option<u16>, is_mod_0: bool) -> Arg {
    let mut calc_args: Vec<String> = vec![];
    
    if r_m == 0b00000000 {
        calc_args.push("bx".into());
        calc_args.push("si".into());
    }
    if r_m == 0b00000001 {
        calc_args.push("bx".into());
        calc_args.push("di".into());
    }
    if r_m == 0b00000010 {
        calc_args.push("bp".into());
        calc_args.push("si".into());
    }
    if r_m == 0b00000011 {
        calc_args.push("bp".into());
        calc_args.push("di".into());
    }
    if r_m == 0b00000100 {
        calc_args.push("si".into());
    }
    if r_m == 0b00000101 {
        calc_args.push("di".into());
    }
    if r_m == 0b00000110 && !is_mod_0 {
        calc_args.push("bp".into());
    }
    if r_m == 0b00000111 {
        calc_args.push("bx".into());
    }

    if disp.is_some() && disp.unwrap() != 0 {
        calc_args.push(disp.unwrap().to_string());
    }

    return  Arg::new_addrs_calc(calc_args);
}


fn decode_imediate_to_register(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

    let is_wide = word & 0b00001000 == 0b00001000;
    let target_register = find_register_name(&word, 5, is_wide);
    
    let mut data: [u8; 2] = [0, 0];
    data[0] =  match  reader.read_next() {
        Ok(b) => b,
        Err(_) => { return Err("Unexpected EOF during File read".into())} 
    };
    if is_wide {
        data[1] =  match  reader.read_next() {
            Ok(b) => b,
            Err(_) => { return Err("Unexpected EOF during File read".into())} 
        };  
    }

    data.swap(0, 1);
    let data_value = u16::from_be_bytes(data); 
    Ok(Instruction { mnemonic: "mov".into(), arg: vec![Arg::new_register(target_register), Arg::new_immediate(data_value.to_string(), None)] })
}

fn is_a_conditional_jump_op(word: u8) -> bool {
    return CONDITIONAL_JUMP_OP_TABLE.iter().any(|it| it.0 == word)
}

fn find_decoder_type_and_mnemonic_type(word: u8) -> Result<(DecoderType, MnemonicType), String> {

    if is_a_conditional_jump_op(word) {
        return Ok((DecoderType::Jump, MnemonicType::ControlTransfer))
    }

    if word & 0b10110000 == 0b10110000 {
        return Ok((DecoderType::ImmToReg, MnemonicType::DataTransfer));
    }  

    if word >> 2 == 0b00100000 {
        return Ok((DecoderType::ImmToMemReg, MnemonicType::Arithmetic))
    }

    if word >> 1 == 0b01100011 {
        return Ok((DecoderType::ImmToMemReg, MnemonicType::DataTransfer))
    }

    if word >> 6 == 0b00000000 && word << 5 >> 7 == 0b00000000 {
        return Ok((DecoderType::RegMemoryToEither, MnemonicType::Arithmetic))
    }

    if word >> 2 == 0b00100010 {
        return Ok((DecoderType::RegMemoryToEither, MnemonicType::DataTransfer));
    }

    if word & 0b10100000 == 0b10100000 || word & 0b10100000 == 0b10100000 {
        return Ok((DecoderType::MemoryToAcc, MnemonicType::DataTransfer));
    }   

    if word & 0b00000100 == 0b00000100  {
        return Ok((DecoderType::ImmediateToAcc, MnemonicType::Arithmetic));
    }

    Err("op code types have not been recegnised".into())
}

const MOD_FIELD_RM_IS_REG: u8  = 0b00000011;
const MOD_FIELD_NO_DISP: u8    = 0b00000000;
const MOD_FIELD_DISP_LOW: u8   = 0b00000001;
const MOD_FIELD_DISP_HIGH: u8  = 0b00000010;

fn calc_disp_status(word2: u8, rm_field: u8) -> (bool, bool) { // has disp low, has disp high

    if word2 & MOD_FIELD_RM_IS_REG == MOD_FIELD_RM_IS_REG {
        return (false, false)
    } 

    if word2 & MOD_FIELD_DISP_LOW == MOD_FIELD_DISP_LOW {
        return (true, false)
    }
    if word2 & MOD_FIELD_DISP_HIGH == MOD_FIELD_DISP_HIGH {
        return (true, true)
    }

    if word2 & MOD_FIELD_NO_DISP == MOD_FIELD_NO_DISP && (rm_field >> 1 & 0b00000011 == 0b00000011 && rm_field << 7 == 0b00000000) {
        return (true, true)
    }
    return (false, false)
}

fn calc_disp_value(reader: &mut OctetReader, does_have_disp_low: bool, does_have_disp_high: bool) -> Result<Option<u16>, String> {
    let mut disps: [u8; 2] = [0, 0];
    if does_have_disp_low {
        disps[0] =  match  reader.read_next() {
            Ok(b) => b,
            Err(_) => { return Err("Unexpected EOF during File read".into())} 
        }; 
    }
    if does_have_disp_low && does_have_disp_high {
        disps[1] =  match  reader.read_next() {
            Ok(b) => b,
            Err(_) => { return Err("Unexpected EOF during File read".into())} 
        };
    }
    disps.swap(0, 1);
    let disp_value = if does_have_disp_high || does_have_disp_low {Some(u16::from_be_bytes(disps))} else {None}; 
    return Ok(disp_value)
}

fn decode_arithmetic_op_mnemonic(field: u8) -> Result<String, String> {

    return  match  field {
        0b00000000 => Ok("add".into()),
        0b00000101 => Ok("sub".into()),
        0b00000111 => Ok("cmp".into()),
        _other => Err("Arithmetic op mnemonic not found".into())
    };
}

fn decode_conditional_jump_op_mnemonic(field: u8) -> Result<String, String> {
    return match CONDITIONAL_JUMP_OP_TABLE.iter().find(|it| it.0 == field) {
        Some(it) => return Ok(it.1.into()),
        None => Err("cannot find jump instruction".into())
    }
}

fn decode_data_transfer_op_mnemonic(_field: u8) -> Result<String, String> {
    return Ok("mov".into())
}

fn print_immedidate(number: u16, is_signed: bool) -> String {
    if !is_signed {
        return number.to_string()
    }
    let is_positive = number & 0b1000000000000000 == number & 0b1000000000000000;
    let signed_number: i16 = (number as i16) * if is_positive {1} else {-1};
    return signed_number.to_string() 
}

fn decode_immediate_to_mem_reg(mnemonic_type: MnemonicType, word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {
    
    let word2 =  match  reader.read_next() {
        Ok(b) => b,
        Err(_) => { return Err("Unexpected EOF during File read".into())} 
    };
    let is_wide = word & 0b00000001 == 0b00000001;
    let is_signed = if mnemonic_type == MnemonicType::Arithmetic {word & 0b00000010 == 0b00000010} else {false};
    let is_to_reg = word2 >> 6 == MOD_FIELD_RM_IS_REG;

    let (have_disp_low, have_disp_high) = calc_disp_status(word2 >> 6, word2 << 5 >> 5);
    
    let mnemonic_result = match mnemonic_type {
      MnemonicType::Arithmetic => decode_arithmetic_op_mnemonic(word2 << 2 >> 5),
      MnemonicType::DataTransfer => decode_data_transfer_op_mnemonic(word2 << 2 >> 5),
      _default=> return Err("Unsuported mnemonic type".into())
    };

    let mnemonic = match mnemonic_result {
        Ok(it) => it, 
        Err(e) => return  Err(e)
    };

    let disp_value = if mnemonic_type == MnemonicType::DataTransfer {None} else {  match calc_disp_value(reader, have_disp_low, have_disp_high) {
        Ok(it) => it,
        Err(e) => return Err(e)
    }};

    let immediate_size = if !is_to_reg { Some(if is_wide {ImmediateSize::Word} else {ImmediateSize::Byte}) } else {None};
    let data_value = match calc_disp_value(reader, true, is_wide && !is_signed) {
        Ok(it) => Arg::new_immediate(print_immedidate(it.unwrap(), is_signed), immediate_size),
        Err(e) => return Err(e)
    };

    

    let r_m_field = if is_to_reg  {
        Arg::new_register(find_register_name(&word2, 5, is_wide))
    } else {
        decode_non_reg_rm_field(word2 << 5 >> 5, &disp_value, word2 >> 6 == MOD_FIELD_NO_DISP)
    };
    return Ok(Instruction::new_full(mnemonic, vec![r_m_field, data_value]));
}

fn decode_reg_memory_to_either(mnemonic_type: MnemonicType, word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

    let word2 =  match  reader.read_next() {
        Ok(b) => b,
        Err(_) => { return Err("Unexpected EOF during File read".into())} 
    };
    let is_wide = word & 0b00000001 == 0b00000001;
    let reg_from = word & 0b00000010 == 0b00000010;
    let is_reg_to_reg = word2 >> 6 == MOD_FIELD_RM_IS_REG;
    let (have_disp_low, have_disp_high) = calc_disp_status(word2 >> 6, word2 << 5 >> 5);
    
    let mnemonic_result = match mnemonic_type {
      MnemonicType::Arithmetic => decode_arithmetic_op_mnemonic(word << 2 >> 5),
      MnemonicType::DataTransfer => decode_data_transfer_op_mnemonic(word << 2 >> 5),
      _default=> return Err("Unsuported mnemonic type".into())
    };

    let mnemonic = match mnemonic_result {
        Ok(it) => it, 
        Err(e) => return  Err(e)
    };

    let disp_value = match calc_disp_value(reader, have_disp_low, have_disp_high) {
        Ok(it) => it,
        Err(e) => return Err(e)
    };
    
    let r_m_field = if is_reg_to_reg  {
        Arg::new_register(find_register_name(&word2, 5, is_wide))
    } else {
        decode_non_reg_rm_field(word2 << 5 >> 5, &disp_value, word2 >> 6 == MOD_FIELD_NO_DISP)
    };
    let reg_field = Arg::new_register(find_register_name(&word2, 2, is_wide));

    let instr_args = if reg_from  { vec![reg_field, r_m_field] } else { vec![r_m_field, reg_field] };
    return Ok(Instruction::new_full(mnemonic, instr_args));
} 

fn decode_conditional_jump(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

    let mnemonic_result = decode_conditional_jump_op_mnemonic(word);

    let mnemonic = match mnemonic_result {
        Ok(it) => it, 
        Err(e) => return  Err(e)
    };

    let data_value = match calc_disp_value(reader, true, false) {
        Ok(it) => Arg::new_immediate(format!("{:#x}", it.unwrap()), None),
        Err(e) => return Err(e)
    };

    return Ok(Instruction::new_full(mnemonic, vec![data_value]))
}

fn decode_memory_to_acc(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {
 
    let is_wide = word & 0b00000001 == 0b00000001;

    let disp_value = match calc_disp_value(reader, true, true) {
        Ok(it) => it.unwrap(),
        Err(e) => return Err(e)
    };

    let acc_arg = Arg::new_register(if is_wide { "AX" } else { "AL" });
    let addrs_arg = Arg::new_immediate(format!("{:#x}", disp_value), None);

    let instr_args = if word & 0b10100000 == 0b10100000 {vec![acc_arg, addrs_arg] } else { vec![addrs_arg, acc_arg] };

    return Ok(Instruction::new_full("MOV".into(), instr_args))
}


fn decode_imm_to_acc(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {
    let is_wide = word & 0b00000001 == 0b00000001;
    
    let mnemonic_result = decode_arithmetic_op_mnemonic(word << 2 >> 5);

    let mnemonic = match mnemonic_result {
        Ok(it) => it, 
        Err(e) => return  Err(e)
    };

    let data_value = match calc_disp_value(reader, true, is_wide) {
        Ok(it) => Arg::new_immediate(it.unwrap().to_string(), None),
        Err(e) => return Err(e)
    };

    let acc_arg = Arg::new_register(if is_wide { "ax" } else { "al" });
    return Ok(Instruction::new_full(mnemonic, vec![acc_arg, data_value]));
}

fn decode(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

    let (decoder_type, mnemonic_type) = match find_decoder_type_and_mnemonic_type(word) {
        Ok(r) => r,
        Err(e )=> return Err(e),
    };

    let decode_result = match decoder_type {
        DecoderType::ImmToMemReg => decode_immediate_to_mem_reg(mnemonic_type, word, reader),
        DecoderType::ImmToReg => decode_imediate_to_register(word, reader),
        DecoderType::ImmediateToAcc => decode_imm_to_acc(word, reader),
        DecoderType::MemoryToAcc => decode_memory_to_acc(word, reader),
        DecoderType::RegMemoryToEither => decode_reg_memory_to_either(mnemonic_type, word, reader),
        DecoderType::Jump => decode_conditional_jump(word, reader),
    };
    return decode_result
}

fn main() {
    let file = File::open(FILE_PATH);
    let mut reader = OctetReader::new(file.unwrap());

    let mut read_done = false;
    let read_done_borrow  = &mut read_done;

    println!("bits 16");

    while !*read_done_borrow
    {
        let mut  word_bytes: [u8; 1] = [0];
        word_bytes[0] = match  reader.read_next() {
            Ok(b) => b,
            Err(_) => {*read_done_borrow = true; continue;} 
        };

        let instruction = match decode(word_bytes[0], &mut reader) {
            Ok(it) => it,
            Err(e) => {println!("Error during decode: {}", e); continue;}
        };

        println!("{}",  instruction.to_string())        
    }

}
