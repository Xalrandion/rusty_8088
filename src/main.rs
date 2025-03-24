use std::{collections::HashMap, fs::File, io::{BufReader, Error, Read}};

use octet_reader::octet_reader::OctetReader;
mod octet_reader;


static FILE_PATH: &str = "/home/xalrandion/dev/rusty_8088/fixtures/listing_39";
static WORD_LENGHT: u64 = 16;
static INST_MOV_REG_TO_REG: u8 = 0b00100010;
static INST_MOV_IMMEDIATE_TO_REG: u8 = 0b00001011;

static REG_CODE_AX: u8 = 0b00000000;
static REG_CODE_CX: u8 = 0b00000001;
static REG_CODE_DX: u8 = 0b00000010;
static REG_CODE_BX: u8 = 0b00000011;
static REG_CODE_SP: u8 = 0b00000100;
static REG_CODE_BP: u8 = 0b00000101;
static REG_CODE_SI: u8 = 0b00000110;
static REG_CODE_DI: u8 = 0b00000111;

static REG_CODE_AL: u8 = 0b00000000;
static REG_CODE_CL: u8 = 0b00000001;
static REG_CODE_DL: u8 = 0b00000010;
static REG_CODE_BL: u8 = 0b00000011;
static REG_CODE_AH: u8 = 0b00000100;
static REG_CODE_CH: u8 = 0b00000101;
static REG_CODE_DH: u8 = 0b00000110;
static REG_CODE_BH: u8 = 0b00000111;

const REGISTER_TABLE: [&str; 16] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh", 
                                    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di"];

struct Instruction {
    mnemonic: String,
    arg: Vec<Arg>
}

struct Arg {
    name: String,
    is_address: bool,
    is_register: bool,
    is_address_calc: bool,
    
    addrs_calc_param: Vec<String>
}

impl Instruction {
    fn new(mnemonic: &str) -> Self {
        return Self { mnemonic: mnemonic.into(), arg: vec![]}
    }
}

impl Arg {
    fn new_register(name: &str) -> Self {
        return  Self {name: name.into(), is_register: true, is_address: false, is_address_calc: false, addrs_calc_param: Vec::new() };
    }

    fn new_immediate(name: String) -> Self {
        return Self{ name: name, is_register: false, is_address: false, is_address_calc: false, addrs_calc_param: Vec::new()};
    }

    fn new_addrs_calc(calc_params: Vec<String>) -> Self {
        return  Self {name: String::new(), is_register: false, is_address: false, is_address_calc: true, addrs_calc_param: calc_params };
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
        if self.is_address_calc {
            return format!("[{}]", self.addrs_calc_param.join(" + "));
        }
        return self.name.clone();
    }
}


fn find_register_name(word: &u8, start_byte: u8, is_wide: bool) ->&str {

    let clean_word = word.clone() << start_byte >> 5;
    return get_register_name(clean_word, is_wide);
}

fn get_register_name(registe_code: u8, is_wide: bool)  -> &'static str { 
    return REGISTER_TABLE[usize::from( registe_code + (if is_wide { REG_CODE_BH +1 } else { 0 } ))]
}

fn decode_non_reg_rm_field(r_m: u8, disp: &Option<u16>) -> Arg {
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
    if r_m == 0b00000110 {
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

fn decode_register_to_register(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

    let word2 =  match  reader.read_next() {
        Ok(b) => b,
        Err(_) => { return Err("Unexpected EOF during File read".into())} 
    };
    let reg_from = word & 0b00000010 == 0b00000010;
    let is_wide = word & 0b00000001 == 0b00000001;

    let is_reg_to_reg = word2 >> 6 == 0b00000011;
    let does_have_disp_low = word2 >> 6 == 0b00000001;
    let does_have_disp_high = word2 >> 6 == 0b00000010 || (word2 >> 6 == 0b00000000 && word2 << 4 >> 5 == 0b00000110);

    let mut disps: [u8; 2] = [0, 0];
    if does_have_disp_low || does_have_disp_high {
        disps[0] =  match  reader.read_next() {
            Ok(b) => b,
            Err(_) => { return Err("Unexpected EOF during File read".into())} 
        }; 
    }
    if does_have_disp_high {
        disps[1] =  match  reader.read_next() {
            Ok(b) => b,
            Err(_) => { return Err("Unexpected EOF during File read".into())} 
        };
    }
    disps.swap(0, 1);
    let disp_value = if does_have_disp_high || does_have_disp_low {Some(u16::from_be_bytes(disps))} else {None}; 

    let r_m_field = if is_reg_to_reg {
        Arg::new_register(find_register_name(&word2, 5, is_wide))
    } else {
        decode_non_reg_rm_field(word2 << 5 >> 5, &disp_value)
    };
    let reg_field= Arg::new_register(find_register_name(&word2, 2, is_wide));

    let mut res = Instruction::new("mov");
    
    res.arg.push(r_m_field);
    res.arg.push(reg_field);
    if reg_from {
      res.arg.swap(0, 1);  
    } 
    Ok(res)
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
    Ok(Instruction { mnemonic: "mov".into(), arg: vec![Arg::new_register(target_register), Arg::new_immediate(data_value.to_string())] })
}

fn decode(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {
    println!("{:b}", word);
    if word >> 2 == INST_MOV_REG_TO_REG {
      return decode_register_to_register(word, reader);
    }
    if word >> 4 == INST_MOV_IMMEDIATE_TO_REG {
      return  decode_imediate_to_register(word, reader);
    }
    Err("Unsuported command".into())
}

fn main() {
    let file = File::open(FILE_PATH);
    let mut reader = OctetReader::new(file.unwrap());

    let mut read_done = false;
    let read_done_borrow  = &mut read_done;

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
