use std::fs::File;
use std::io::Read;
use util::*;

pub const RAM_SIZE: usize = 64*1024;

pub struct Sram {
    pub bytes: Vec<u8>,
}

impl Sram {
    pub fn new() -> Sram {
        let bytes: Vec<u8> = vec![0; RAM_SIZE];
        
        Sram {
            bytes: bytes, 
        }
    }

    pub fn load_offset(&mut self, file_name: &str, offset: u16) {
        let mut f = File::open(&file_name).expect("Unable to open file");
        
        let mut rom_bytes: Vec<u8> = Vec::new();
        f.read_to_end(&mut rom_bytes).expect("Unable to read bytes");

        for (i, &item) in rom_bytes.iter().enumerate() {
            self.bytes[i+offset as usize] = item;
        }
    }

    pub fn load(&mut self, file_name: &str)
    {
        self.load_offset(file_name, 0x00);
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        return self.bytes[address as usize];
    }

    pub fn read_dword(&self, address: u16) -> u16 {        
        return u8_to_u16(self.bytes[address as usize], self.bytes[address as usize + 1]);
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }

    pub fn write_dword(&mut self, address: u16, value: u16) {
        let (upper, lower) = u16_to_u8(value);
        self.bytes[address as usize] = lower;
        self.bytes[address as usize + 1] = upper;
    }

    pub fn write_dword_stack(&mut self, address: u16, value: u16) {
        let (lower, upper) = u16_to_u8(value);
        self.bytes[address as usize] = upper;
        self.bytes[address as usize + 1] = lower;
    }
}