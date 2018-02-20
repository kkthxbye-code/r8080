#[allow(dead_code)]

extern crate minifb;
extern crate byteorder;

mod ram;
mod opcode;
mod cpu;
mod util;
mod registers;
mod instructions;

use cpu::Cpu;
use ram::Sram;

fn main() {
	space_invaders();
	//test_rom();
}


fn test_rom() {
    let rom_path = String::from("C:/TEST.COM");
    let mut ram: Sram = Sram::new();
    ram.load_offset(&rom_path, 0x100);

    let mut cpu: Cpu = Cpu::new(ram);
    cpu.move_pc(0x100);
    cpu.ram.write_byte(0x0005, 0xC9);
    
    cpu.run();
}

fn space_invaders() {
    let rom_path = String::from("invaders.rom");
    let mut ram: Sram = Sram::new();
    ram.load(&rom_path);

    let mut cpu: Cpu = Cpu::new(ram);
   
    cpu.run();
}