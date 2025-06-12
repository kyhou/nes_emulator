use crate::{Cartridge, Cpu, Ppu};

pub struct Bus {
    pub cpu_ram: [u8; 2 * 1024],
    pub controller: [u8; 2],
    system_clock_counter: i32,
    controller_state: [u8; 2],
    dma_page: u8,
    dma_addr: u8,
    dma_data: u8,
    dma_transfer: bool,
    dma_dummy: bool,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            cpu_ram: [0; 2 * 1024],
            controller: [0; 2],
            system_clock_counter: 0,
            controller_state: [0; 2],
            dma_page: 0x00,
            dma_addr: 0x00,
            dma_data: 0x00,
            dma_transfer: false,
            dma_dummy: false,
        }
    }

    pub fn cpu_write(
        &mut self,
        ppu: &mut Ppu,
        cart: &mut Cartridge,
        addr: u16,
        data: u8,
    ) {
        if cart.cpu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.cpu_ram[(addr & 0x07FF) as usize] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            ppu.cpu_write(cart, addr & 0x0007, data);
        } else if addr == 0x4014 {
            self.dma_page = data;
            self.dma_addr = 0x00;
            self.dma_transfer = true;
        } else if addr >= 0x4016 && addr <= 0x4017 {
            self.controller_state[(addr & 0x0001) as usize] =
                self.controller[(addr & 0x0001) as usize];
        }
    }

    pub fn cpu_read(
        &mut self,
        ppu: &mut Ppu,
        cart: &mut Cartridge,
        addr: u16,
        read_only: bool,
    ) -> u8 {
        let mut data: u8 = 0x00;

        if cart.cpu_read(addr, &mut data) {
        } else if addr <= 0x1FFF {
            return self.cpu_ram[(addr & 0x07FF) as usize];
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            data = ppu.cpu_read(cart, addr & 0x0007, read_only);
        } else if addr >= 0x4016 && addr <= 0x4017 {
            data = ((self.controller_state[(addr & 0x0001) as usize] & 0x80) > 0) as u8;
            self.controller_state[(addr & 0x0001) as usize] =
                self.controller_state[(addr & 0x0001) as usize].wrapping_shl(1);
        }

        data
    }

    pub fn reset(&mut self, cpu: &mut Cpu, ppu: &mut Ppu, cart: &mut Cartridge) {
        cart.reset();
        cpu.reset(self, ppu, cart);
        ppu.reset();
        self.system_clock_counter = 0;
        self.dma_page = 0x00;
        self.dma_addr = 0x00;
        self.dma_data = 0x00;
        self.dma_dummy = true;
        self.dma_transfer = false;
    }

    pub fn clock(&mut self, cpu: &mut Cpu, ppu: &mut Ppu, cart: &mut Cartridge) {
        ppu.clock(cart);

        if self.system_clock_counter % 3 == 0 {
            if self.dma_transfer {
                if self.dma_dummy {
                    if self.system_clock_counter % 2 == 1 {
                        self.dma_dummy = false;
                    }
                } else if self.system_clock_counter % 2 == 0 {
                        self.dma_data = self.cpu_read(
                            ppu,
                            cart,
                            (self.dma_page as u16).wrapping_shl(8) | self.dma_addr as u16,
                            false,
                        )
                    } else {
                        match self.dma_addr % 4 {
                            0 => {
                            ppu.oam[(self.dma_addr / 4) as usize].y = self.dma_data;
                            }
                            1 => {
                                ppu.oam[(self.dma_addr / 4) as usize].id = self.dma_data;
                            }
                            2 => {
                                ppu.oam[(self.dma_addr / 4) as usize].attribute = self.dma_data;
                            }
                            3 => {
                                ppu.oam[(self.dma_addr / 4) as usize].x = self.dma_data;
                            }
                            _ => (),
                        }
                        self.dma_addr = self.dma_addr.wrapping_add(1);

                        if self.dma_addr == 0x00 {
                            self.dma_transfer = false;
                            self.dma_dummy = true;
                    }
                }
            } else {
                cpu.clock(self, ppu, cart);
            }
        }

        if ppu.nmi {
            ppu.nmi = false;
            cpu.nmi(self, ppu, cart);
        }

        if cart.get_mapper().borrow().irq_state() {
            cart.get_mapper().borrow_mut().irq_clear();
            cpu.irq(self, ppu, cart);
        }

        self.system_clock_counter += 1;
    }
}
