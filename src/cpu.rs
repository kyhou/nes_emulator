use macroquad::prelude::*;
use std::collections::BTreeMap;

use crate::{Bus, Cartridge, Ppu};

const CYAN: macroquad::color::Color = Color {
    r: 0.0,
    g: 255.0,
    b: 255.0,
    a: 1.0,
};

#[repr(u8)]
enum Flags {
    C = (1 << 0), // Carry bit
    Z = (1 << 1), // Zero
    I = (1 << 2), // Disable Interrupts
    D = (1 << 3), // Decimal Mode
    B = (1 << 4), // Break
    U = (1 << 5), // Unused
    V = (1 << 6), // Overflow
    N = (1 << 7), // Negative
}

struct Instruction {
    name: String,
    operate: fn(&mut Cpu, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8,
    addrmode: fn(&mut Cpu, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8,
    cycles: u8,
}

pub struct Cpu {
    a: u8,       // Accumulator Register
    x: u8,       // X register
    y: u8,       // Y register,
    stkp: u8,    // Stack Pointer (points to location on bus)
    pub pc: u16, // Program counter
    status: u8,  // Status Register
    fetched: u8,
    addr_abs: u16,
    addr_rel: u16,
    opcode: u8,
    cycles: u8,
    clock_count: u32,
    lookup: Vec<Instruction>,
}
impl Cpu {
    pub fn new() -> Self {
        return Self {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            stkp: 0x00,
            pc: 0x0000,
            status: 0x00,
            fetched: 0x00,
            addr_abs: 0x0000,
            addr_rel: 0x0000,
            opcode: 0x00,
            cycles: 0,
            clock_count: 0,
            lookup: vec![
                Instruction {
                    name: String::from("BRK"),
                    operate: Cpu::brk,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ASL"),
                    operate: Cpu::asl,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("PHP"),
                    operate: Cpu::php,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ASL"),
                    operate: Cpu::asl,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ASL"),
                    operate: Cpu::asl,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BPL"),
                    operate: Cpu::bpl,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ASL"),
                    operate: Cpu::asl,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("CLC"),
                    operate: Cpu::clc,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ORA"),
                    operate: Cpu::ora,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ASL"),
                    operate: Cpu::asl,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("JSR"),
                    operate: Cpu::jsr,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("BIT"),
                    operate: Cpu::bit,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ROL"),
                    operate: Cpu::rol,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("PLP"),
                    operate: Cpu::plp,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ROL"),
                    operate: Cpu::rol,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("BIT"),
                    operate: Cpu::bit,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROL"),
                    operate: Cpu::rol,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BMI"),
                    operate: Cpu::bmi,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROL"),
                    operate: Cpu::rol,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("SEC"),
                    operate: Cpu::sec,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("AND"),
                    operate: Cpu::and,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROL"),
                    operate: Cpu::rol,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("RTI"),
                    operate: Cpu::rti,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("LSR"),
                    operate: Cpu::lsr,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("PHA"),
                    operate: Cpu::pha,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LSR"),
                    operate: Cpu::lsr,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("JMP"),
                    operate: Cpu::jmp,
                    addrmode: Cpu::abs,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LSR"),
                    operate: Cpu::lsr,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BVC"),
                    operate: Cpu::bvc,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LSR"),
                    operate: Cpu::lsr,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("CLI"),
                    operate: Cpu::cli,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("EOR"),
                    operate: Cpu::eor,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LSR"),
                    operate: Cpu::lsr,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("RTS"),
                    operate: Cpu::rts,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("ROR"),
                    operate: Cpu::ror,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("PLA"),
                    operate: Cpu::pla,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ROR"),
                    operate: Cpu::ror,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("JMP"),
                    operate: Cpu::jmp,
                    addrmode: Cpu::ind,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROR"),
                    operate: Cpu::ror,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BVS"),
                    operate: Cpu::bvs,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROR"),
                    operate: Cpu::ror,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("SEI"),
                    operate: Cpu::sei,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ADC"),
                    operate: Cpu::adc,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("ROR"),
                    operate: Cpu::ror,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("STY"),
                    operate: Cpu::sty,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("STX"),
                    operate: Cpu::stx,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("DEY"),
                    operate: Cpu::dey,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("TXA"),
                    operate: Cpu::txa,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("STY"),
                    operate: Cpu::sty,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("STX"),
                    operate: Cpu::stx,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("BCC"),
                    operate: Cpu::bcc,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::izy,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("STY"),
                    operate: Cpu::sty,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("STX"),
                    operate: Cpu::stx,
                    addrmode: Cpu::zpy,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("TYA"),
                    operate: Cpu::tya,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::aby,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("TXS"),
                    operate: Cpu::txs,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("STA"),
                    operate: Cpu::sta,
                    addrmode: Cpu::abx,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("LDY"),
                    operate: Cpu::ldy,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("LDX"),
                    operate: Cpu::ldx,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("LDY"),
                    operate: Cpu::ldy,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("LDX"),
                    operate: Cpu::ldx,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("TAY"),
                    operate: Cpu::tay,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("TAX"),
                    operate: Cpu::tax,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LDY"),
                    operate: Cpu::ldy,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDX"),
                    operate: Cpu::ldx,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("BCS"),
                    operate: Cpu::bcs,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("LDY"),
                    operate: Cpu::ldy,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDX"),
                    operate: Cpu::ldx,
                    addrmode: Cpu::zpy,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("CLV"),
                    operate: Cpu::clv,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("TSX"),
                    operate: Cpu::tsx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDY"),
                    operate: Cpu::ldy,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDA"),
                    operate: Cpu::lda,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("LDX"),
                    operate: Cpu::ldx,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("CPY"),
                    operate: Cpu::cpy,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("CPY"),
                    operate: Cpu::cpy,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("DEC"),
                    operate: Cpu::dec,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("INY"),
                    operate: Cpu::iny,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("DEX"),
                    operate: Cpu::dex,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CPY"),
                    operate: Cpu::cpy,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("DEC"),
                    operate: Cpu::dec,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BNE"),
                    operate: Cpu::bne,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("DEC"),
                    operate: Cpu::dec,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("CLD"),
                    operate: Cpu::cld,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("NOP"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("CMP"),
                    operate: Cpu::cmp,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("DEC"),
                    operate: Cpu::dec,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("CPX"),
                    operate: Cpu::cpx,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::izx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("CPX"),
                    operate: Cpu::cpx,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::zp0,
                    cycles: 3,
                },
                Instruction {
                    name: String::from("INC"),
                    operate: Cpu::inc,
                    addrmode: Cpu::zp0,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("INX"),
                    operate: Cpu::inx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::imm,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("NOP"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("CPX"),
                    operate: Cpu::cpx,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::abs,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("INC"),
                    operate: Cpu::inc,
                    addrmode: Cpu::abs,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("BEQ"),
                    operate: Cpu::beq,
                    addrmode: Cpu::rel,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::izy,
                    cycles: 5,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 8,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::zpx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("INC"),
                    operate: Cpu::inc,
                    addrmode: Cpu::zpx,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 6,
                },
                Instruction {
                    name: String::from("SED"),
                    operate: Cpu::sed,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::aby,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("NOP"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 2,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::nop,
                    addrmode: Cpu::imp,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("SBC"),
                    operate: Cpu::sbc,
                    addrmode: Cpu::abx,
                    cycles: 4,
                },
                Instruction {
                    name: String::from("INC"),
                    operate: Cpu::inc,
                    addrmode: Cpu::abx,
                    cycles: 7,
                },
                Instruction {
                    name: String::from("???"),
                    operate: Cpu::xxx,
                    addrmode: Cpu::imp,
                    cycles: 7,
                },
            ],
        };
    }

    fn write(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge, addr: u16, data: u8) {
        bus.cpu_write(ppu, cart, addr, data);
    }

    fn read(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge, addr: u16) -> u8 {
        bus.cpu_read(ppu, cart, addr, false)
    }

    fn get_flag(&self, flag: Flags) -> u8 {
        if (self.status & flag as u8) > 0 {
            1
        } else {
            0
        }
    }

    fn set_flag(&mut self, flag: Flags, v: bool) {
        if v {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    // Addressing Modes

    fn imp(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.fetched = self.a;
        0
    }

    fn imm(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.addr_abs = self.pc;
        self.pc = self.pc.wrapping_add(1);
        0
    }

    fn zp0(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.addr_abs = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        0
    }

    fn zpx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.addr_abs = (self.read(bus, ppu, cart, self.pc).wrapping_add(self.x)) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        0
    }

    fn zpy(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.addr_abs = (self.read(bus, ppu, cart, self.pc).wrapping_add(self.y)) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        0
    }

    fn rel(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.addr_rel = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        if self.addr_rel & 0x80 > 0 {
            self.addr_rel |= 0xFF00;
        }

        0
    }

    fn abs(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let lo: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi.wrapping_shl(8)) | lo;
        0
    }

    fn abx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let lo: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi.wrapping_shl(8)) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.x as u16);

        if (self.addr_abs & 0xFF00) != (hi.wrapping_shl(8)) {
            1
        } else {
            0
        }
    }

    fn aby(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let lo: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi.wrapping_shl(8)) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);

        if (self.addr_abs & 0xFF00) != (hi.wrapping_shl(8)) {
            1
        } else {
            0
        }
    }

