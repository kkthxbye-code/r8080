use std::fmt;
use ram::Sram;
use util::*;
use opcode::Opcode;

use instructions::*;

use std::{thread, time};
use minifb::{Key, WindowOptions, Window};
use byteorder::{BigEndian, ReadBytesExt};

const REG_BC: u8 = 0;
const REG_DE: u8 = 1;
const REG_HL: u8 = 2;
const REG_AF: u8 = 3;

const REG_A: u8 = 7;
const REG_M: u8 = 6;

const FLAG_C: u8 = 1 << 0;
const FLAG_P: u8 = 1 << 2;
const FLAG_AC: u8 = 1 << 4;
const FLAG_INT: u8 = 1 << 5;
const FLAG_Z: u8 = 1 << 6;
const FLAG_S: u8 = 1 << 7;

const INT_END: u16 = 0x08;
const INT_MID: u16 = 0x10;

const WIDTH: usize = 224;
const HEIGHT: usize = 256;

#[allow(dead_code)]
pub struct Cpu {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16,

    pub ram: Sram,

    pub cycles: u32,
    pub instruction_count: u64,

    pub current_opcode: u8,
    pub last_interrupt: u16,
    pub last_interrupt_time: time::Instant,

    pub window: Window,
    
    pub port4hi: u8,
    pub port4lo: u8,
    pub port2: u8,
    pub inp1: u8,
    pub inp2: u8,
    pub port3o: u8,
    pub port5o: u8,

    pub interrupt_in_progress: bool,
}

impl Cpu {
    pub fn new(ram: Sram) -> Cpu {
        let window = Window::new("Space Invaders",
                                 WIDTH,
                                 HEIGHT,
                                 WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

        Cpu {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,

            sp: 0x2400,
            pc: 0x0000,

            ram: ram,
            cycles: 0,
            instruction_count: 0,
            
            current_opcode: 0x00,

            last_interrupt: INT_MID,
            last_interrupt_time: time::Instant::now(),

            window: window,

            port4hi: 0x00,
            port4lo: 0x00,
            port2: 0x00,
            inp1: 0x00,
            inp2: 0x00,
            port3o: 0x00,
            port5o: 0x00,

            interrupt_in_progress: false,
        }
    }
}

impl Cpu {
    pub fn run(&mut self) {
        loop {
            self.check_interrupt();

            let opcode = Opcode::new(self.ram.read_byte(self.pc));

            /*
            if opcode.opcode == 0x76 {
                println!("HALT at {:#06x}", self.pc);
            }

            if self.pc == 0x0005 {
                if self.c == 9 {
                    let addr = self.read_dword(REG_DE);

                    for x in 0..10 {
                        println!("{:?}", self.ram.read_byte(addr+x) as char);
                    }
                }

                if self.c == 2 {
                    println!("{:?}", self.e as char);
                }
            }
            */

            self.current_opcode = opcode.opcode;

            self.pc += 1;

            self.run_instruction(opcode);
            self.instruction_count += 1;
            
            //println!("{:?}", self);
        }
    }
}

// Read/Write register methods and utility
impl Cpu {
    pub fn read_byte(&self, index: u8) -> u8 {
        match index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => self.ram.read_byte(self.read_dword(REG_HL)),
            7 => self.a,
            _ => panic!("Unknown index for read_byte"), 
        }
    }

    pub fn read_stack(&self) -> u16 {
        let value = u8_to_u16(self.ram.read_byte(self.sp + 1), self.ram.read_byte(self.sp));

        value
    }


    pub fn pop_stack(&mut self) -> u16 {
        let value = u8_to_u16(self.ram.read_byte(self.sp), self.ram.read_byte(self.sp+1));
        self.sp += 2;

        value
    }

    pub fn push_stack(&mut self, value: u16) {
        self.sp -= 2;
        self.ram.write_dword_stack(self.sp, value);
    }


    pub fn read_dword(&self, index: u8) -> u16 {
        match index {
            0 => u8_to_u16(self.c, self.b),
            1 => u8_to_u16(self.e, self.d),
            2 => u8_to_u16(self.l, self.h),
            3 => self.sp,
            4 => u8_to_u16(self.f, self.a),
            _ => panic!("Unknown index for read_dword()")
        }
    }

