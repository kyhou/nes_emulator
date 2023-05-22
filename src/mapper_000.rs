use crate::{
    cartridge::Cartridge,
    mapper::{Mapper, RW},
};

pub struct Mapper000 {
    mapper: Mapper,
}
impl Mapper000 {
    pub(crate) fn new(prg_banks: u8, chr_banks: u8) -> Self {
        Mapper000 {
            mapper: Mapper::new(prg_banks, chr_banks),
        }
    }
}

impl RW for Mapper000 {
    fn cpu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            *mapped_addr = (addr
                & (if self.mapper.prg_banks > 1 {
                    0x07FFF
                } else {
                    0x3FFF
                })) as u32;
            true
        } else {
            false
        }
    }

    fn cpu_map_write(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            *mapped_addr = (addr
                & (if self.mapper.prg_banks > 1 {
                    0x07FFF
                } else {
                    0x3FFF
                })) as u32;
            true
        } else {
            false
        }
    }

    fn ppu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            true
        } else {
            false
        }
    }

    fn ppu_map_write(&self, cart: &Cartridge, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x1FFF {
            if cart.chr_banks == 0 {
                *mapped_addr = addr as u32;
                return true;
            }
        }
        return false;
    }

    fn reset(&self) {}
}
