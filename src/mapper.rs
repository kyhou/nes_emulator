use crate::cartridge::{Cartridge, Mirror};

pub struct Mapper {
    pub prg_banks: u8,
    pub chr_banks: u8,
}

pub trait RW {
    fn cpu_map_read(&self, addr: u16, mapped_addr: &mut u32, data: &mut u8) -> bool;
    fn cpu_map_write(&mut self, addr: u16, mapped_addr: &mut u32, data: &u8) -> bool;
    fn ppu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool;
    fn ppu_map_write(&self, cart: &Cartridge, addr: u16, mapped_addr: &mut u32) -> bool;
    fn reset(&mut self);

    fn irq_state(&self) -> bool;
    fn irq_clear(&mut self);

    fn scanline(&mut self);
    fn mirror(&self) -> Mirror;
}

impl Mapper {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Mapper {
            prg_banks,
            chr_banks,
        }
    }
}