    pub fn write_byte(&mut self, index: u8, value: u8) {
        match index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => {
                let hl: u16 = self.read_dword(REG_HL);
                self.ram.write_byte(hl, value)
            },
            7 => self.a = value,
            _ => panic!("Unknown index for read_byte"), 
        }
    }

    pub fn write_dword(&mut self, index: u8, value: u16) {
        match index {
            0 => {
                let (upper, lower) = u16_to_u8(value);
                self.b = upper;
                self.c = lower;
            },
            1 => {
                let (upper, lower) = u16_to_u8(value);
                self.d = upper;
                self.e = lower;
            },
            2 => {
                let (upper, lower) = u16_to_u8(value);

                self.h = upper;
                self.l = lower;
            },
            3 => {
                self.sp = value;
            },
            4 => {
                let (upper, lower) = u16_to_u8(value);
                self.a = upper;
                self.f = lower;
            },
            _ => panic!("Unknwon index for read_dword()")
        }
    }

    pub fn read_im_byte(&mut self) -> u8 {
        let im = self.ram.read_byte(self.pc);
        self.pc += 1;

        im
    }

    pub fn read_im_dword(&mut self) -> u16 {
        let im = self.ram.read_dword(self.pc);
        self.pc += 2;

        im
    }

    pub fn set_flags(&mut self, mask: u8, initial: u16, result: u16) {
        if (mask & FLAG_Z) != 0 {
            if result as u8 == 0 {
                self.f |= FLAG_Z;
            } else {
                self.f &= !FLAG_Z;
            }
        }

        if (mask & FLAG_C) != 0 {
            if result > 255 {
                self.f |= FLAG_C;
            } else {
                self.f &= !FLAG_C;
            }
        }

        if (mask & FLAG_P) != 0 {
            if is_even_parity(result as u8) {
                self.f |= FLAG_P;
            } else {
                self.f &= !FLAG_P;
            }
        }

        if (mask & FLAG_S) != 0 {
            if result & 0x80 != 0 {
                self.f |= FLAG_S;
            } else {
                self.f &= !FLAG_S;
            }
        }

        if (mask & FLAG_AC) != 0 {
            if (initial & 0xf) > (result & 0xf) {
                self.f |= FLAG_AC;
            } else {
                self.f &= !FLAG_AC;
            }
        }
    }

    pub fn read_flag(&mut self, flag: u8) -> bool {
        let res = match flag {
            FLAG_AC => self.f & FLAG_AC,
            FLAG_C => self.f & FLAG_C,
            FLAG_Z => self.f & FLAG_Z,
            FLAG_P => self.f & FLAG_P,
            FLAG_S => self.f & FLAG_S,
            FLAG_INT => self.f & FLAG_INT,
            _ => panic!("Unknown Flag"),
        };

        if res != 0 {
            true
        } else { 
            false 
        }
    }

    pub fn move_pc(&mut self, address: u16) {
        self.pc = address;
    }

    pub fn check_interrupt(&mut self) {
        let now = time::Instant::now();
        let elapsed = now.duration_since(self.last_interrupt_time);
        let nanos = elapsed.subsec_nanos() as u64;

        let elapsed_nanos = elapsed.as_secs() + nanos;
        let needed: u64 = 1000000000/120;

        if elapsed_nanos < needed {
            let sleep_period = (needed - elapsed_nanos) / 1_000_000;
            let sleep_duration = time::Duration::from_millis(sleep_period);

            thread::sleep(sleep_duration);
        }

        if self.cycles > 16667 {
            self.cycles -= 16667;

            if self.read_flag(FLAG_INT) {
                self.interrupt();
            }

            self.last_interrupt_time = time::Instant::now();
        }
    }

    pub fn interrupt(&mut self) {
        self.interrupt_in_progress = true;

        let address: u16;
        let pc = self.pc;

        if self.last_interrupt == INT_END {
            address = INT_MID;
        } else {
            address = INT_END;
            self.handle_input();
            self.vblank();
        }

        self.push_stack(pc);
        self.pc = address;

        self.last_interrupt = address;
    }

    pub fn dump_flags(&mut self) {
        println!("Z: {:?} AC: {:?} C: {:?} P: {:?} S: {:?} I: {:?}", 
            self.read_flag(FLAG_Z),
            self.read_flag(FLAG_AC),
            self.read_flag(FLAG_C),
            self.read_flag(FLAG_P),
            self.read_flag(FLAG_S),
            self.read_flag(FLAG_INT)
        );
    }
}



impl Cpu {
    fn get_vram(&self) -> &[u8] {
        &self.ram.bytes[0x2400..0x4000]
    }

    fn vblank(&mut self) {
        let mut framebuffer: Vec<u32> = Vec::new();
        let mut framebuffer_new: Vec<u32> = Vec::new();

        for (i, byte) in self.get_vram().iter().enumerate() {
            for shift in 0..8 {
                let pixel = if (byte & (1 << shift)) == 0 {
                    [0, 0, 0, 255]
                } else {
                    [255, 255, 255, 255]
                };
            
                let mut buff = &pixel[..];
                let num = buff.read_u32::<BigEndian>().unwrap();

                framebuffer.push(num);
            }
        }

        for y in (0..HEIGHT).rev() {
            for x in (0..WIDTH) {
                framebuffer_new.push(framebuffer[y+(HEIGHT*x)]);
            }
        }

        self.window.update_with_buffer(&framebuffer_new).unwrap();
    }

