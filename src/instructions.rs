use cpu::*;
use util::*;

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

//Misc instrctions

pub fn nop(state: &mut Cpu) {
}

pub fn ei(state: &mut Cpu) {
    state.f |= FLAG_INT; 
}

//Input/Output
pub fn inp(state: &mut Cpu) {
    let port = state.read_im_byte();

    match port {
        0x01 => {
            let inp1 = state.inp1;
            state.write_byte(REG_A, inp1);
        },
        0x02 => {
            let inp2 = state.inp2;
            state.write_byte(REG_A, inp2);
        },
        0x03 => {
            let port4hi = state.port4hi as u16;
            let port4lo = state.port4lo as u16;
            let port2 = state.port2 as u16;

            let result = (((port4hi << 8) | port4lo) << port2) >> 8;

            state.write_byte(REG_A, result as u8);
        },
        _ => (),
    }
}

pub fn out(state: &mut Cpu) {
    let port = state.read_im_byte();

    match port {
        0x02 => {
            let _a = state.a;
            state.port2 = 0; //Temp, should b a
        },
        0x03 => {
            let port3o = state.port3o;

            state.write_byte(REG_A, port3o);
        },
        0x04 => {
            let port4hi = state.port4hi;
            let a = state.a;

            state.port4lo = port4hi;
            state.port4hi = a;
        },
        _ => /*println!("Unimplemented port for OUT: {:#04x}", port)*/ (),
    }
}


//Jump instructions
pub fn jmp(state: &mut Cpu) {
    let dest = state.read_im_dword();

    state.pc = dest;
}

pub fn jm(state: &mut Cpu) {
    let address = state.read_im_dword();

    if state.read_flag(FLAG_S) {
        state.pc = address;
    }
}

pub fn jnz(state: &mut Cpu) {
    let address = state.read_im_dword();

    if !state.read_flag(FLAG_Z) {
        state.pc = address;
    }
}

pub fn jz(state: &mut Cpu) {
    let address = state.read_im_dword();

    if state.read_flag(FLAG_Z) {
        state.pc = address;
    }
}

pub fn jc(state: &mut Cpu) {
    let address = state.read_im_dword();

    if state.read_flag(FLAG_C) {
        state.pc = address;
    }
}

pub fn jnc(state: &mut Cpu) {
    let address = state.read_im_dword();

    if !state.read_flag(FLAG_C) {
        state.pc = address;
    }
}

pub fn jpe(state: &mut Cpu) {
    let address = state.read_im_dword();

    if state.read_flag(FLAG_P) {
        state.pc = address;
    }
}

pub fn jpo(state: &mut Cpu) {
    let address = state.read_im_dword();

    if !state.read_flag(FLAG_P) {
        state.pc = address;
    }
}

pub fn jp(state: &mut Cpu) {
    let address = state.read_im_dword();

    if !state.read_flag(FLAG_S) {
        state.pc = address;
    }
}

//Call instructions
pub fn call(state: &mut Cpu) {
    let address = state.read_im_dword();

    let pc = state.pc;

    state.push_stack(pc);
    
    state.pc = address;
}

