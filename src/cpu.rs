use std::fmt;
use ram::Sram;
use util::*;
use opcode::Opcode;

use minifb::{Key, WindowOptions, Window};
use byteorder::{BigEndian, ReadBytesExt};

#[allow(dead_code)]
const REG_BC: u8 = 0;
#[allow(dead_code)]
const REG_DE: u8 = 1;
const REG_HL: u8 = 2;
#[allow(dead_code)]
const REG_AF: u8 = 3;

const REG_A: u8 = 7;

const FLAG_C: u8 = 1 << 0;
const FLAG_P: u8 = 1 << 2;
const FLAG_AC: u8 = 1 << 4;
const FLAG_INT: u8 = 1 << 5;
const FLAG_Z: u8 = 1 << 6;
const FLAG_S: u8 = 1 << 7;

const INT_MID: u16 = 0x08;
const INT_END: u16 = 0x10;

const WIDTH: usize = 256;
const HEIGHT: usize = 224;

#[allow(dead_code)]
pub struct Cpu {
	a: u8,
	f: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	l: u8,
	sp: u16,
	pc: u16,
	ram: Sram,
	cycles: u32,
	current_opcode: u8,
	instruction_count: u64,
	last_interrupt: u16,
	window: Window,
	buffer: Vec<u32>,
}

impl Cpu {
	pub fn new(ram: Sram) -> Cpu {
		let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

		let mut window = Window::new("Test - ESC to exit",
                                 HEIGHT,
                                 WIDTH,
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
			sp: 0x0000,
			pc: 0x0000,
			ram: ram,
			cycles: 0,
			current_opcode: 0x00,
			instruction_count: 0,
			last_interrupt: INT_MID,
			window: window,
			buffer: buffer,
		}
	}
}

