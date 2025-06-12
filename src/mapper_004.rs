use crate::{
    cartridge::{Cartridge, Mirror},
    mapper::{Mapper, RW},
};

pub struct Mapper004 {
    mapper: Mapper,
    target_register: u8,
    prg_bank_mode: bool,
    chr_inversion: bool,
    mirror_mode: Mirror,
    register: [u32; 8],
    chr_bank: [u32; 8],
    prg_bank: [u32; 4],
    irq_active: bool,
    irq_enable: bool,
    irq_update: bool,
    irq_counter: u16,
    irq_reload: u16,
    ram_static: Vec<u8>,
}
impl Mapper004 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Self {
        let mut mapper = Mapper004 {
            mapper: Mapper::new(prg_banks, chr_banks),
            target_register: 0x00,
            prg_bank_mode: false,
            chr_inversion: false,
            mirror_mode: Mirror::Horizontal,
            register: [0; 8],
            chr_bank: [0; 8],
            prg_bank: [0; 4],
            irq_active: false,
            irq_enable: false,
            irq_update: false,
            irq_counter: 0x0000,
            irq_reload: 0x0000,
            ram_static: Vec::new(),
        };
        mapper.ram_static.resize(32 * 1024, 0);
        mapper.reset();

        mapper
    }
}

impl RW for Mapper004 {
    fn cpu_map_read(&self, addr: u16, mapped_addr: &mut u32, data: &mut u8) -> bool {
        if addr >= 0x6000 && addr <= 0x7FFF {
            *mapped_addr = 0xFFFFFFFF;
            *data = self.ram_static[(addr & 0x1FFF) as usize];
            return true;
        }

        if addr >= 0x8000 && addr <= 0x9FFF {
            *mapped_addr = self.prg_bank[0] + (addr & 0x1FFF) as u32;
            return true;
        }

        if addr >= 0xA000 && addr <= 0xBFFF {
            *mapped_addr = self.prg_bank[1] + (addr & 0x1FFF) as u32;
            return true;
        }

        if addr >= 0xC000 && addr <= 0xDFFF {
            *mapped_addr = self.prg_bank[2] + (addr & 0x1FFF) as u32;
            return true;
        }

        if addr >= 0xE000 {
            *mapped_addr = self.prg_bank[3] + (addr & 0x1FFF) as u32;
            return true;
        }

        false
    }

    fn cpu_map_write(&mut self, addr: u16, mapped_addr: &mut u32, data: &u8) -> bool {
        if addr >= 0x6000 && addr <= 0x7FFF {
            *mapped_addr = 0xFFFFFFFF;
            self.ram_static[(addr & 0x1FFF) as usize] = *data;
            return true;
        }

        if addr >= 0x8000 && addr <= 0x9FFF {
            if (addr & 0x0001) == 0x0000 {
                self.target_register = data & 0x07;
                self.prg_bank_mode = (data & 0x40) == 0x40;
                self.chr_inversion = (data & 0x80) == 0x80;
            } else {
                self.register[self.target_register as usize] = *data as u32;
            }

            if self.chr_inversion {
                self.chr_bank[0] = self.register[2] * 0x0400;
                self.chr_bank[1] = self.register[3] * 0x0400;
                self.chr_bank[2] = self.register[4] * 0x0400;
                self.chr_bank[3] = self.register[5] * 0x0400;
                self.chr_bank[4] = (self.register[0] & 0xFE) * 0x0400;
                self.chr_bank[5] = self.register[0] * 0x0400 + 0x0400;
                self.chr_bank[6] = (self.register[1] & 0xFE) * 0x0400;
                self.chr_bank[7] = self.register[1] * 0x0400 + 0x0400;
            } else {
                self.chr_bank[0] = (self.register[0] & 0xFE) * 0x0400;
                self.chr_bank[1] = self.register[0] * 0x0400 + 0x0400;
                self.chr_bank[2] = (self.register[1] & 0xFE) * 0x0400;
                self.chr_bank[3] = self.register[1] * 0x0400 + 0x0400;
                self.chr_bank[4] = self.register[2] * 0x0400;
                self.chr_bank[5] = self.register[3] * 0x0400;
                self.chr_bank[6] = self.register[4] * 0x0400;
                self.chr_bank[7] = self.register[5] * 0x0400;
            }

            let prg_banks = (self.mapper.prg_banks * 2 - 1) as u32;

            if self.prg_bank_mode {
                self.prg_bank[2] = (self.register[6] & prg_banks) * 0x2000;
                self.prg_bank[0] = (self.mapper.prg_banks * 2 - 2) as u32 * 0x2000;
            } else {
                self.prg_bank[0] = (self.register[6] & prg_banks) * 0x2000;
                self.prg_bank[2] = (self.mapper.prg_banks * 2 - 2) as u32 * 0x2000;
            }

            self.prg_bank[1] = (self.register[7] & prg_banks) * 0x2000;
            self.prg_bank[3] = prg_banks * 0x2000;

            return false;
        }

        if addr >= 0xA000 && addr <= 0xBFFF {
            if (addr & 0x0001) == 0x0000 {
                if (data & 0x01) == 0x01 {
                    self.mirror_mode = Mirror::Horizontal;
                } else {
                    self.mirror_mode = Mirror::Vertical;
                }
            } else {
                //TODO: PRG Ram Protect
            }

            return false;
        }

        if addr >= 0xC000 && addr <= 0xDFFF {
            if (addr & 0x0001) == 0x0000 {
                self.irq_reload = *data as u16;
            } else {
                self.irq_counter = 0x0000;
            }

            return false;
        }

        if addr >= 0xE000 {
            if (addr & 0x0001) == 0x0000 {
                self.irq_enable = false;
                self.irq_active = false;
            } else {
                self.irq_enable = true;
            }

            return false;
        }

        false
    }