pub fn cnz(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if !state.read_flag(FLAG_Z) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cnc(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if !state.read_flag(FLAG_C) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cz(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if state.read_flag(FLAG_Z) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

//Return instructions
pub fn ret(state: &mut Cpu) {
    let address = state.pop_stack();

    state.pc = address;
}


pub fn rz(state: &mut Cpu) {
    if state.read_flag(FLAG_Z) {
        let address = state.pop_stack();

        state.pc = address;          

        state.cycles += 6;
    }
}

pub fn rp(state: &mut Cpu) {
    if !state.read_flag(FLAG_S) {
        let address = state.pop_stack();

        state.pc = address;          

        state.cycles += 6;
    }
}

pub fn rm(state: &mut Cpu) {
    if state.read_flag(FLAG_S) {
        let address = state.pop_stack();

        state.pc = address;          

        state.cycles += 6;
    }
}

pub fn rpe(state: &mut Cpu) {
    if state.read_flag(FLAG_P) {
        let address = state.pop_stack();

        state.pc = address;          

        state.cycles += 6;
    }
}

pub fn rpo(state: &mut Cpu) {
    if !state.read_flag(FLAG_P) {
        let address = state.pop_stack();

        state.pc = address;          

        state.cycles += 6;
    }
}

pub fn rnz(state: &mut Cpu) {
    if !state.read_flag(FLAG_Z) {
        let address = state.pop_stack();

        state.pc = address; 
        
        state.cycles += 6;
    }
}

pub fn rnc(state: &mut Cpu) {
    if !state.read_flag(FLAG_C) {
        let address = state.pop_stack();

        state.pc = address;    

        state.cycles += 6;      
    }
}
pub fn rc(state: &mut Cpu) {
    if state.read_flag(FLAG_C) {
        let address = state.pop_stack();

        state.pc = address;  

        state.cycles += 6;        
    }
}

//Load instructions
pub fn lxi(state: &mut Cpu) {
    let im = state.read_im_dword();
    let dst = (state.current_opcode >> 4) & 0x03;

    state.write_dword(dst, im);
}

pub fn mvi(state: &mut Cpu) {
    let im = state.read_im_byte();
    let dst = (state.current_opcode >> 3) & 0x07;

    if dst == REG_M {
        state.cycles += 3;
    }

    state.write_byte(dst, im);
}

pub fn shld(state: &mut Cpu) {
    let address = state.read_im_dword();
    let value = state.read_dword(REG_HL);

    state.ram.write_dword(address, value);
}

pub fn ldax(state: &mut Cpu) {
    let src = (state.current_opcode >> 4) & 0x03;
    let dst = REG_A;

    let address = state.read_dword(src);
    let value = state.ram.read_byte(address);

    state.write_byte(dst, value);
}

pub fn mov(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;
    let dst = (state.current_opcode >> 3) & 0x07;

    if src == REG_M || dst == REG_M {
        state.cycles += 2;
    }

    let value = state.read_byte(src);
    state.write_byte(dst, value);
}

pub fn xchg(state: &mut Cpu) {
    let de = state.read_dword(REG_DE);
    let hl = state.read_dword(REG_HL);

    state.write_dword(REG_DE, hl);
    state.write_dword(REG_HL, de);
}

pub fn lda(state: &mut Cpu) {
    let address = state.read_im_dword();
    let value = state.ram.read_byte(address);
    state.write_byte(REG_A, value);
}

pub fn lhld(state: &mut Cpu) {
    let address = state.read_im_dword();
    let value = state.ram.read_dword(address);

    state.write_dword(REG_HL, value);
}
pub fn xthl(state: &mut Cpu) {
    let value = state.pop_stack();
    let hl = state.read_dword(REG_HL);
    state.push_stack(hl);

    state.write_dword(REG_HL, value);
}

pub fn pchl(state: &mut Cpu) {
    state.pc = state.read_dword(REG_HL);
}

//Stack instructions
pub fn push(state: &mut Cpu) {
    let src = (state.current_opcode >> 4) & 0x03;

    if src == 3 {
        //PSW
        let value = state.read_dword(4);
        state.push_stack(value);
    } else {
        let value = state.read_dword(src);
        state.push_stack(value);
    }
}

pub fn pop(state: &mut Cpu) {
    let dst = (state.current_opcode >> 4) & 0x03;

    let value = state.pop_stack();

    if dst == 3 {
        //PSW
        state.write_dword(4, value);
    } else {
        state.write_dword(dst, value);
    }
}

//Arithmetic instructions
pub fn inx(state: &mut Cpu) {
    let dst = (state.current_opcode >> 4) & 0x03;
    let curr = state.read_dword(dst);

    let res = curr.wrapping_add(1);
    state.write_dword(dst, res);
}

pub fn dcr(state: &mut Cpu) {
    let dst = (state.current_opcode >> 3) & 0x07;
    let curr = state.read_byte(dst) as u16;

    if dst == REG_M {
        state.cycles += 5;
    }

    let res = curr.wrapping_sub(1);

    state.write_byte(dst, res as u8);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P, curr, res);
}


pub fn sta(state: &mut Cpu) {
    let address = state.read_im_dword();
    let value = state.read_byte(REG_A);
    state.ram.write_byte(address, value);
}


pub fn cpi(state: &mut Cpu) {
    let rhs = state.read_im_byte() as u16;
    let lhs = state.read_byte(REG_A) as u16;

    let result = lhs.wrapping_sub(rhs);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_C | FLAG_Z | FLAG_P, rhs, result);
}



pub fn dad(state: &mut Cpu) {
    let src = (state.current_opcode >> 4) & 0x03;
    let value = state.read_dword(src) as u32;

    let result: u32 = value.wrapping_add(state.read_dword(REG_HL) as u32);

    if result > 0xffff {
        state.f |= FLAG_C;
    } else {
        state.f &= !FLAG_C;
    }

    state.write_dword(REG_HL, result as u16);
}

pub fn inr(state: &mut Cpu) {
    let dst = (state.current_opcode >> 3) & 0x07;
    let curr = state.read_byte(dst) as u16;

    let res = curr.wrapping_add(1);

    if dst == REG_M {
        state.cycles += 5;
    }

    state.write_byte(dst, res as u8);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P, curr, res);
}