    fn ind(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let ptr_lo: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let ptr_hi: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let ptr: u16 = (ptr_hi.wrapping_shl(8)) | ptr_lo;

        if ptr_lo == 0x00FF {
            // simulate page boundary hardware bug
            self.addr_abs = ((self.read(bus, ppu, cart, ptr & 0xFF00) as u16).wrapping_shl(8))
                | self.read(bus, ppu, cart, ptr) as u16;
        } else {
            // behave normally
            self.addr_abs = ((self.read(bus, ppu, cart, ptr.wrapping_add(1)) as u16).wrapping_shl(8))
                | self.read(bus, ppu, cart, ptr) as u16;
        }

        0
    }

    fn izx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let t: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let lo: u16 = self.read(bus, ppu, cart, t.wrapping_add(self.x as u16) & 0x00FF) as u16;
        let hi: u16 = self.read(
            bus,
            ppu,
            cart,
            t.wrapping_add(self.x as u16).wrapping_add(1) & 0x00FF,
        ) as u16;

        self.addr_abs = (hi.wrapping_shl(8)) | lo;

        0
    }

    fn izy(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        let t: u16 = self.read(bus, ppu, cart, self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let lo: u16 = self.read(bus, ppu, cart, t & 0x00FF) as u16;
        let hi: u16 = self.read(bus, ppu, cart, (t.wrapping_add(1)) & 0x00FF) as u16;

        self.addr_abs = (hi.wrapping_shl(8)) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);

        if (self.addr_abs & 0x00FF) != (hi.wrapping_shl(8)) {
            1
        } else {
            0
        }
    }

    // Opcodes
    fn adc(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);

        let temp: u16 = (self.a as u16)
            .wrapping_add(self.fetched as u16)
            .wrapping_add(self.get_flag(Flags::C) as u16);
        self.set_flag(Flags::C, temp > 255);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);
        self.set_flag(
            Flags::V,
            (!(self.a as u16 ^ self.fetched as u16) & (self.a as u16 ^ temp) & 0x0080) == 0x0080,
        );
        self.a = (temp & 0x00FF) as u8;

        1
    }

    fn and(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.a &= self.fetched;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        1
    }

    fn asl(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = (self.fetched as u16).wrapping_shl(1);
        self.set_flag(Flags::C, (temp as u16 & 0xFF00) > 0);
        self.set_flag(Flags::Z, (temp as u16 & 0x00FF) == 0x00);
        self.set_flag(Flags::N, (temp as u16 & 0x80) == 0x80);

        if (self.lookup[self.opcode as usize].addrmode) as usize == (Cpu::imp) as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, ppu, cart, self.addr_abs, (temp & 0x00FF) as u8);
        }

        0
    }

    fn bcc(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::C) == 0 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn bcs(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::C) == 1 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn beq(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::Z) == 1 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn bit(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = self.a & self.fetched;

        self.set_flag(Flags::Z, (temp as u16 & 0x00FF) == 0x00);
        self.set_flag(Flags::N, self.fetched & (1 << 7) == (1 << 7));
        self.set_flag(Flags::V, self.fetched & (1 << 6) == (1 << 6));

        0
    }

    fn bmi(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::N) == 1 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn bne(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::Z) == 0 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn bpl(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::N) == 0 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn brk(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.set_flag(Flags::I, true);
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            ((self.pc.wrapping_shr(8)) & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            (self.pc & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);

        self.set_flag(Flags::B, true);
        self.write(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16), self.status);
        self.stkp = self.stkp.wrapping_sub(1);
        self.set_flag(Flags::B, false);

        self.pc = self.read(bus, ppu, cart, 0xFFFE) as u16
            | ((self.read(bus, ppu, cart, 0xFFFF) as u16).wrapping_shl(8));

        0
    }

    fn bvc(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::V) == 0 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn bvs(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        if self.get_flag(Flags::V) == 1 {
            self.cycles = self.cycles.wrapping_add(1);
            self.addr_abs = self.pc.wrapping_add(self.addr_rel);

            if (self.addr_abs & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles = self.cycles.wrapping_add(1);
            }

            self.pc = self.addr_abs;
        }

        0
    }

    fn clc(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::C, false);
        0
    }

    fn cld(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::D, false);
        0
    }

    fn cli(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::I, false);
        0
    }

    fn clv(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::V, false);
        0
    }

    fn cmp(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = (self.a as u16).wrapping_sub(self.fetched as u16);
        self.set_flag(Flags::C, self.a >= self.fetched);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        1
    }

    fn cpx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = (self.x as u16).wrapping_sub(self.fetched as u16);
        self.set_flag(Flags::C, self.x >= self.fetched);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        0
    }

    fn cpy(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = (self.y as u16).wrapping_sub(self.fetched as u16);
        self.set_flag(Flags::C, self.y >= self.fetched);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        0
    }

    fn dec(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = self.fetched.wrapping_sub(1);
        self.write(bus, ppu, cart, self.addr_abs, temp & 0x00FF);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        0
    }

    fn dex(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) == 0x80);

        0
    }

    fn dey(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) == 0x80);

        0
    }

    fn eor(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.a ^= self.fetched;

        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        1
    }

    fn inc(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = self.fetched.wrapping_add(1);
        self.write(bus, ppu, cart, self.addr_abs, temp & 0x00FF);

        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        0
    }

    fn inx(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) == 0x80);

        0
    }

    fn iny(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) == 0x80);

        0
    }

    fn jmp(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.pc = self.addr_abs;

        0
    }

    fn jsr(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.pc = self.pc.wrapping_sub(1);

        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            ((self.pc.wrapping_shr(8)) & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            (self.pc & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);

        self.pc = self.addr_abs;

        0
    }

    fn lda(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.a = self.fetched;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        1
    }

    fn ldx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.x = self.fetched;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) == 0x80);

        1
    }

    fn ldy(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.y = self.fetched;
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) == 0x80);

        1
    }

    fn lsr(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.set_flag(Flags::C, (self.fetched & 0x0001) == 0x0001);
        let temp = self.fetched.wrapping_shr(1);
        self.set_flag(Flags::Z, (temp as u16 & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp as u16 & 0x0080) == 0x0080);

        if (self.lookup[self.opcode as usize].addrmode) as usize == (Cpu::imp) as usize {
            self.a = temp & 0x00FF;
        } else {
            self.write(bus, ppu, cart, self.addr_abs, (temp as u16 & 0x00FF) as u8);
        }

        0
    }

    fn nop(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        match self.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => 1,
            _ => 0,
        }
    }

    fn ora(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        self.a |= self.fetched;

        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        1
    }

    fn pha(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.write(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16), self.a);
        self.stkp = self.stkp.wrapping_sub(1);

        0
    }

    fn php(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            self.status | Flags::B as u8 | Flags::U as u8,
        );
        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, false);
        self.stkp = self.stkp.wrapping_sub(1);

        0
    }

    fn pla(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.stkp = self.stkp.wrapping_add(1);
        self.a = self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16));
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        0
    }

    fn plp(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.stkp = self.stkp.wrapping_add(1);
        self.status = self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16));
        self.set_flag(Flags::U, true);

        0
    }

    fn rol(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);

        let temp = (self.fetched.wrapping_shl(1)) as u16 | self.get_flag(Flags::C) as u16;

        self.set_flag(Flags::C, (temp & 0xFF00) == 0xFF00);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x0000);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        if (self.lookup[self.opcode as usize].addrmode) as usize == (Cpu::imp) as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, ppu, cart, self.addr_abs, (temp & 0x00FF) as u8);
        }

        0
    }

    fn ror(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let temp = (self.get_flag(Flags::C).wrapping_shl(7)) as u16 | (self.fetched.wrapping_shr(1)) as u16;
        self.set_flag(Flags::C, (self.fetched & 0x01) == 0x01);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0x00FF);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);

        if (self.lookup[self.opcode as usize].addrmode) as usize == (Cpu::imp) as usize {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(bus, ppu, cart, self.addr_abs, (temp & 0x00FF) as u8);
        }

        0
    }

    fn rti(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.stkp = self.stkp.wrapping_add(1);
        self.status = self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16));
        self.status &= !(Flags::B as u8);
        self.status &= !(Flags::U as u8);

        self.stkp = self.stkp.wrapping_add(1);
        self.pc = self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16)) as u16;
        self.stkp = self.stkp.wrapping_add(1);
        self.pc |= (self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16)) as u16).wrapping_shl(8);

        0
    }

    fn rts(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.stkp = self.stkp.wrapping_add(1);
        self.pc = self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16)) as u16;
        self.stkp = self.stkp.wrapping_add(1);
        self.pc |= (self.read(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16)) as u16).wrapping_shl(8);

        self.pc = self.pc.wrapping_add(1);
        0
    }

    fn sbc(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.fetch(bus, ppu, cart);
        let value: u16 = (self.fetched as u16) ^ 0x00FF;

        let temp: u16 = (self.a as u16)
            .wrapping_add(value)
            .wrapping_add(self.get_flag(Flags::C) as u16);
        self.set_flag(Flags::C, (temp & 0xFF00) == 0xFF00);
        self.set_flag(Flags::Z, (temp & 0x00FF) == 0);
        self.set_flag(Flags::N, (temp & 0x0080) == 0x0080);
        self.set_flag(
            Flags::V,
            ((temp ^ self.a as u16) & (temp ^ value) & 0x0080) == 0x0080,
        );
        self.a = (temp & 0x00FF) as u8;

        1
    }

    fn sec(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::C, true);
        0
    }

    fn sed(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::D, true);
        0
    }

    fn sei(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.set_flag(Flags::I, true);
        0
    }

    fn sta(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.write(bus, ppu, cart, self.addr_abs, self.a);
        0
    }

    fn stx(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.write(bus, ppu, cart, self.addr_abs, self.x);
        0
    }

    fn sty(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        self.write(bus, ppu, cart, self.addr_abs, self.y);
        0
    }

    fn tax(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.x = self.a;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) == 0x80);

        0
    }

    fn tay(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.y = self.a;
        self.set_flag(Flags::Z, self.y == 0x00);
        self.set_flag(Flags::N, (self.y & 0x80) == 0x80);

        0
    }

    fn tsx(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.x = self.stkp;
        self.set_flag(Flags::Z, self.x == 0x00);
        self.set_flag(Flags::N, (self.x & 0x80) == 0x80);

        0
    }

    fn txa(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.a = self.x;
        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        0
    }

    fn txs(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.stkp = self.x;

        0
    }

    fn tya(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        self.a = self.y;

        self.set_flag(Flags::Z, self.a == 0x00);
        self.set_flag(Flags::N, (self.a & 0x80) == 0x80);

        0
    }

    fn xxx(&mut self, _bus: &mut Bus, _ppu: &mut Ppu, _cart: &mut Cartridge) -> u8 {
        0
    }

    pub fn clock(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) {
        if self.cycles == 0 {
            self.opcode = self.read(bus, ppu, cart, self.pc);
            self.set_flag(Flags::U, true);
            self.pc = self.pc.wrapping_add(1);

            self.cycles = self.lookup[self.opcode as usize].cycles;
            let additional_cycle1 =
                (self.lookup[self.opcode as usize].addrmode)(self, bus, ppu, cart);
            let additional_cycle2 =
                (self.lookup[self.opcode as usize].operate)(self, bus, ppu, cart);

            self.cycles = self.cycles.wrapping_add(additional_cycle1 & additional_cycle2);

            self.set_flag(Flags::U, true)
        }

        self.clock_count += 1;
        self.cycles = self.cycles.wrapping_sub(1);
    }

    pub fn reset(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) {
        self.addr_abs = 0xFFFC;
        let lo: u16 = self.read(bus, ppu, cart, self.addr_abs) as u16;
        let hi: u16 = self.read(bus, ppu, cart, self.addr_abs.wrapping_add(1)) as u16;
        self.pc = hi.wrapping_shl(8) | lo;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stkp = 0xFD;
        self.status = 0x00 | Flags::U as u8;

        self.addr_abs = 0x0000;
        self.addr_rel = 0x0000;
        self.fetched = 0x00;

        self.cycles = 8;
    }

    pub fn irq(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) {
        if self.get_flag(Flags::I) == 0 {
            self.write(
                bus,
                ppu,
                cart,
                (0x0100 as u16).wrapping_add(self.stkp as u16),
                ((self.pc.wrapping_shr(8)) & 0x00FF) as u8,
            );
            self.stkp = self.stkp.wrapping_sub(1);
            self.write(
                bus,
                ppu,
                cart,
                (0x0100 as u16).wrapping_add(self.stkp as u16),
                (self.pc & 0x00FF) as u8,
            );
            self.stkp = self.stkp.wrapping_sub(1);

            self.set_flag(Flags::B, false);
            self.set_flag(Flags::U, true);
            self.set_flag(Flags::I, true);
            self.write(
                bus,
                ppu,
                cart,
                (0x0100 as u16).wrapping_add(self.stkp as u16),
                self.status,
            );
            self.stkp = self.stkp.wrapping_sub(1);

            self.addr_abs = 0xFFFE;
            self.pc = self.read(bus, ppu, cart, self.addr_abs) as u16;
            self.pc |= (self.read(bus, ppu, cart, self.addr_abs.wrapping_add(1)) as u16).wrapping_shr(8);

            self.cycles = 7;
        }
    }

    pub fn nmi(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) {
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            ((self.pc.wrapping_shr(8)) & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);
        self.write(
            bus,
            ppu,
            cart,
            (0x0100 as u16).wrapping_add(self.stkp as u16),
            (self.pc & 0x00FF) as u8,
        );
        self.stkp = self.stkp.wrapping_sub(1);

        self.set_flag(Flags::B, false);
        self.set_flag(Flags::U, true);
        self.set_flag(Flags::I, true);
        self.write(bus, ppu, cart, (0x0100 as u16).wrapping_add(self.stkp as u16), self.status);
        self.stkp = self.stkp.wrapping_sub(1);

        self.addr_abs = 0xFFFA;
        self.pc = self.read(bus, ppu, cart, self.addr_abs) as u16;
        self.pc |= (self.read(bus, ppu, cart, self.addr_abs.wrapping_add(1)) as u16).wrapping_shl(8);

        self.cycles = 8;
    }

    fn fetch(&mut self, bus: &mut Bus, ppu: &mut Ppu, cart: &mut Cartridge) -> u8 {
        if !(self.lookup[self.opcode as usize].addrmode as usize == Cpu::imp as usize) {
            self.fetched = self.read(bus, ppu, cart, self.addr_abs);
        }

        self.fetched
    }

    pub fn disassemble(
        &self,
        n_start: u16,
        n_stop: u16,
        bus: &mut Bus,
        ppu: &mut Ppu,
        cart: &mut Cartridge,
    ) -> BTreeMap<u16, String> {
        let mut addr: u32 = n_start as u32;
        let mut value: u8;
        let mut lo: u8;
        let mut hi: u8;
        let mut map = BTreeMap::new();
        let mut line_addr: u16;

        while addr <= n_stop as u32 {
            line_addr = addr as u16;

            let mut s_inst: String = String::from("$") + &format!("{:04X}", addr)[..] + ": ";

            let opcode: u8 = bus.cpu_read(ppu, cart, addr as u16, true);
            addr += 1;
            s_inst.push_str(&self.lookup[opcode as usize].name[..]);
            s_inst.push_str(" ");

            if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::imp) as usize {
                s_inst.push_str(" {IMP}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::imm) as usize {
                value = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("#$");
                s_inst.push_str(&format!("{:02X}", value)[..]);
                s_inst.push_str(" {IMM}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::zp0) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:02X}", lo)[..]);
                s_inst.push_str(" {ZP0}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::zpx) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:02X}", lo)[..]);
                s_inst.push_str(", X {ZPX}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::zpy) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:02X}", lo)[..]);
                s_inst.push_str(", Y {ZPY}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::izx) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("($");
                s_inst.push_str(&format!("{:02X}", lo)[..]);
                s_inst.push_str(", X) {IZX}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::izy) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("($");
                s_inst.push_str(&format!("{:02X}", lo)[..]);
                s_inst.push_str("), Y {IZY}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::abs) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                hi = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:04X}", ((hi as u16).wrapping_shl(8) | lo as u16))[..]);
                s_inst.push_str(" {ABS}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::abx) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                hi = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:04X}", ((hi as u16).wrapping_shl(8) | lo as u16))[..]);
                s_inst.push_str(", X {ABX}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::aby) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                hi = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:04X}", ((hi as u16).wrapping_shl(8) | lo as u16))[..]);
                s_inst.push_str(", Y {ABY}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::ind) as usize {
                lo = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                hi = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("($");
                s_inst.push_str(&format!("{:04X}", ((hi as u16).wrapping_shl(8) | lo as u16))[..]);
                s_inst.push_str(") {IND}");
            } else if (self.lookup[opcode as usize].addrmode) as usize == (Cpu::rel) as usize {
                value = bus.cpu_read(ppu, cart, addr as u16, true);
                addr += 1;
                s_inst.push_str("$");
                s_inst.push_str(&format!("{:02X}", value)[..]);
                s_inst.push_str(" [$");
                s_inst.push_str(&format!("{:04X}", addr as i32 + (value as i8) as i32)[..]);
                s_inst.push_str("] {REL}");
            }

            map.insert(line_addr, s_inst);
        }

        map
    }

    pub fn complete(&self) -> bool {
        self.cycles == 0
    }

    pub fn load_program(
        &mut self,
        bus: &mut Bus,
        mut n_offset: u16,
        program: Vec<u8>,
        reset_lo: u8,
        reset_hi: u8,
    ) {
        for i in program.iter() {
            bus.cpu_ram[n_offset as usize] = *i;
            n_offset += 1;
        }

        bus.cpu_ram[0xFFFC] = reset_lo;
        bus.cpu_ram[0xFFFD] = reset_hi;
    }

    pub fn draw_ram(
        &self,
        bus: &mut Bus,
        ppu: &mut Ppu,
        cart: &mut Cartridge,
        x: i32,
        y: i32,
        mut n_addr: u16,
        n_rows: i32,
        n_columns: i32,
    ) {
        let n_ram_x = x;
        let mut n_ram_y = y;
        for _row in 0..n_rows {
            let mut s_offset = String::from("$");
            s_offset.push_str(&format!("{:04X}", n_addr)[..]);
            for _col in 0..n_columns {
                s_offset.push_str(" ");
                s_offset.push_str(&format!("{:02X}", bus.cpu_read(ppu, cart, n_addr, true))[..]);
                n_addr += 1;
            }
            draw_text(&s_offset[..], n_ram_x as f32, n_ram_y as f32, 25.0, WHITE);
            n_ram_y += 15;
        }
    }

    pub fn draw_cpu(&self, mut x: i32, y: i32) {
        draw_text("STATUS:", x as f32, y as f32, 25.0, WHITE);

        x += 15;

        if (self.status & Flags::N as u8) == Flags::N as u8 {
            draw_text("N", (x + 64) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("N", (x + 64) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::V as u8) == Flags::V as u8 {
            draw_text("V", (x + 80) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("V", (x + 80) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::U as u8) == Flags::U as u8 {
            draw_text("-", (x + 96) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("-", (x + 96) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::B as u8) == Flags::B as u8 {
            draw_text("B", (x + 112) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("B", (x + 112) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::D as u8) == Flags::D as u8 {
            draw_text("D", (x + 128) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("D", (x + 128) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::I as u8) == Flags::I as u8 {
            draw_text("I", (x + 144) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("I", (x + 144) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::Z as u8) == Flags::Z as u8 {
            draw_text("Z", (x + 160) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("Z", (x + 160) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::C as u8) == Flags::C as u8 {
            draw_text("C", (x + 178) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("C", (x + 178) as f32, y as f32, 25.0, RED);
        }

        if (self.status & Flags::N as u8) == Flags::N as u8 {
            draw_text("N", (x + 64) as f32, y as f32, 25.0, GREEN);
        } else {
            draw_text("N", (x + 64) as f32, y as f32, 25.0, RED);
        }

        let mut temp = String::from("PC: $");
        temp.push_str(&format!("{:04X}", self.pc)[..]);
        draw_text(&temp[..], x as f32, (y + 15) as f32, 25.0, WHITE);

        temp = String::from("A: $");
        temp.push_str(&format!("{:02X}", self.a)[..]);
        temp.push_str("  [");
        temp.push_str(self.a.to_string().as_str());
        temp.push_str("]");
        draw_text(&temp[..], x as f32, (y + 30) as f32, 25.0, WHITE);

        temp = String::from("X: $");
        temp.push_str(&format!("{:02X}", self.x)[..]);
        temp.push_str("  [");
        temp.push_str(self.x.to_string().as_str());
        temp.push_str("]");
        draw_text(&temp[..], x as f32, (y + 45) as f32, 25.0, WHITE);

        temp = String::from("Y: $");
        temp.push_str(&format!("{:02X}", self.y)[..]);
        temp.push_str("  [");
        temp.push_str(self.y.to_string().as_str());
        temp.push_str("]");
        draw_text(&temp[..], x as f32, (y + 60) as f32, 25.0, WHITE);

        temp = String::from("Stack P: $");
        temp.push_str(&format!("{:04X}", self.stkp)[..]);
        draw_text(&temp[..], x as f32, (y + 75) as f32, 25.0, WHITE);
    }

    pub fn draw_code(
        &self,
        pc: &u16,
        x: i64,
        y: i64,
        n_lines: i64,
        map_asm: &BTreeMap<u16, String>,
    ) {
        let mut n_line_y: i64 = ((n_lines.wrapping_shr(1)) * 10) + y;
        let mut it_a = map_asm.range(..);

        if let Some(instruction) = it_a.find(|(k, _v)| k == &pc) {
            draw_text(instruction.1, x as f32, n_line_y as f32, 25.0, CYAN);
            while n_line_y < ((n_lines * 10) + y) {
                n_line_y += 17;
                if let Some(instruction) = it_a.next() {
                    draw_text(instruction.1, x as f32, n_line_y as f32, 25.0, WHITE);
                }
            }
        }

        let mut n_line_y: i64 = ((n_lines.wrapping_shr(1)) * 10) + y;
        let mut it_a = map_asm.range(..).rev();
        if let Some(_) = it_a.find(|(k, _v)| k == &pc) {
            while n_line_y > y {
                n_line_y -= 17;
                if let Some(instruction) = it_a.next() {
                    draw_text(instruction.1, x as f32, n_line_y as f32, 25.0, WHITE);
                }
            }
        }
    }
}
