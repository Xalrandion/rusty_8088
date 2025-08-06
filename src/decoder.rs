pub mod decoder {
    use std::fs::File;

    use crate::{instruction::instruction::{AddressArg, Arg, ImmediateArg, ImmediateSize, Instruction, Mnemonics, RegArg, Registers, REGISTERS_VALUES}, octet_reader::octet_reader::OctetReader};
    

    const CONDITIONAL_JUMP_OP_TABLE: [(u8, &Mnemonics); 35] = [
        (0b01110100, &Mnemonics::JE),
        (0b01110100, &Mnemonics::JE),
        (0b01110100, &Mnemonics::JZ),
        (0b01111100, &Mnemonics::JL),
        (0b01111100, &Mnemonics::JNGE),
        (0b01111110, &Mnemonics::JLE),
        (0b01111110, &Mnemonics::JNG),
        (0b01110010, &Mnemonics::JB),
        (0b01110010, &Mnemonics::JNAE),
        (0b01110110, &Mnemonics::JBE),
        (0b01110110, &Mnemonics::JNA),
        (0b01111010, &Mnemonics::JP),
        (0b01111010, &Mnemonics::JPE),
        (0b01110000, &Mnemonics::JO),
        (0b01111000, &Mnemonics::JS),
        (0b01110101, &Mnemonics::JNE),
        (0b01110101, &Mnemonics::JNZ),
        (0b01111101, &Mnemonics::JNL),
        (0b01111101, &Mnemonics::JGE),
        (0b01111111, &Mnemonics::JNLE),
        (0b01111111, &Mnemonics::JG),
        (0b01110011, &Mnemonics::JNB),
        (0b01110011, &Mnemonics::JAE),
        (0b01110111, &Mnemonics::JNBE),
        (0b01110111, &Mnemonics::JA),
        (0b01111011, &Mnemonics::JNP),
        (0b01111011, &Mnemonics::JPO),
        (0b01110001, &Mnemonics::JNO),
        (0b01111001, &Mnemonics::JNS),
        (0b11100010, &Mnemonics::LOOP),
        (0b11100001, &Mnemonics::LOOPZ),
        (0b11100001, &Mnemonics::LOOPE),
        (0b11100000, &Mnemonics::LOOPNZ),
        (0b11100000, &Mnemonics::LOOPNE),
        (0b11100011, &Mnemonics::JCXZ),
        ];

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


        fn find_register(word: &u8, start_byte: u8, is_wide: bool) -> Registers {

            let clean_word = word.clone() << start_byte >> 5;
            return get_register(clean_word, is_wide);
        }

        fn get_register(registe_code: u8, is_wide: bool) -> Registers {
            return REGISTERS_VALUES[(registe_code as usize) + if is_wide { (Registers::BH as usize) +1 } else { 0 }].clone();
        }

        fn decode_non_reg_rm_field(r_m: u8, disp: &Option<u16>, is_mod_0: bool) -> Arg {
            let mut registers: Vec<Registers> = vec![];
            
            if r_m == 0b00000000 {
                registers.push(Registers::BX);
                registers.push(Registers::SI);
            }
            if r_m == 0b00000001 {
                registers.push(Registers::BX);
                registers.push(Registers::DI);
            }
            if r_m == 0b00000010 {
                registers.push(Registers::BP);
                registers.push(Registers::SI);
            }
            if r_m == 0b00000011 {
                registers.push(Registers::BP);
                registers.push(Registers::DI);
            }
            if r_m == 0b00000100 {
                registers.push(Registers::SI);
            }
            if r_m == 0b00000101 {
                registers.push(Registers::DI);
            }
            if r_m == 0b00000110 && !is_mod_0 {
                registers.push(Registers::BP);
            }
            if r_m == 0b00000111 {
                registers.push(Registers::BX);
            }

            let immediate_address = match disp {
                Some(it) => it,
                None => &0,
            };

            
            return  Arg::Address(AddressArg{immediate_address: *immediate_address, registers_to_add: registers, is_relative: false});
        }


        fn decode_imediate_to_register(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {

            let is_wide = word & 0b00001000 == 0b00001000;
            let target_register = find_register(&word, 5, is_wide);
            
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
            
            Ok(Instruction::new_full(Mnemonics::Mov, vec![
                Arg::Register(RegArg { register: target_register }),
                Arg::Immediate(ImmediateArg { value: data_value, immediate_size: None, is_signed: false, is_wide: is_wide })]))
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

        fn decode_arithmetic_op_mnemonic(field: u8) -> Result<Mnemonics, String> {

            return  match  field {
                0b00000000 => Ok(Mnemonics::Add),
                0b00000101 => Ok(Mnemonics::Sub),
                0b00000111 => Ok(Mnemonics::Cmp),
                _other => Err("Arithmetic op mnemonic not found".into())
            };
        }

        fn decode_conditional_jump_op_mnemonic(field: u8) -> Result<Mnemonics, String> {
            return match CONDITIONAL_JUMP_OP_TABLE.iter().find(|it| it.0 == field) {
                Some(it) => return Ok(it.1.clone()),
                None => Err("cannot find jump instruction".into())
            }
        }

        fn decode_data_transfer_op_mnemonic(_field: u8) -> Result<Mnemonics, String> {
            return Ok(Mnemonics::Mov)
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
                Ok(it) => Arg::Immediate(ImmediateArg { value: it.unwrap(), immediate_size, is_signed, is_wide }),
                Err(e) => return Err(e)
            };
            
            let r_m_field = if is_to_reg  {
                Arg::Register(RegArg { register: find_register(&word2, 5, is_wide)})
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
                Arg::Register(RegArg { register: find_register(&word2, 5, is_wide) })
            } else {
                decode_non_reg_rm_field(word2 << 5 >> 5, &disp_value, word2 >> 6 == MOD_FIELD_NO_DISP)
            };
            
            let reg_field = Arg::Register(RegArg { register: find_register(&word2, 2, is_wide) });

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
                Ok(it) => Arg::Address(AddressArg { immediate_address: it.unwrap(), registers_to_add: vec![], is_relative: true}),
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

            
            let acc_arg = Arg::Register(RegArg { register:  if is_wide { Registers::AX } else { Registers::AL }});
            let addrs_arg = Arg::Address(AddressArg { immediate_address: disp_value, registers_to_add: vec![], is_relative: false });

            let instr_args = if word & 0b10100000 == 0b10100000 {vec![acc_arg, addrs_arg] } else { vec![addrs_arg, acc_arg] };

            return Ok(Instruction::new_full(Mnemonics::Mov, instr_args))
        }


        fn decode_imm_to_acc(word: u8, reader: &mut OctetReader) -> Result<Instruction, String> {
            let is_wide = word & 0b00000001 == 0b00000001;
            
            let mnemonic_result = decode_arithmetic_op_mnemonic(word << 2 >> 5);

            let mnemonic = match mnemonic_result {
                Ok(it) => it, 
                Err(e) => return  Err(e)
            };

            let data_value = match calc_disp_value(reader, true, is_wide) {
                Ok(it) => Arg::Immediate(ImmediateArg { value: it.unwrap(), immediate_size: None, is_signed: false, is_wide: is_wide }),
                Err(e) => return Err(e)
            };

            
            let acc_arg = Arg::Register(RegArg { register: if is_wide { Registers::AX } else { Registers::AL } });
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

    
    pub fn decode_file(filepath: &str) -> Vec<Instruction> {
        let file = File::open(filepath);
        let mut reader = OctetReader::new(file.unwrap());

        let mut read_done = false;
        let read_done_borrow  = &mut read_done;
        let mut decoded_instructions = vec![];

        

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

            decoded_instructions.push(instruction);    
        }

        return decoded_instructions
    }
}