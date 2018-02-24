#[allow(dead_code)]

extern crate minifb;
extern crate byteorder;
#[macro_use] extern crate text_io;

mod ram;
mod opcode;
mod cpu;
mod util;
mod instructions;

use cpu::Cpu;
use ram::Sram;

fn main() {
	space_invaders();
	//test_rom();
    //baloon_bomber();
    //lunar_rescue();
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

fn baloon_bomber() {
    //let rom_path = String::from("bal.rom");
    let mut ram: Sram = Sram::new();
    
    ram.load_offset("C:\\ballbomb\\tn01", 0x0000);
    ram.load_offset("C:\\ballbomb\\tn02", 0x0800);
    ram.load_offset("C:\\ballbomb\\tn03", 0x1000);
    ram.load_offset("C:\\ballbomb\\tn04", 0x1800);
    ram.load_offset("C:\\ballbomb\\tn05-1", 0x4000);
    //ram.load(&rom_path);

    let mut cpu: Cpu = Cpu::new(ram);
   
    cpu.run();
}

fn lunar_rescue() {
    //let rom_path = String::from("bal.rom");
    let mut ram: Sram = Sram::new();
    
    ram.load_offset("C:\\lrescue\\lrescue.1", 0x0000);
    ram.load_offset("C:\\lrescue\\lrescue.2", 0x0800);
    ram.load_offset("C:\\lrescue\\lrescue.3", 0x1000);
    ram.load_offset("C:\\lrescue\\lrescue.4", 0x1800);
    ram.load_offset("C:\\lrescue\\lrescue.5", 0x4000);
    ram.load_offset("C:\\lrescue\\lrescue.6", 0x4800);

    //ram.load(&rom_path);

    let mut cpu: Cpu = Cpu::new(ram);
   
    cpu.run();
}