// Read/Write register methods and utility
impl Cpu {
	fn read_byte(&self, index: u8) -> u8 {
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

	fn pop_stack(&mut self) -> u16 {
    	let value = u8_to_u16(self.ram.read_byte(self.sp + 1), self.ram.read_byte(self.sp));
    	self.sp += 2;

    	value
	}

	fn push_stack(&mut self, value: u16) {
    	self.sp -= 2;
    	self.ram.write_dword(self.sp, value);
	}


	fn read_dword(&self, index: u8) -> u16 {
		match index {
			0 => u8_to_u16(self.c, self.b),
			1 => u8_to_u16(self.e, self.d),
			2 => u8_to_u16(self.l, self.h),
			3 => self.sp,
			4 => u8_to_u16(self.f, self.a),
			_ => panic!("Unknwon index for read_dword()")
		}
	}

	fn write_byte(&mut self, index: u8, value: u8) {
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

	fn write_dword(&mut self, index: u8, value: u16) {
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

	fn read_im_byte(&mut self) -> u8 {
		let im = self.ram.read_byte(self.pc);
		self.pc += 1;

		im
	}

	fn read_im_dword(&mut self) -> u16 {
		let im = self.ram.read_dword(self.pc);
		self.pc += 2;

		im
	}

	fn set_flags(&mut self, mask: u8, initial: u16, result: u16) {
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

	fn read_flag(&mut self, flag: u8) -> bool {
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

	fn check_interrupt(&mut self) {
		if self.cycles > 16667 {
			self.cycles -= 1667;

			if self.read_flag(FLAG_INT) {
				self.interrupt();
			}
		}
	}

	fn interrupt(&mut self) {
		let address: u16;
		let pc = self.pc;

		if self.last_interrupt == INT_END {
			address = INT_MID;
		} else {
			address = INT_END;
			self.vblank();
		}

		self.push_stack(pc);
		self.pc = address;

		self.last_interrupt = address;
	}

	fn dump_flags(&mut self) {
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
		let mut framebuffer: Vec<u32> = vec![0; WIDTH * HEIGHT];


		for (i, byte) in self.get_vram().iter().enumerate() {
	        const SHIFT_END: u8 = 7;

	        // Really x is y and y is x as the frame is rotated 90 degrees
	        let y = i * 8 / (WIDTH as usize + 1);

	        for shift in 0..SHIFT_END + 1 {
	            let x = ((i * 8) % (WIDTH as usize)) + shift as usize;

				let pixel = if (byte >> shift) & 1 == 0 {
				    	[0, 0, 0, 255]
					} else {
					    [255, 255, 255, 255]
					};

	            let mut buff = &pixel[..];
	            let num = buff.read_u32::<BigEndian>().unwrap();

	            framebuffer[(x*HEIGHT+y) as usize] = num;
	        }
    	}

    	for (i, byte) in framebuffer.iter().rev().enumerate() {
    		self.buffer[i] = *byte as u32;
    	}

    	self.window.update_with_buffer(&self.buffer).unwrap();
	}
}


impl Cpu {
	pub fn run(&mut self) {
		loop {
			self.check_interrupt();

			let opcode = Opcode::new(self.ram.read_byte(self.pc));

			self.current_opcode = opcode.opcode;

	        self.pc += 1;

	        self.run_instruction(opcode);
	        self.instruction_count += 1;
		}
	}
}


//Instructions
impl Cpu {
	fn run_instruction(&mut self, opcode: Opcode) {
		match opcode.opcode {
			//TODO: Fix memory cycle counting.
			0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 	=> { self.nop(); self.cycles += 4; },
			0xC3 | 0xCB 										  	=> { self.jmp(); self.cycles += 10; },
			0x01 | 0x11 | 0x21 | 0x31								=> { self.lxi(); self.cycles += 10;},
			0x06 | 0x16 | 0x26 | 0x36 | 0x0E | 0x1E | 0x2E | 0x3E 	=> { self.mvi(); self.cycles += 7; },
			0xCD | 0xDD | 0xED | 0xFD								=> { self.call(); self.cycles += 17; },
			0x0A | 0x1A												=> { self.ldax(); self.cycles += 7; },
			0x40 | 0x50 | 0x60 | 0x70 | 0x41 | 0x51 | 0x61 | 
			0x71 | 0x42 | 0x52 | 0x62 | 0x72 | 0x43 | 0x53 | 
			0x63 | 0x73 | 0x44 | 0x54 | 0x64 | 0x74 | 0x45 | 
			0x55 | 0x65 | 0x75 | 0x46 | 0x56 | 0x66 | 0x47 | 
			0x57 | 0x67 | 0x77 | 0x48 | 0x58 | 0x68 | 0x78 | 
			0x49 | 0x59 | 0x69 | 0x79 | 0x4A | 0x5A | 0x6A | 
			0x7A | 0x4B | 0x5B | 0x6B | 0x7B | 0x4C | 0x5C | 
			0x6C | 0x7C | 0x4D | 0x5D | 0x6D | 0x7D | 0x4E | 
			0x5E | 0x6E | 0x7E | 0x4F | 0x5F | 0x6F | 0x7F			=> { self.mov(); self.cycles += 5; },
			0x03 | 0x13 | 0x23 | 0x33								=> { self.inx(); self.cycles += 5; },
			0x05 | 0x15 | 0x25 | 0x35 | 0x0D | 0x1D | 0x2D | 0x3D	=> { self.dcr(); self.cycles += 5; },
			0xC2													=> { self.jnz(); self.cycles += 10; },
			0x32													=> { self.sta(); self.cycles += 13; },
			0xC9 | 0xD9												=> { self.ret(); self.cycles += 10; },
			0xFE													=> { self.cpi(); self.cycles += 7; },
			0xC5 | 0xD5 | 0xE5 | 0xF5								=> { self.push(); self.cycles += 11; },
			0x09 | 0x19 | 0x29 | 0x39 								=> { self.dad(); self.cycles += 10; },
			0xEB													=> { self.xchg(); self.cycles += 5; },
			0xC1 | 0xD1 | 0xE1 | 0xF1								=> { self.pop(); self.cycles += 10; },
			0xD3													=> { self.out(); self.cycles += 10; },
			0x04 | 0x14 | 0x24 | 0x34  |0x0C | 0x1C | 0x2C | 0x3C 	=> { self.inr(); self.cycles += 5; },
			0x0F													=> { self.rrc(); self.cycles += 4; },
			0xE6 													=> { self.ani(); self.cycles += 7; },
			0xC6 													=> { self.adi(); self.cycles += 7; },
			0x88 | 0x89 | 0x8A | 0x8B | 0x8C | 0x8D | 0x8E | 0x8F	=> { self.adc(); self.cycles += 4; },
			0x3A													=> { self.lda(); self.cycles += 13; },
			0xA8 | 0xA9 | 0xAA | 0xAB | 0xAC | 0xAD | 0xAE | 0xAF	=> { self.xra(); self.cycles += 4; },
			0xFB													=> { self.ei(); self.cycles += 4; },
			0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5 | 0xA6 | 0xA7	=> { self.ana(); self.cycles += 4; },
			0xCA													=> { self.jz(); self.cycles += 10; },
			0xDB													=> { self.inp(); self.cycles += 10; },
			0xC8 													=> { self.rz(); self.cycles += 11; },
			0xDA													=> { self.jc(); self.cycles += 10; },
			0xD2													=> { self.jnc(); self.cycles += 10; },
			0x37 													=> { self.stc(); self.cycles += 4; },
			0xD8 													=> { self.rc(); self.cycles += 11; },


			_ => {
				println!("Unknown opcode: {:?}", opcode);
				println!("{:?}", self);
				println!("Instruction Count: {:?}", self.instruction_count);
				self.dump_flags();
				panic!("HALT!");
			}
		}
	}

	fn nop(&mut self) {
		self.cycles += 4;
	}

	fn jmp(&mut self) {
		let dest = self.read_im_dword();
		self.pc = dest;
	}

	fn lxi(&mut self) {
		let im = self.read_im_dword();
		let dst = (self.current_opcode >> 4) & 0x03;

		self.write_dword(dst, im);
	}

	fn mvi(&mut self) {
		let im = self.read_im_byte();
		let dst = (self.current_opcode >> 3) & 0x07;

		self.write_byte(dst, im);
	}

	fn call(&mut self) {
		let address = self.read_im_dword();
		let sp = self.pc;
		
		self.push_stack(sp);
		
		self.pc = address;
	}

	fn ldax(&mut self) {
		let src = (self.current_opcode >> 4) & 0x03;
		let dst = REG_A;

		let address = self.read_dword(src);
		let value = self.ram.read_byte(address);

		self.write_byte(dst, value);
	}

	fn mov(&mut self) {
		let src = self.current_opcode & 0x07;
		let dst = (self.current_opcode >> 3) & 0x07;

		let value = self.read_byte(src);
		self.write_byte(dst, value);
	}

	fn inx(&mut self) {
		let dst = (self.current_opcode >> 4) & 0x03;
		let curr = self.read_dword(dst);

		let res = curr.wrapping_add(1);
		self.write_dword(dst, res);
	}

	fn dcr(&mut self) {
		let dst = (self.current_opcode >> 3) & 0x07;
		let curr = self.read_byte(dst) as u16;

		let res = curr.wrapping_sub(1);

		self.write_byte(dst, res as u8);

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P, curr, res);
	}

	fn jnz(&mut self) {
		let address = self.read_im_dword();

		if !self.read_flag(FLAG_Z) {
			self.pc = address;
		}
	}

	fn jz(&mut self) {
		let address = self.read_im_dword();

		if self.read_flag(FLAG_Z) {
			self.pc = address;
		}
	}

	fn sta(&mut self) {
		let address = self.read_im_dword();
		let value = self.read_byte(REG_A);
		self.ram.write_byte(address, value);
	}

	fn ret(&mut self) {
		let address = self.pop_stack();

		self.pc = address;
	}

	fn cpi(&mut self) {
		let rhs = self.read_im_byte() as u16;
		let lhs = self.read_byte(REG_A) as u16;

		let result = lhs.wrapping_sub(rhs);

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P, rhs, result);
	}

	fn push(&mut self) {
		let src = (self.current_opcode >> 4) & 0x03;

		if src == 3 {
			//PSW
			let value = self.read_dword(4);
			self.push_stack(value);
		} else {
			let value = self.read_dword(src);
			self.push_stack(value);
		}
	}

	fn dad(&mut self) {
		let src = (self.current_opcode >> 4) & 0x03;
		let value = self.read_dword(src) as u32;

		let result: u32 = value.wrapping_add(self.read_dword(REG_HL) as u32);

		if result > 0xffff {
			self.f |= FLAG_C;
		} else {
			self.f &= !FLAG_C;
		}

		self.write_dword(REG_HL, result as u16);
	}

	fn xchg(&mut self) {
		let de = self.read_dword(REG_DE);
		let hl = self.read_dword(REG_HL);

		self.write_dword(REG_DE, hl);
		self.write_dword(REG_HL, de);
	}

	fn pop(&mut self) {
		let dst = (self.current_opcode >> 4) & 0x03;

		let value = self.pop_stack();

		if dst == 3 {
			//PSW
			self.write_dword(4, value);
		} else {
			self.write_dword(dst, value);
		}
	}

	fn out(&mut self) {
		let port = self.read_im_byte();

		match port {
			_ => /*println!("Unimplemented port for OUT: {:#04x}", port)*/ (),
		}
	}

	fn inr(&mut self) {
		let dst = (self.current_opcode >> 3) & 0x07;
		let curr = self.read_byte(dst) as u16;

		let res = curr.wrapping_add(1);

		self.write_byte(dst, res as u8);

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P, curr, res);
	}

	fn rrc(&mut self) {
		if (self.a & 1) != 0 {
			self.f |= FLAG_C;
		} else {
			self.f &= !FLAG_C;
		}

		let result = (self.a >> 1) | (self.f & FLAG_C) << 7;
		self.write_byte(REG_A, result);
	}

	fn ani(&mut self) {
		let im = self.read_im_byte() as u16;
		let a = self.read_byte(REG_A) as u16;
		let result = a & im;

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);

		self.write_byte(REG_A, result as u8);
	}

	fn adi(&mut self) {
		let im = self.read_im_byte() as u16;
		let a = self.read_byte(REG_A) as u16;
		let result = a + im;

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, im, result);

		self.write_byte(REG_A, result as u8);
	}