    fn ppu_map_read(&self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x03FF {
            *mapped_addr = self.chr_bank[0] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x0400 && addr <= 0x07FF {
            *mapped_addr = self.chr_bank[1] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x0800 && addr <= 0x0BFF {
            *mapped_addr = self.chr_bank[2] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x0C00 && addr <= 0x0FFF {
            *mapped_addr = self.chr_bank[3] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x1000 && addr <= 0x13FF {
            *mapped_addr = self.chr_bank[4] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x1400 && addr <= 0x17FF {
            *mapped_addr = self.chr_bank[5] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x1800 && addr <= 0x1BFF {
            *mapped_addr = self.chr_bank[6] + (addr & 0x03FF) as u32;
            return true;
        }

        if addr >= 0x1C00 && addr <= 0x1FFF {
            *mapped_addr = self.chr_bank[7] + (addr & 0x03FF) as u32;
            return true;
        }

        false
    }

    fn ppu_map_write(&self, _cart: &Cartridge, _addr: u16, _mapped_addr: &mut u32) -> bool {
        false
    }

    fn reset(&mut self) {
        self.target_register = 0;
        self.prg_bank_mode = false;
        self.chr_inversion = false;
        self.mirror_mode = Mirror::Horizontal;

        self.irq_active = false;
        self.irq_enable = false;
        self.irq_update = false;
        self.irq_counter = 0x0000;
        self.irq_reload = 0x0000;

        for i in 0..4 {
            self.prg_bank[i] = 0x0000;
        }

        for i in 0..8 {
            self.chr_bank[i] = 0x0000;
            self.register[i] = 0xFFFF;
        }

        self.prg_bank[0] = 0 * 0x2000;
        self.prg_bank[1] = 1 * 0x2000;
        self.prg_bank[2] = (self.mapper.prg_banks * 2 - 2) as u32 * 0x2000;
        self.prg_bank[3] = (self.mapper.prg_banks * 2 - 1) as u32 * 0x2000;
    }

    fn irq_state(&self) -> bool {
        self.irq_active
    }

    fn irq_clear(&mut self) {
        self.irq_active = false;
    }

    fn scanline(&mut self) {
        if self.irq_counter == 0 {
            self.irq_counter = self.irq_reload;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_counter == 0 && self.irq_enable {
            self.irq_active = true;
        }
    }

    fn mirror(&self) -> Mirror {
        self.mirror_mode.clone()
    }
}