pub fn rrc(state: &mut Cpu) {
    if (state.a & 1) != 0 {
        state.f |= FLAG_C;
    } else {
        state.f &= !FLAG_C;
    }

    let result = (state.a >> 1) | ((state.f & FLAG_C) << 7);
    state.write_byte(REG_A, result);
}

pub fn ani(state: &mut Cpu) {
    let im = state.read_im_byte() as u16;
    let a = state.read_byte(REG_A) as u16;
    let result = a & im;

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, im, result);

    state.write_byte(REG_A, result as u8);
}

pub fn adi(state: &mut Cpu) {
    let im = state.read_im_byte() as u16;
    let a = state.read_byte(REG_A) as u16;
    let result = a.wrapping_add(im);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, im, result);

    state.write_byte(REG_A, result as u8);
}

pub fn adc(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;        
    let value = state.read_byte(src) as u16;
    let a = state.read_byte(REG_A) as u16;

    if src == REG_M {
        state.cycles += 3;
    }

    let im_result = a.wrapping_add(value);
    let result = im_result.wrapping_add(state.f as u16 & FLAG_C as u16);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, value, result);

    state.write_byte(REG_A, result as u8);
}

pub fn aci(state: &mut Cpu) {
    let value = state.read_im_byte() as u16;
    let a = state.read_byte(REG_A) as u16;

    let im_result = a.wrapping_add(value);
    let result = im_result.wrapping_add(state.f as u16 & FLAG_C as u16);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, value, result);

    state.write_byte(REG_A, result as u8);
}

pub fn xra(state: &mut Cpu) {
    let dst = state.current_opcode & 0x07;

    let value = state.read_byte(dst);
    let a = state.a as u16;
    let result = state.a as u16 ^ value as u16;

    if dst == REG_M {
        state.cycles += 3;
    }

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);
    state.write_byte(REG_A, result as u8);
}

pub fn ana(state: &mut Cpu) {
    let dst = state.current_opcode & 0x07;
    let value = state.read_byte(dst);
    let a = state.a as u16;
	let result = value as u16 & state.a as u16;

    if dst == REG_M {
        state.cycles += 3;
    }

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);
    state.write_byte(REG_A, result as u8);
}

pub fn stc(state: &mut Cpu) {
    state.f |= FLAG_C;
}

pub fn ora(state: &mut Cpu) {
    let dst = state.current_opcode & 0x07;
    let value = state.read_byte(dst);
    let a = state.a as u16; 
    let result = value as u16 | state.a as u16;

    if dst == REG_M {
        state.cycles += 3;
    }

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);
    state.write_byte(REG_A, result as u8);
}

pub fn rlc(state: &mut Cpu) {
    if (state.a & (1 << 7)) != 0 {
        state.f |= FLAG_C;
    } else {
        state.f &= !FLAG_C;
    }

    let result = (state.a << 1) | (state.f & FLAG_C);

    state.write_byte(REG_A, result);
}

//================================================

pub fn rar(state: &mut Cpu) {
    let tmp = state.a;

    state.a = tmp >> 1;

    if state.read_flag(FLAG_C) {
        state.a |= 0x80;
    }

    if (tmp & 1) > 0 {
        state.f |= FLAG_C;
    } else {
        state.f &= !FLAG_C;
    }
}

pub fn ral(state: &mut Cpu) {
    let tmp = state.a;

    state.a = tmp << 1;

    if state.read_flag(FLAG_C) {
        state.a |= 0x01;
    }

    if (tmp & 0x80) > 0 {
        state.f |= FLAG_C;
    } else {
        state.f &= !FLAG_C;
    }
}

pub fn ori(state: &mut Cpu) {
    let value = state.read_im_byte();
    let result = value as u16 | state.a as u16;
    let a = state.a;

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a as u16, result);
    state.write_byte(REG_A, result as u8);
}

