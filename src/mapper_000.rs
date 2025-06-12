use crate::{
    cartridge::Cartridge,
    mapper::{Mapper, RW},
};

pub struct Mapper000 {
    mapper: Mapper,
}
impl Mapper000 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        let mut mapper = Mapper000 {
            mapper: Mapper::new(prg_banks, chr_banks),
        };

        mapper.reset();

        mapper
    }
}

impl RW for Mapper000 {
    fn cpu_map_read(&self, addr: u16, mapped_addr: &mut u32, _data: &mut u8) -> bool {
        if addr >= 0x8000 {
            *mapped_addr = (addr
                & (if self.mapper.prg_banks > 1 {
                    0x7FFF
                } else {
                    0x3FFF
                })) as u32;
            true
        } else {
            false
        }
    }

    fn cpu_map_write(&mut self, addr: u16, mapped_addr: &mut u32, _data: &u8) -> bool {
        if addr >= 0x8000 {
            *mapped_addr = (addr
                & (if self.mapper.prg_banks > 1 {
                    0x7FFF
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
        false
    }

    fn reset(&mut self) {}

    fn irq_state(&self) -> bool {
        false
    }

    fn irq_clear(&mut self) {}

    fn scanline(&mut self) {}

    fn mirror(&self) -> crate::cartridge::Mirror {
        crate::cartridge::Mirror::Hardware
    }
}
