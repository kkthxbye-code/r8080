#[allow(dead_code)]

extern crate minifb;
extern crate byteorder;

mod ram;
mod opcode;
mod cpu;
mod util;

use cpu::Cpu;
use ram::Sram;

fn main() {
    let rom_path = String::from("invaders.rom");
    let mut ram: Sram = Sram::new();
    ram.load(&rom_path);

    let mut cpu: Cpu = Cpu::new(ram);
    
    cpu.run();
}