pub fn daa(state: &mut Cpu) {
    let mut result = state.a as u16;

    let least = result & 0xf;

    if state.read_flag(FLAG_AC) || least > 9 {
        result += 6;

        if result & 0xf < least {
            state.f |= FLAG_AC;
        }
    }

    let least = result & 0xf;
    let mut most = (result >> 4) & 0xf;

    if state.read_flag(FLAG_C) || most > 9 {
        most += 6;
    }

    let result = ((most << 4) as u16) | least as u16;
    let a = state.a as u16;
    state.set_flags(FLAG_S | FLAG_Z | FLAG_P | FLAG_C, a, result);

    state.a = result as u8;
}

pub fn dcx(state: &mut Cpu) {
    let dst = (state.current_opcode >> 4) & 0x03;
    let curr = state.read_dword(dst);

    let res = curr.wrapping_sub(1);
    state.write_dword(dst, res);
}

pub fn sbi(state: &mut Cpu) {
    //TODO: Make pretty
    let lhs = state.a as u16;
    let mut rhs = state.read_im_byte() as u16;

    let carry;

    if state.read_flag(FLAG_C) {
        carry = 1;
    } else {
        carry = 0;
    }

    rhs = rhs.wrapping_add(carry) as u16;

    let answer = (state.a as u16).wrapping_sub(rhs);
    let a = state.a as u16;

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, answer);

    state.a = answer as u8;
}

pub fn sbb(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;

    if src == REG_M {
        state.cycles += 3;
    }

    //TODO: Make pretty
    let lhs = state.a as u16;
    let mut rhs = state.read_byte(src) as u16;

    let carry;

    if state.read_flag(FLAG_C) {
        carry = 1;
    } else {
        carry = 0;
    }

    rhs = rhs.wrapping_add(carry) as u16;

    let answer = (state.a as u16).wrapping_sub(rhs);
    let a = state.a as u16;

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, answer);

    state.a = answer as u8;
}

pub fn sui(state: &mut Cpu) {
    let value = state.read_im_byte() as u16;
    let a = state.a as u16;

    let result = a.wrapping_sub(value);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);

    state.write_byte(REG_A, result as u8);
}

pub fn add(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;

    if src == REG_M {
        state.cycles += 3;
    }

    let value = state.read_byte(src) as u16;
    let a = state.a as u16;

    let result = a.wrapping_add(value);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);

    state.write_byte(REG_A, result as u8);
}

pub fn sub(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;

    if src == REG_M {
        state.cycles += 3;
    }

    let value = state.read_byte(src) as u16;
    let a = state.a as u16;

    let result = a.wrapping_sub(value);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);

    state.write_byte(REG_A, result as u8);
}

pub fn cma(state: &mut Cpu) {
    let result = !state.read_byte(REG_A);

    state.write_byte(REG_A, result);
}

pub fn cmp(state: &mut Cpu) {
    let src = state.current_opcode & 0x07;

    if src == REG_M {
        state.cycles += 3;
    }

    let value = state.read_byte(src) as u16;
    let a = state.a as u16;

    let result = a.wrapping_sub(value);

    state.set_flags(FLAG_S | FLAG_AC | FLAG_Z | FLAG_P | FLAG_C, a, result);
}

pub fn stax(state: &mut Cpu) {
    let src = (state.current_opcode >> 4) & 0x03;
    
    let address = state.read_dword(src);
    let value = state.read_byte(REG_A);

    state.ram.write_byte(address, value);
}

pub fn xri(state: &mut Cpu) {
    let value = state.read_im_byte();
    let a = state.a;
    let result = state.a ^ value;

    state.f &= !FLAG_C;

    state.set_flags(FLAG_S | FLAG_Z | FLAG_P, a as u16, result as u16);
    state.write_byte(REG_A, result);
}

pub fn cc(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if state.read_flag(FLAG_C) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cpo(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if !state.read_flag(FLAG_P) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cm(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if state.read_flag(FLAG_S) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cpe(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if state.read_flag(FLAG_P) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cp(state: &mut Cpu) {
    let address = state.read_im_dword();
    
    if !state.read_flag(FLAG_S) {
        let pc = state.pc;
        state.push_stack(pc);
        state.pc = address;

        state.cycles += 6;
    }
}

pub fn cmc(state: &mut Cpu) {
    if state.read_flag(FLAG_C) {
        state.f &= !FLAG_C;
    } else {
        state.f |= FLAG_C;
    }
}

pub fn sphl(state: &mut Cpu) {
    let value = state.read_dword(REG_HL);
    state.sp = value;
}