	fn adc(&mut self) {
		let src = (self.current_opcode >> 3) & 0x07;		
		let value = self.read_byte(src) as u16;
		let a = self.read_byte(REG_A) as u16;

		let im_result = a.wrapping_add(value);
		let result = im_result.wrapping_add(self.f as u16 & FLAG_C as u16);

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, value, result);

		self.write_byte(REG_A, result as u8);
	}

	fn lda(&mut self) {
		let address = self.read_im_dword();
		let value = self.ram.read_byte(address);
		self.write_byte(REG_A, value);
	}

	fn xra(&mut self) {
		let dst = (self.current_opcode >> 3) & 0x07;
		let value = self.read_byte(dst);
		let result = value as u16 ^ self.a as u16;

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, value as u16, result);
		self.write_byte(dst, result as u8);
	}

	fn ana(&mut self) {
		let dst = (self.current_opcode >> 3) & 0x07;
		let value = self.read_byte(dst);
		let result = value as u16 & self.a as u16;

		self.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, value as u16, result);
		self.write_byte(dst, result as u8);
	}


	fn ei(&mut self) {
		self.f |= FLAG_INT; 
	}

	fn inp(&mut self) {
		let _port = self.read_im_byte();
	}

	fn rz(&mut self) {
		if self.read_flag(FLAG_Z) {
			let address = self.pop_stack();

			self.pc = address;			
		}
	}

	fn jc(&mut self) {
		let address = self.read_im_dword();

		if self.read_flag(FLAG_C) {
			self.pc = address;
		}
	}

	fn jnc(&mut self) {
		let address = self.read_im_dword();

		if !self.read_flag(FLAG_C) {
			self.pc = address;
		}
	}

	fn stc(&mut self) {
		self.f |= FLAG_C;
	}

	fn rc(&mut self) {
		if self.read_flag(FLAG_C) {
			let address = self.pop_stack();

			self.pc = address;			
		}
	}
}

impl fmt::Debug for Cpu {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
        	f, 
        	"PC: {:#06x} SP: {:#06x} A: {:#04x} B: {:#04x} C: {:#04x} D: {:#04x} E: {:#04x} H: {:#04x} L: {:#04x} F: {:#04x} ", 
        	self.pc, self.sp, self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f
    	)
	}
}