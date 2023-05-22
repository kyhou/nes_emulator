use crate::{Cartridge, Cpu, Ppu};

pub struct Bus {
    pub cpu_ram: [u8; 2 * 1024],
    pub controller: [u8; 2],
    n_system_clock_counter: i32,
    controller_state: [u8; 2],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cpu_ram: [0; 2 * 1024],
            controller: [0;2],
            n_system_clock_counter: 0,
            controller_state: [0; 2],
        }
    }

    pub fn cpu_write(&mut self, ppu: &mut Ppu, cart: &mut Cartridge, addr: u16, data: u8) {
        if cart.cpu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.cpu_ram[(addr & 0x07FF) as usize] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            ppu.cpu_write(cart, addr & 0x0007, data);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            self.controller_state[(addr & 0x0001) as usize] = self.controller[(addr & 0x0001) as usize];
        }
    }

    pub fn cpu_read(&mut self, ppu: &mut Ppu, cart: &mut Cartridge, addr: u16, b_read_only: bool) -> u8 {
        let mut data: u8 = 0x00;

        if cart.cpu_read(addr, &mut data) {
        } else if addr <= 0x1FFF {
            return self.cpu_ram[(addr & 0x07FF) as usize];
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            data = ppu.cpu_read(cart, addr & 0x007, b_read_only);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            data = ((self.controller_state[(addr & 0x0001) as usize] & 0x80) > 0) as u8;
            self.controller_state[(addr & 0x0001) as usize] <<= 1;
        }

        data
    }

    pub fn reset(&mut self, cpu: &mut Cpu, ppu: &mut Ppu, cart: &mut Cartridge) {
        cpu.reset(self, ppu, cart);
        ppu.reset();
        // cart.reset();
        self.n_system_clock_counter = 0;
    }

    pub fn clock(&mut self, cpu: &mut Cpu, ppu: &mut Ppu, cart: &mut Cartridge) {
        ppu.clock(cart);

        if self.n_system_clock_counter % 3 == 0 {
            cpu.clock(self, ppu, cart);
        }

        if ppu.nmi {
            ppu.nmi = false;
            cpu.nmi(self, ppu, cart);
        }

        self.n_system_clock_counter += 1;
    }
}
