use bitfield::bitfield;
use macroquad::prelude::*;

use crate::{cartridge, Cartridge};

pub struct Ppu {
    tbl_name: [[u8; 1024]; 2],
    tbl_palette: [u8; 32],
    tbl_pattern: [[u8; 4096]; 2], // Javid Future
    pallete_screen: [Color; 0x40],
    sprite_screen: Image,
    sprite_name_table: [Image; 2],
    sprite_pattern_table: [Image; 2],
    pub frame_complete: bool,
    scanline: i16,
    cycle: i16,
    status: Status,
    mask: Mask,
    control: PpuControl,
    address_latch: u8,
    data_buffer: u8,
    pub nmi: bool,
    vram_addr: LoopyRegister,
    tram_addr: LoopyRegister,
    fine_x: u8,
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,
}

bitfield! {
    pub struct Status(u8);
    impl Debug;
    u8;
    unused, _: 4, 0;
    sprite_overflow, _: 5;
    sprite_zero_hit, _: 6;
    vertical_blank, set_vertical_blank: 7;
}

bitfield! {
    pub struct Mask(u8);
    impl Debug;
    u8;
    grayscale, _: 0;
    render_background_left, _: 1;
    render_sprites_left, _: 2;
    render_background, _: 3;
    render_sprites, _: 4;
    enhance_red, _: 5;
    enhance_green, _: 6;
    enhance_blue, _: 7;
}

bitfield! {
    pub struct PpuControl(u8);
    impl Debug;
    u8;
    nametable_x, set_nametable_x: 0;
    nametable_y, set_nametable_y: 1;
    increment_mode, set_increment_mode: 2;
    pattern_sprite, set_pattern_sprite: 3;
    pattern_background, set_pattern_background: 4;
    sprite_size, set_sprite_size: 5;
    slave_mode, set_slave_mode: 6; // unused
    enable_nmi, set_enable_nmi: 7;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct LoopyRegister(u16);
    impl Debug;
    u16;
    coarse_x, set_coarse_x: 4, 0;
    coarse_y, set_coarse_y: 9, 5;
    nametable_x, set_nametable_x: 10;
    nametable_y, set_nametable_y: 11;
    fine_y, set_fine_y: 14, 12;
    unused, _: 15;
}

pub trait Debug {
    fn get_screen(&self) -> &Image;
    fn get_name_table(&self, i: u8) -> &Image;
    fn get_pattern_table(&mut self, i: u8, pallet: &u8, cart: &mut Cartridge) -> &Image;
}

impl Debug for Ppu {
    fn get_screen(&self) -> &Image {
        &self.sprite_screen
    }

    fn get_name_table(&self, i: u8) -> &Image {
        &self.sprite_name_table[i as usize]
    }

    fn get_pattern_table(&mut self, i: u8, pallete: &u8, cart: &mut Cartridge) -> &Image {
        for tile_y in 0u16..16 {
            for tile_x in 0u16..16 {
                let offset: u16 = tile_y
                    .wrapping_mul(self.get_screen().width)
                    .wrapping_add(tile_x.wrapping_mul(16));

                for row in 0u16..8 {
                    let mut tile_lsb: u8 = self.ppu_read(
                        cart,
                        (i as u16)
                            .wrapping_mul(0x1000)
                            .wrapping_add(offset as u16)
                            .wrapping_add(row as u16),
                        false,
                    );

                    let mut tile_msb: u8 = self.ppu_read(
                        cart,
                        (i as u16)
                            .wrapping_mul(0x1000)
                            .wrapping_add(offset as u16)
                            .wrapping_add(row as u16)
                            .wrapping_add(0x0008),
                        false,
                    );

                    for col in 0u16..8 {
                        let pixel: u8 = (tile_lsb & 0x01).wrapping_shl(1) | (tile_msb & 0x01);
                        tile_lsb = tile_lsb.wrapping_shr(1);
                        tile_msb = tile_msb.wrapping_shr(1);

                        self.sprite_pattern_table[i as usize].set_pixel(
                            tile_x
                                .wrapping_mul(8)
                                .wrapping_add((7 as u16).wrapping_sub(col))
                                as u32,
                            tile_y.wrapping_mul(8).wrapping_add(row) as u32,
                            self.get_colour_from_pallet_ram(cart, &pallete, &pixel),
                        );
                    }
                }
            }
        }

        &self.sprite_pattern_table[i as usize]
    }
}

