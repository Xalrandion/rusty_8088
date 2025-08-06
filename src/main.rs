use crate::decoder::decoder::decode_file;


mod decoder;
mod instruction;
mod octet_reader;
static FILE_PATH: &str = "/home/alexg/dev/rusty_8088/fixture/listing_0041_add_sub_cmp_jnz";


fn main() {
    let instructions = decode_file(FILE_PATH);

    println!("bits 16");
    instructions.iter().for_each(|it| println!("{}", it.to_string()));
}