    fn handle_input(&mut self) {
        if !self.window.is_open() {
            return ();
        }

        let mut input_received = false;

        if self.window.is_key_down(Key::Left) {
            self.inp1 |= (1 << 5);
            input_received = true;
        }

        if self.window.is_key_down(Key::Right) {
            self.inp1 |= (1 << 6);
            input_received = true;
        }

        if self.window.is_key_down(Key::C) {
            self.inp1 |= (1 << 0);
            input_received = true;
        }

        if self.window.is_key_down(Key::X) {
            self.inp1 |= (1 << 2);
            input_received = true;
        }

        if self.window.is_key_down(Key::Z) {
            self.inp1 |= (1 << 4);
            input_received = true;
        }

        if !input_received {
            self.inp1 = 0x0;
        }
    }
}

//Instructions
impl Cpu {
    fn run_instruction(&mut self, opcode: Opcode) {
        match opcode.opcode {
            //TODO: Fix memory cycle counting.
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38   => { nop(self); self.cycles += 4; },
            0xC3 | 0xCB                                             => { jmp(self); self.cycles += 10; },
            0x01 | 0x11 | 0x21 | 0x31                               => { lxi(self); self.cycles += 10;},
            0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E   => { mvi(self); self.cycles += 7; },
            0xCD | 0xDD | 0xED | 0xFD                               => { call(self); self.cycles += 17; },
            0x0A | 0x1A                                             => { ldax(self); self.cycles += 7; },
            0x40 | 0x50 | 0x60 | 0x70 | 0x41 | 0x51 | 0x61 | 
            0x71 | 0x42 | 0x52 | 0x62 | 0x72 | 0x43 | 0x53 | 
            0x63 | 0x73 | 0x44 | 0x54 | 0x64 | 0x74 | 0x45 | 
            0x55 | 0x65 | 0x75 | 0x46 | 0x56 | 0x66 | 0x47 | 
            0x57 | 0x67 | 0x77 | 0x48 | 0x58 | 0x68 | 0x78 | 
            0x49 | 0x59 | 0x69 | 0x79 | 0x4A | 0x5A | 0x6A | 
            0x7A | 0x4B | 0x5B | 0x6B | 0x7B | 0x4C | 0x5C | 
            0x6C | 0x7C | 0x4D | 0x5D | 0x6D | 0x7D | 0x4E | 
            0x5E | 0x6E | 0x7E | 0x4F | 0x5F | 0x6F | 0x7F          => { mov(self); self.cycles += 5; },
            0x03 | 0x13 | 0x23 | 0x33                               => { inx(self); self.cycles += 5; },
            0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D   => { dcr(self); self.cycles += 5; },
            0xC2                                                    => { jnz(self); self.cycles += 10; },
            0x32                                                    => { sta(self); self.cycles += 13; },
            0xC9 | 0xD9                                             => { ret(self); self.cycles += 10; },
            0xFE                                                    => { cpi(self); self.cycles += 7; },
            0xC5 | 0xD5 | 0xE5 | 0xF5                               => { push(self); self.cycles += 11; },
            0x09 | 0x19 | 0x29 | 0x39                               => { dad(self); self.cycles += 10; },
            0xEB                                                    => { xchg(self); self.cycles += 5; },
            0xC1 | 0xD1 | 0xE1 | 0xF1                               => { pop(self); self.cycles += 10; },
            0xD3                                                    => { out(self); self.cycles += 10; },
            0x04 | 0x14 | 0x24 | 0x34  |0x0C | 0x1C | 0x2C | 0x3C   => { inr(self); self.cycles += 5; },
            0x0F                                                    => { rrc(self); self.cycles += 4; },
            0xE6                                                    => { ani(self); self.cycles += 7; },
            0xC6                                                    => { adi(self); self.cycles += 7; },
            0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8E | 0x8F   => { adc(self); self.cycles += 4; },
            0x3A                                                    => { lda(self); self.cycles += 13; },
            0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAE | 0xAF   => { xra(self); self.cycles += 4; },
            0xFB                                                    => { ei(self); self.cycles += 4; },
            0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA6 | 0xA7   => { ana(self); self.cycles += 4; },
            0xCA                                                    => { jz(self); self.cycles += 10; },
            0xDB                                                    => { inp(self); self.cycles += 10; },
            0xC8                                                    => { rz(self); self.cycles += 5; },
            0xDA                                                    => { jc(self); self.cycles += 10; },
            0xD2                                                    => { jnc(self); self.cycles += 10; },
            0x37                                                    => { stc(self); self.cycles += 4; },
            0xD8                                                    => { rc(self); self.cycles += 5; },
            0xB0 | 0xB1 | 0xB2 | 0xB3 | 0xB4 | 0xB5 | 0xB6 | 0xB7   => { ora(self); self.cycles += 4; },
            0x07                                                    => { rlc(self); self.cycles += 4; },
            0xC4                                                    => { cnz(self); self.cycles += 11; },
            0x2A                                                    => { lhld(self); self.cycles += 16; },
            0x1F                                                    => { rar(self); self.cycles += 4; },
            0xF6                                                    => { ori(self); self.cycles += 7; },
            0xE3                                                    => { xthl(self); self.cycles += 18; },
            0xE9                                                    => { pchl(self); self.cycles += 5; },
            0xC0                                                    => { rnz(self); self.cycles += 5; },
            0xD0                                                    => { rnc(self); self.cycles += 5; },
            0x27                                                    => { daa(self); self.cycles += 4; },
            0xCC                                                    => { cz(self); self.cycles += 11; },
            0x0B | 0x1B | 0x2B | 0x3B                               => { dcx(self); self.cycles += 5; },
            0xFA                                                    => { jm(self); self.cycles += 10; },
            0x22                                                    => { shld(self); self.cycles += 16; },
            0xD6                                                    => { sui(self); self.cycles += 7; },
            0xDE                                                    => { sbi(self); self.cycles += 7; },
            0x80 | 0x81 | 0x82 | 0x83 | 0x84 | 0x85 | 0x86 | 0x87   => { add(self); self.cycles += 4; },
            0x2F                                                    => { cma(self); self.cycles += 4; },
            0xB8 | 0xB9 | 0xBA | 0xBB | 0xBC | 0xBD | 0xBE | 0xBF   => { cmp(self); self.cycles += 4; },
            0xD4                                                    => { cnc(self); self.cycles += 11; },
            0x90 | 0x91 | 0x92 | 0x93 | 0x94 | 0x95 | 0x96 | 0x97   => { sub(self); self.cycles += 4; },
            0xEA                                                    => { jpe(self); self.cycles += 10; },
            0xE2                                                    => { jpo(self); self.cycles += 10; },
            0xF2                                                    => { jp(self); self.cycles += 10; },
            0xCE                                                    => { aci(self); self.cycles += 7; },
            0x02 | 0x12                                             => { stax(self); self.cycles += 7; },
            0xEE                                                    => { xri(self); self.cycles += 7; },
            0xDC                                                    => { cc(self); self.cycles += 11; },
            0xE4                                                    => { cpo(self); self.cycles += 11; },
            0xFC                                                    => { cm(self); self.cycles += 11; },
            0xEC                                                    => { cpe(self); self.cycles += 11; },
            0xF4                                                    => { cp(self); self.cycles += 11; },
            0xE8                                                    => { rpe(self); self.cycles += 5; },
            0xE0                                                    => { rpo(self); self.cycles += 5; },
            0xF0                                                    => { rp(self); self.cycles += 5; },
            0xF8                                                    => { rm(self); self.cycles += 5; },
            0x98 | 0x99 | 0x9A | 0x9B | 0x9C | 0x9D | 0x9E | 0x9F   => { sbb(self); self.cycles += 4; },
            0x3F                                                    => { cmc(self); self.cycles += 4; },
            0x17                                                    => { ral(self); self.cycles += 4; },
            0xF9                                                    => { sphl(self); self.cycles += 5; },
            0xF3                                                    => { di(self); self.cycles += 4; },

            _ => {
                println!("Unknown opcode: {:?}", opcode);
                println!("{:?}", self);
                println!("Instruction Count: {:?}", self.instruction_count);
                self.dump_flags();
                panic!("HALT!");
            }
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sv = self.read_stack();

        write!(
            f, 
            "PC: {:#06x} OPCODE: {:?} SP: {:#06x} SV: {:#06X} A: {:#04x} B: {:#04x} C: {:#04x} D: {:#04x} E: {:#04x} H: {:#04x} L: {:#04x} F: {:#04x} ", 
            self.pc, Opcode::new(self.current_opcode), self.sp, sv, self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f
        )
        
        /*
        write!(
            f, 
            "PC: {:#06x} OPCODE: {:#04x} CYCLES: {} SP: {:#06x} A: {:#04x} B: {:#04x} C: {:#04x} D: {:#04x} E: {:#04x} H: {:#04x} L: {:#04x} F: {:#04x} ", 
            self.pc, Opcode::new(self.current_opcode).opcode, self.cycles, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f
        )*/
    }
}