impl Ppu {
    pub fn new() -> Self {
        let mut pallet = [BLACK; 64];
        pallet[0x00] = Color::from_rgba(84, 84, 84, 255);
        pallet[0x01] = Color::from_rgba(0, 30, 116, 255);
        pallet[0x02] = Color::from_rgba(8, 16, 144, 255);
        pallet[0x03] = Color::from_rgba(48, 0, 136, 255);
        pallet[0x04] = Color::from_rgba(68, 0, 100, 255);
        pallet[0x05] = Color::from_rgba(92, 0, 48, 255);
        pallet[0x06] = Color::from_rgba(84, 4, 0, 255);
        pallet[0x07] = Color::from_rgba(60, 24, 0, 255);
        pallet[0x08] = Color::from_rgba(32, 42, 0, 255);
        pallet[0x09] = Color::from_rgba(8, 58, 0, 255);
        pallet[0x0A] = Color::from_rgba(0, 64, 0, 255);
        pallet[0x0B] = Color::from_rgba(0, 60, 0, 255);
        pallet[0x0C] = Color::from_rgba(0, 50, 60, 255);
        pallet[0x0D] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x0E] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x0F] = Color::from_rgba(0, 0, 0, 255);

        pallet[0x10] = Color::from_rgba(152, 150, 152, 255);
        pallet[0x11] = Color::from_rgba(8, 76, 196, 255);
        pallet[0x12] = Color::from_rgba(48, 50, 236, 255);
        pallet[0x13] = Color::from_rgba(92, 30, 228, 255);
        pallet[0x14] = Color::from_rgba(136, 20, 176, 255);
        pallet[0x15] = Color::from_rgba(160, 20, 100, 255);
        pallet[0x16] = Color::from_rgba(152, 34, 32, 255);
        pallet[0x17] = Color::from_rgba(120, 60, 0, 255);
        pallet[0x18] = Color::from_rgba(84, 90, 0, 255);
        pallet[0x19] = Color::from_rgba(40, 114, 0, 255);
        pallet[0x1A] = Color::from_rgba(8, 124, 0, 255);
        pallet[0x1B] = Color::from_rgba(0, 118, 40, 255);
        pallet[0x1C] = Color::from_rgba(0, 102, 120, 255);
        pallet[0x1D] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x1E] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x1F] = Color::from_rgba(0, 0, 0, 255);

        pallet[0x20] = Color::from_rgba(236, 238, 236, 255);
        pallet[0x21] = Color::from_rgba(76, 154, 236, 255);
        pallet[0x22] = Color::from_rgba(120, 124, 236, 255);
        pallet[0x23] = Color::from_rgba(176, 98, 236, 255);
        pallet[0x24] = Color::from_rgba(228, 84, 236, 255);
        pallet[0x25] = Color::from_rgba(236, 88, 180, 255);
        pallet[0x26] = Color::from_rgba(236, 106, 100, 255);
        pallet[0x27] = Color::from_rgba(212, 136, 32, 255);
        pallet[0x28] = Color::from_rgba(160, 170, 0, 255);
        pallet[0x29] = Color::from_rgba(116, 196, 0, 255);
        pallet[0x2A] = Color::from_rgba(76, 208, 32, 255);
        pallet[0x2B] = Color::from_rgba(56, 204, 108, 255);
        pallet[0x2C] = Color::from_rgba(56, 180, 204, 255);
        pallet[0x2D] = Color::from_rgba(60, 60, 60, 255);
        pallet[0x2E] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x2F] = Color::from_rgba(0, 0, 0, 255);

        pallet[0x30] = Color::from_rgba(236, 238, 236, 255);
        pallet[0x31] = Color::from_rgba(168, 204, 236, 255);
        pallet[0x32] = Color::from_rgba(188, 188, 236, 255);
        pallet[0x33] = Color::from_rgba(212, 178, 236, 255);
        pallet[0x34] = Color::from_rgba(236, 174, 236, 255);
        pallet[0x35] = Color::from_rgba(236, 174, 212, 255);
        pallet[0x36] = Color::from_rgba(236, 180, 176, 255);
        pallet[0x37] = Color::from_rgba(228, 196, 144, 255);
        pallet[0x38] = Color::from_rgba(204, 210, 120, 255);
        pallet[0x39] = Color::from_rgba(180, 222, 120, 255);
        pallet[0x3A] = Color::from_rgba(168, 226, 144, 255);
        pallet[0x3B] = Color::from_rgba(152, 226, 180, 255);
        pallet[0x3C] = Color::from_rgba(160, 214, 228, 255);
        pallet[0x3D] = Color::from_rgba(160, 162, 160, 255);
        pallet[0x3E] = Color::from_rgba(0, 0, 0, 255);
        pallet[0x3F] = Color::from_rgba(0, 0, 0, 255);

        Ppu {
            tbl_name: [[0; 1024]; 2],
            tbl_palette: [0; 32],
            tbl_pattern: [[0; 4096]; 2],
            pallete_screen: pallet,
            sprite_screen: Image::gen_image_color(256, 240, WHITE),
            sprite_name_table: [
                Image::gen_image_color(256, 240, WHITE),
                Image::gen_image_color(256, 240, WHITE),
            ],
            sprite_pattern_table: [
                Image::gen_image_color(128, 128, WHITE),
                Image::gen_image_color(128, 128, WHITE),
            ],
            frame_complete: false,
            scanline: 0,
            cycle: 0,
            status: Status(0),
            mask: Mask(0),
            control: PpuControl(0),
            address_latch: 0x00,
            data_buffer: 0x00,
            nmi: false,
            vram_addr: LoopyRegister(0),
            tram_addr: LoopyRegister(0),
            fine_x: 0x00,
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
        }
    }

    pub fn cpu_write(&mut self, cart: &mut Cartridge, addr: u16, data: u8) {
        match addr {
            0x0000 => {
                self.control.0 = data;
                self.tram_addr.set_nametable_x(self.control.nametable_x());
                self.tram_addr.set_nametable_y(self.control.nametable_y());
            } // Control
            0x0001 => {
                self.mask.0 = data;
            } // Mask
            0x0002 => {} // Status
            0x0003 => {} // OAM Address
            0x0004 => {} // OAM Data
            0x0005 => {
                if self.address_latch == 0 {
                    self.fine_x = data & 0x07;
                    self.tram_addr.set_coarse_x(data.wrapping_shr(3) as u16); // TODO: check position of the "as u16"
                    self.address_latch = 1;
                } else {
                    self.tram_addr.set_fine_y((data & 0x07) as u16);
                    self.tram_addr.set_coarse_y(data.wrapping_shr(3) as u16); // TODO: check position of the "as u16"
                    self.address_latch = 0;
                }
            } // Scroll
            0x0006 => {
                if self.address_latch == 0 {
                    self.tram_addr.0 =
                        ((data & 0x3F) as u16).wrapping_shl(8) | (self.tram_addr.0 & 0x00FF); //TODO: check position of the "as u16"
                    self.address_latch = 1;
                } else {
                    self.tram_addr.0 = (self.tram_addr.0 & 0xFF00) | (data as u16);
                    self.vram_addr = self.tram_addr;
                    self.address_latch = 0;
                }
            } // PPU Address
            0x0007 => {
                self.ppu_write(cart, self.vram_addr.0, data);
                self.vram_addr.0 = self
                    .vram_addr
                    .0
                    .wrapping_add(if self.control.increment_mode() { 32 } else { 1 });
            } // PPU Data
            _ => {}
        }
    }

    pub fn cpu_read(&mut self, cart: &mut Cartridge, addr: u16, read_only: bool) -> u8 {
        let mut data: u8 = 0x00;

        if read_only {
            match addr {
                0x0000 => {
                    data = self.control.0;
                } // Control
                0x0001 => {
                    data = self.mask.0;
                } // Mask
                0x0002 => {
                    data = self.status.0;
                } // Status
                0x0003 => {} // OAM Address
                0x0004 => {} // OAM Data
                0x0005 => {} // Scroll
                0x0006 => {} // PPU Address
                0x0007 => {} // PPU Data
                _ => {}
            }
        } else {
            match addr {
                0x0000 => {} // Control
                0x0001 => {} // Mask
                0x0002 => {
                    data = (self.status.0 & 0xE0) | (self.data_buffer & 0x1F);
                    self.status.set_vertical_blank(false);
                    self.address_latch = 0;
                } // Status
                0x0003 => {} // OAM Address
                0x0004 => {} // OAM Data
                0x0005 => {} // Scroll
                0x0006 => {} // PPU Address
                0x0007 => {
                    data = self.data_buffer;
                    self.data_buffer = self.ppu_read(cart, self.vram_addr.0, false);

                    if self.vram_addr.0 >= 0x3F00 {
                        data = self.data_buffer;
                    }

                    self.vram_addr.0 = self
                        .vram_addr
                        .0
                        .wrapping_add(if self.control.increment_mode() { 32 } else { 1 });
                } // PPU Data
                _ => {}
            }
        }

        data
    }

    pub fn ppu_write(&mut self, cart: &mut Cartridge, mut addr: u16, data: u8) {
        addr &= 0x3FFF;

        if cart.ppu_write(addr, data) {
        } else if addr <= 0x1FFF {
            self.tbl_pattern[((addr & 0x1000).wrapping_shr(12)) as usize]
                [(addr & 0x0FFF) as usize] = data;
        } else if addr >= 0x2000 && addr <= 0x3EFF {
            addr &= 0x0FFF;
            match cart.mirror {
                cartridge::Mirror::Vertical => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0400 && addr <= 0x07FF {
                        self.tbl_name[1][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF {
                        self.tbl_name[0][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF {
                        self.tbl_name[1][(addr & 0x03FF) as usize] = data;
                    }
                }
                cartridge::Mirror::Horizontal => {
                    if addr <= 0x03FF {
                        self.tbl_name[0][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0400 && addr <= 0x07FF {
                        self.tbl_name[0][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF {
                        self.tbl_name[1][(addr & 0x03FF) as usize] = data;
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF {
                        self.tbl_name[1][(addr & 0x03FF) as usize] = data
                    }
                }
                cartridge::Mirror::OneScreenLo => todo!(),
                cartridge::Mirror::OneScreenHi => todo!(),
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            addr &= 0x001F;
            // match addr {
            //     0x0010 => addr = 0x0000,
            //     0x0014 => addr = 0x0004,
            //     0x0018 => addr = 0x0008,
            //     0x001C => addr = 0x000C,
            //     _ => (),
            // }
            match addr {
                0x0010 | 0x0014 | 0x0018 | 0x001C => addr &= 0x000F,
                _ => (),
            }

            self.tbl_palette[addr as usize] = data;
        }
    }

    pub fn ppu_read(&self, cart: &mut Cartridge, mut addr: u16, _b_read_only: bool) -> u8 {
        let mut data: u8 = 0x00;
        addr &= 0x3FFF;

        if cart.ppu_read(addr, &mut data) {
        } else if addr <= 0x1FFF {
            data = self.tbl_pattern[((addr & 0x1000).wrapping_shr(12)) as usize]
                [(addr & 0x0FFF) as usize];
        } else if addr >= 0x2000 && addr <= 0x3EFF {
            addr &= 0x0FFF;
            match cart.mirror {
                cartridge::Mirror::Vertical => {
                    if addr <= 0x03FF {
                        data = self.tbl_name[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0400 && addr <= 0x07FF {
                        data = self.tbl_name[1][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF {
                        data = self.tbl_name[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF {
                        data = self.tbl_name[1][(addr & 0x03FF) as usize];
                    }
                }
                cartridge::Mirror::Horizontal => {
                    if addr <= 0x03FF {
                        data = self.tbl_name[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0400 && addr <= 0x07FF {
                        data = self.tbl_name[0][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0800 && addr <= 0x0BFF {
                        data = self.tbl_name[1][(addr & 0x03FF) as usize];
                    }
                    if addr >= 0x0C00 && addr <= 0x0FFF {
                        data = self.tbl_name[1][(addr & 0x03FF) as usize];
                    }
                }
                cartridge::Mirror::OneScreenLo => todo!(),
                cartridge::Mirror::OneScreenHi => todo!(),
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            addr &= 0x001F;
            // match addr {
            //     0x0010 => addr = 0x0000,
            //     0x0014 => addr = 0x0004,
            //     0x0018 => addr = 0x0008,
            //     0x001C => addr = 0x000C,
            //     _ => (),
            // }
            match addr {
                0x0010 | 0x0014 | 0x0018 | 0x001C => addr &= 0x000F,
                _ => (),
            }

            data =
                self.tbl_palette[addr as usize] & (if self.mask.grayscale() { 0x30 } else { 0x3F });
        }

        data
    }

    fn increment_scroll_x(&mut self) {
        if self.mask.render_background() || self.mask.render_sprites() {
            if self.vram_addr.coarse_x() == 31 {
                self.vram_addr.set_coarse_x(0);
                self.vram_addr
                    .set_nametable_x(!self.vram_addr.nametable_x());
            } else {
                self.vram_addr
                    .set_coarse_x(self.vram_addr.coarse_x().wrapping_add(1));
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.mask.render_background() || self.mask.render_sprites() {
            if self.vram_addr.fine_y() < 7 {
                self.vram_addr
                    .set_fine_y(self.vram_addr.fine_y().wrapping_add(1));
            } else {
                self.vram_addr.set_fine_y(0);

                if self.vram_addr.coarse_y() == 29 {
                    self.vram_addr.set_coarse_y(0);
                    self.vram_addr
                        .set_nametable_y(!self.vram_addr.nametable_y())
                } else if self.vram_addr.coarse_y() == 31 {
                    self.vram_addr.set_coarse_y(0);
                } else {
                    self.vram_addr
                        .set_coarse_y(self.vram_addr.coarse_y().wrapping_add(1));
                }
            }
        }
    }

    fn transfer_address_x(&mut self) {
        if self.mask.render_background() || self.mask.render_sprites() {
            self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
            self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
        }
    }

    fn transfer_address_y(&mut self) {
        if self.mask.render_background() || self.mask.render_sprites() {
            self.vram_addr.set_fine_y(self.tram_addr.fine_y());
            self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
            self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00)
            | if (self.bg_next_tile_attrib & 0b00000001) == 0b00000001 {
                0xFF
            } else {
                0x00
            };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | if (self.bg_next_tile_attrib & 0b00000010) == 0b00000010 {
                0xFF
            } else {
                0x00
            };
    }

    fn update_shifters(&mut self) {
        if self.mask.render_background() {
            self.bg_shifter_pattern_lo = self.bg_shifter_pattern_lo.wrapping_shl(1);
            self.bg_shifter_pattern_hi = self.bg_shifter_pattern_hi.wrapping_shl(1);

            self.bg_shifter_attrib_lo = self.bg_shifter_attrib_lo.wrapping_shl(1);
            self.bg_shifter_attrib_hi = self.bg_shifter_attrib_hi.wrapping_shl(1);
        }
    }

    pub fn clock(&mut self, cart: &mut Cartridge) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.set_vertical_blank(false);
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();

                        self.bg_next_tile_id =
                            self.ppu_read(cart, 0x2000 | (self.vram_addr.0 & 0x0FFF), false);
                    }
                    2 => {
                        self.bg_next_tile_attrib = self.ppu_read(
                            cart,
                            0x23C0
                                | (self.vram_addr.nametable_y() as u16).wrapping_shl(11)
                                | (self.vram_addr.nametable_x() as u16).wrapping_shl(10)
                                | self.vram_addr.coarse_y().wrapping_shr(2).wrapping_shl(3)
                                | self.vram_addr.coarse_x().wrapping_shr(2),
                            false,
                        );
                        if self.vram_addr.coarse_y() & 0x02 == 0x02 {
                            self.bg_next_tile_attrib = self.bg_next_tile_attrib.wrapping_shr(4);
                        }
                        if self.vram_addr.coarse_x() & 0x02 == 0x02 {
                            self.bg_next_tile_attrib = self.bg_next_tile_attrib.wrapping_shr(2);
                        }
                        self.bg_next_tile_attrib &= 0x03;
                    }
                    4 => {
                        self.bg_next_tile_lsb = self.ppu_read(
                            cart,
                            (self.control.pattern_background() as u16)
                                .wrapping_shl(12)
                                .wrapping_add((self.bg_next_tile_id as u16).wrapping_shl(4))
                                .wrapping_add(self.vram_addr.fine_y()),
                            false,
                        );
                    }
                    6 => {
                        self.bg_next_tile_msb = self.ppu_read(
                            cart,
                            (self.control.pattern_background() as u16)
                                .wrapping_shl(12)
                                .wrapping_add((self.bg_next_tile_id as u16).wrapping_shl(4))
                                .wrapping_add(self.vram_addr.fine_y())
                                .wrapping_add(8),
                            false,
                        );
                    }
                    7 => {
                        self.increment_scroll_x();
                    }
                    _ => {}
                }
            }

            if self.scanline == 256 {
                self.increment_scroll_y();
            }

            if self.scanline == 257 {
                self.load_background_shifters();
                self.transfer_address_x();
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id =
                    self.ppu_read(cart, 0x2000 | (self.vram_addr.0 & 0x0FFF), false);
            }

            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.transfer_address_y();
            }
        }

        if self.scanline == 240 {}

        if self.scanline >= 241 && self.scanline < 261 {
            if self.scanline == 241 && self.cycle == 1 {
                self.status.set_vertical_blank(true);

                self.nmi = self.control.enable_nmi();
            }
        }

        let mut bg_pixel: u8 = 0x00;
        let mut bg_pallete: u8 = 0x00;

        if self.mask.render_background() {
            let bit_mux: u16 = 0x8000u16.wrapping_shr(self.fine_x as u32);

            let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;
            bg_pixel = p1_pixel.wrapping_shl(1) | p0_pixel;

            let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_pallete = bg_pal1.wrapping_shl(1) | bg_pal0;
        }

        // let n = rand::gen_range(0, 63); //if (rand::rand() % 2) == 1 { 0x3F } else { 0x30 };
        if (self.cycle <= self.sprite_screen.width as i16)
            && (self.cycle >= 1)
            && (self.scanline >= 0)
            && (self.scanline < self.sprite_screen.height as i16)
        {
            // println!("cycle: {:?}", self.cycle);
            // println!("cycle: {:?}", self.cycle.wrapping_sub(1) as u32);
            // println!("scanline: {:?}", self.scanline);
            self.sprite_screen.set_pixel(
                (self.cycle - 1) as u32,
                self.scanline as u32,
                self.get_colour_from_pallet_ram(cart, &bg_pallete, &bg_pixel),
            );
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.address_latch = 0x00;
        self.data_buffer = 0x00;
        self.scanline = 0;
        self.cycle = 0;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status.0 = 0x00;
        self.mask.0 = 0x00;
        self.control.0 = 0x00;
        self.vram_addr.0 = 0x0000;
        self.tram_addr.0 = 0x0000;
    }

    pub fn get_colour_from_pallet_ram(
        &self,
        cart: &mut Cartridge,
        pallete: &u8,
        pixel: &u8,
    ) -> Color {
        self.pallete_screen[(self.ppu_read(
            cart,
            (0x3F00 as u16)
                .wrapping_add(((*pallete).wrapping_shl(2)) as u16)
                .wrapping_add(*pixel as u16),
            false,
        ) & 0x3F) as usize]
    }
}
