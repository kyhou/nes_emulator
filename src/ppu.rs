use bitfield::bitfield;
use macroquad::prelude::*;

use crate::{cartridge, Cartridge};

pub struct Ppu {
    pub tbl_name: [[u8; 1024]; 2],
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
    pub oam: [ObjectAttributeEntry; 64],
    oam_addr: u8,
    sprite_scanline: [ObjectAttributeEntry; 8],
    sprite_count: u8,
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,
    scanline_trigger: bool,
    odd_frame: bool,
}

bitfield! {
    pub struct Status(u8);
    impl Debug;
    u8;
    unused, _: 4, 0;
    sprite_overflow, set_sprite_overflow: 5;
    sprite_zero_hit, set_sprite_zero_hit: 6;
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectAttributeEntry {
    pub y: u8,
    pub id: u8,
    pub attribute: u8,
    pub x: u8,
}

impl ObjectAttributeEntry {
    fn new(y: u8, id: u8, attribute: u8, x: u8) -> Self {
        ObjectAttributeEntry {
            y,
            id,
            attribute,
            x,
        }
    }
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
        for tile_y in 0_u16..16 {
            for tile_x in 0_u16..16 {
                let offset: u16 = tile_y
                    .wrapping_mul(self.get_screen().width)
                    .wrapping_add(tile_x.wrapping_mul(16));

                for row in 0_u16..8 {
                    let mut tile_lsb: u8 = self.ppu_read(
                        cart,
                        (i as u16)
                            .wrapping_mul(0x1000)
                            .wrapping_add(offset)
                            .wrapping_add(row),
                        false,
                    );

                    let mut tile_msb: u8 = self.ppu_read(
                        cart,
                        (i as u16)
                            .wrapping_mul(0x1000)
                            .wrapping_add(offset)
                            .wrapping_add(row)
                            .wrapping_add(0x0008),
                        false,
                    );

                    for col in 0u16..8 {
                        let pixel: u8 = (tile_msb & 0x01).wrapping_shl(1) | (tile_lsb & 0x01);
                        tile_lsb = tile_lsb.wrapping_shr(1);
                        tile_msb = tile_msb.wrapping_shr(1);

                        self.sprite_pattern_table[i as usize].set_pixel(
                            tile_x.wrapping_mul(8).wrapping_add(7_u16.wrapping_sub(col)) as u32,
                            tile_y.wrapping_mul(8).wrapping_add(row) as u32,
                            self.get_colour_from_pallet_ram(cart, pallete.clone(), pixel.clone()),
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
            oam: [ObjectAttributeEntry::new(0, 0, 0, 0); 64],
            oam_addr: 0x00,
            sprite_scanline: [ObjectAttributeEntry::new(0, 0, 0, 0); 8],
            sprite_count: 0x00,
            sprite_shifter_pattern_lo: [0; 8],
            sprite_shifter_pattern_hi: [0; 8],
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
            scanline_trigger: false,
            odd_frame: false,
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
            0x0003 => {
                self.oam_addr = data;
            } // OAM Address
            0x0004 => match self.oam_addr % 4 {
                0 => {
                    self.oam[(self.oam_addr / 4) as usize].y = data;
                }
                1 => {
                    self.oam[(self.oam_addr / 4) as usize].id = data;
                }
                2 => {
                    self.oam[(self.oam_addr / 4) as usize].attribute = data;
                }
                3 => {
                    self.oam[(self.oam_addr / 4) as usize].x = data;
                }
                _ => (),
            }, // OAM Data
            0x0005 => {
                if self.address_latch == 0 {
                    self.fine_x = data & 0x07;
                    self.tram_addr.set_coarse_x(data.wrapping_shr(3) as u16);
                    self.address_latch = 1;
                } else {
                    self.tram_addr.set_fine_y((data & 0x07) as u16);
                    self.tram_addr.set_coarse_y(data.wrapping_shr(3) as u16);
                    self.address_latch = 0;
                }
            } // Scroll
            0x0006 => {
                if self.address_latch == 0 {
                    self.tram_addr.0 =
                        ((data & 0x3F) as u16).wrapping_shl(8) | (self.tram_addr.0 & 0x00FF);
                    self.address_latch = 1;
                } else {
                    self.tram_addr.0 = (self.tram_addr.0 & 0xFF00) | (data as u16);
                    self.vram_addr = self.tram_addr.clone();
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
                0x0004 => match self.oam_addr % 4 {
                    0 => {
                        data = self.oam[(self.oam_addr / 4) as usize].y;
                    }
                    1 => {
                        data = self.oam[(self.oam_addr / 4) as usize].id;
                    }
                    2 => {
                        data = self.oam[(self.oam_addr / 4) as usize].attribute;
                    }
                    3 => {
                        data = self.oam[(self.oam_addr / 4) as usize].x;
                    }
                    _ => (),
                }, // OAM Data
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
            match cart.mirror() {
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
                cartridge::Mirror::Hardware => todo!(),
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            addr &= 0x001F;

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
            match cart.mirror() {
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
                cartridge::Mirror::Hardware => todo!(),
            }
        } else if addr >= 0x3F00 && addr <= 0x3FFF {
            addr &= 0x001F;

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
            | if (self.bg_next_tile_attrib & 0b01) > 0 {
                0xFF
            } else {
                0x00
            };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | if (self.bg_next_tile_attrib & 0b10) > 0 {
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

        if self.mask.render_sprites() && self.cycle >= 1 && self.cycle < 258 {
            for i in 0_usize..self.sprite_count as usize {
                if self.sprite_scanline[i].x > 0 {
                    self.sprite_scanline[i].x = self.sprite_scanline[i].x.wrapping_sub(1);
                } else {
                    self.sprite_shifter_pattern_lo[i] =
                        self.sprite_shifter_pattern_lo[i].wrapping_shl(1);
                    self.sprite_shifter_pattern_hi[i] =
                        self.sprite_shifter_pattern_hi[i].wrapping_shl(1);
                }
            }
        }
    }

    pub fn clock(&mut self, cart: &mut Cartridge) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0
                && self.cycle == 0
                && self.odd_frame
            && (self.mask.render_background() || self.mask.render_sprites())
        {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.set_vertical_blank(false);
                self.status.set_sprite_overflow(false);
                self.status.set_sprite_zero_hit(false);

                for i in 0_usize..8 {
                    self.sprite_shifter_pattern_lo[i] = 0;
                    self.sprite_shifter_pattern_hi[i] = 0;
                }
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

                        if self.vram_addr.coarse_y() & 0x02 > 0 {
                            self.bg_next_tile_attrib = self.bg_next_tile_attrib.wrapping_shr(4);
                        }

                        if self.vram_addr.coarse_x() & 0x02 > 0 {
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

            if self.cycle == 256 {
                self.increment_scroll_y();
            }

            if self.cycle == 257 {
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

            // Foreground Rendering

            if self.cycle == 257 && self.scanline >= 0 {
                self.sprite_scanline = [ObjectAttributeEntry::new(0xFF, 0xFF, 0xFF, 0xFF); 8];
                self.sprite_count = 0;

                for i in 0_usize..8 {
                    self.sprite_shifter_pattern_lo[i] = 0;
                    self.sprite_shifter_pattern_hi[i] = 0;
                }

                let mut oam_entry: u8 = 0;

                self.sprite_zero_hit_possible = false;

                while oam_entry < 64 && self.sprite_count < 9 {
                    let diff: i16 = self
                        .scanline
                        .wrapping_sub(self.oam[oam_entry as usize].y as i16);

                    let sprite_size = if self.control.sprite_size() { 16 } else { 8 };
                    if diff >= 0 && diff < sprite_size && self.sprite_count < 8 {
                        if self.sprite_count < 8 {
                            if oam_entry == 0 {
                                self.sprite_zero_hit_possible = true;
                            }

                            self.sprite_scanline[self.sprite_count as usize] =
                                self.oam[oam_entry as usize].clone();
                        }
                            self.sprite_count = self.sprite_count.wrapping_add(1);
                    }

                    oam_entry = oam_entry.wrapping_add(1);
                }

                self.status.set_sprite_overflow(self.sprite_count >= 8);
            }

            if self.cycle == 340 {
                for i in 0_usize..self.sprite_count as usize {
                    let mut sprite_pattern_bits_lo: u8;
                    let mut sprite_pattern_bits_hi: u8;
                    let sprite_pattern_addr_lo: u16;
                    let sprite_pattern_addr_hi: u16;

                    if !self.control.sprite_size() {
                        // 8x8
                        if self.sprite_scanline[i].attribute & 0x80 == 0 {
                            // sprite not flipped
                            sprite_pattern_addr_lo = (self.control.pattern_sprite() as u16)
                                .wrapping_shl(12)
                                | (self.sprite_scanline[i].id as u16).wrapping_shl(4)
                                | (self.scanline as u16)
                                    .wrapping_sub(self.sprite_scanline[i].y as u16);
                        } else {
                            //sprite flipped
                            sprite_pattern_addr_lo = (self.control.pattern_sprite() as u16)
                                .wrapping_shl(12)
                                | (self.sprite_scanline[i].id as u16).wrapping_shl(4)
                                | 7_u16.wrapping_sub(
                                    (self.scanline as u16)
                                        .wrapping_sub(self.sprite_scanline[i].y as u16),
                                );
                        }
                    } else {
                        // 8x16
                        if self.sprite_scanline[i].attribute & 0x80 == 0 {
                            // sprite not flipped
                            if self.scanline.wrapping_sub(self.sprite_scanline[i].y as i16) < 8 {
                                sprite_pattern_addr_lo = (self.sprite_scanline[i].id as u16 & 0x01)
                                    .wrapping_shl(12)
                                    | (self.sprite_scanline[i].id as u16 & 0xFE).wrapping_shl(4)
                                    | (self.scanline as u16)
                                        .wrapping_sub(self.sprite_scanline[i].y as u16)
                                        & 0x07;
                            } else {
                                sprite_pattern_addr_lo = (self.sprite_scanline[i].id as u16 & 0x01)
                                    .wrapping_shl(12)
                                    | (self.sprite_scanline[i].id as u16 & 0xFE)
                                        .wrapping_add(1)
                                        .wrapping_shl(4)
                                    | (self.scanline as u16)
                                        .wrapping_sub(self.sprite_scanline[i].y as u16)
                                        & 0x07;
                            }
                        } else {
                            //sprite flipped
                            if self.scanline.wrapping_sub(self.sprite_scanline[i].y as i16) < 8 {
                                sprite_pattern_addr_lo = (self.sprite_scanline[i].id as u16 & 0x01)
                                    .wrapping_shl(12)
                                    | (self.sprite_scanline[i].id as u16 & 0xFE)
                                        .wrapping_add(1)
                                        .wrapping_shl(4)
                                    | 7_u16.wrapping_sub(
                                        (self.scanline as u16)
                                            .wrapping_sub(self.sprite_scanline[i].y as u16)
                                            & 0x07,
                                    );
                            } else {
                                sprite_pattern_addr_lo = (self.sprite_scanline[i].id as u16 & 0x01)
                                    .wrapping_shl(12)
                                    | (self.sprite_scanline[i].id as u16 & 0xFE).wrapping_shl(4)
                                    | 7_u16.wrapping_sub(
                                        (self.scanline as u16)
                                            .wrapping_sub(self.sprite_scanline[i].y as u16)
                                            & 0x07,
                                    );
                            }
                        }
                    }

                    sprite_pattern_addr_hi = sprite_pattern_addr_lo.wrapping_add(8);
                    sprite_pattern_bits_lo = self.ppu_read(cart, sprite_pattern_addr_lo, false);
                    sprite_pattern_bits_hi = self.ppu_read(cart, sprite_pattern_addr_hi, false);

                    if (self.sprite_scanline[i].attribute & 0x40) > 0 {
                        let flip_byte = |mut b: u8| {
                            b = (b & 0xF0).wrapping_shr(4) | (b & 0x0F).wrapping_shl(4);
                            b = (b & 0xCC).wrapping_shr(2) | (b & 0x33).wrapping_shl(2);
                            b = (b & 0xAA).wrapping_shr(1) | (b & 0x55).wrapping_shl(1);
                            b
                        };

                        sprite_pattern_bits_lo = flip_byte(sprite_pattern_bits_lo);
                        sprite_pattern_bits_hi = flip_byte(sprite_pattern_bits_hi);
                    }

                    self.sprite_shifter_pattern_lo[i] = sprite_pattern_bits_lo;
                    self.sprite_shifter_pattern_hi[i] = sprite_pattern_bits_hi;
                }
            }
        }

        if self.scanline == 240 {}

        if self.scanline >= 241 && self.scanline < 261 {
            if self.scanline == 241 && self.cycle == 1 {
                self.status.set_vertical_blank(true);

                if self.control.enable_nmi() {
                    self.nmi = true;
                }
            }
        }

        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        if self.mask.render_background()
            && (self.mask.render_background_left() || (self.cycle >= 9))
        {
                let bit_mux: u16 = 0x8000_u16.wrapping_shr(self.fine_x as u32);

                let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
                let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;
                bg_pixel = p1_pixel.wrapping_shl(1) | p0_pixel;

                let bg_pal0: u8 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
                let bg_pal1: u8 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
                bg_palette = bg_pal1.wrapping_shl(1) | bg_pal0;
        }

        let mut fg_pixel: u8 = 0x00;
        let mut fg_palette: u8 = 0x00;
        let mut fg_priority: bool = false;

        if self.mask.render_sprites() && (self.mask.render_sprites_left() || (self.cycle >= 9)) {
            self.sprite_zero_being_rendered = false;

            for i in 0..self.sprite_count as usize {
                if self.sprite_scanline[i].x == 0 {
                    let fg_pixel_lo: u8 = ((self.sprite_shifter_pattern_lo[i] & 0x80) > 0) as u8;
                    let fg_pixel_hi: u8 = ((self.sprite_shifter_pattern_hi[i] & 0x80) > 0) as u8;
                    fg_pixel = fg_pixel_hi.wrapping_shl(1) | fg_pixel_lo;

                    fg_palette = (self.sprite_scanline[i].attribute & 0x03).wrapping_add(0x04);
                    fg_priority = (self.sprite_scanline[i].attribute & 0x20) == 0;

                    if fg_pixel != 0 {
                        if i == 0 {
                            self.sprite_zero_being_rendered = true;
                        }

                        break;
                    }
                }
            }
        }

        let mut pixel: u8 = 0x00;
        let mut palette: u8 = 0x00;

        if bg_pixel == 0 && fg_pixel == 0 {
            pixel = 0x00;
            palette = 0x00;
        } else if bg_pixel == 0 && fg_pixel > 0 {
            pixel = fg_pixel;
            palette = fg_palette;
        } else if bg_pixel > 0 && fg_pixel == 0 {
            pixel = bg_pixel;
            palette = bg_palette;
        } else if bg_pixel > 0 && fg_pixel > 0 {
            if fg_priority {
                pixel = fg_pixel;
                palette = fg_palette;
            } else {
                pixel = bg_pixel;
                palette = bg_palette;
            }

            if self.sprite_zero_being_rendered && self.sprite_zero_hit_possible {
                if self.mask.render_background() && self.mask.render_sprites() {
                    if !(self.mask.render_background_left() | self.mask.render_sprites_left()) {
                        if self.cycle >= 9 && self.cycle < 258 {
                            self.status.set_sprite_zero_hit(true);
                        }
                    } else {
                        if self.cycle >= 1 && self.cycle < 258 {
                            self.status.set_sprite_zero_hit(true);
                        }
                    }
                }
            }
        }

        if (self.cycle <= self.sprite_screen.width as i16)
            && (self.cycle >= 1)
            && (self.scanline >= 0)
            && (self.scanline < self.sprite_screen.height as i16)
        {
            self.sprite_screen.set_pixel(
                (self.cycle - 1) as u32,
                self.scanline as u32,
                self.get_colour_from_pallet_ram(cart, palette.clone(), pixel.clone()),
            );
        }

        self.cycle += 1;

        if self.mask.render_background() || self.mask.render_sprites() {
            if self.cycle == 260 && self.scanline < 240 {
                cart.get_mapper().borrow_mut().scanline();
            }
        }

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;
                self.frame_complete = true;
                self.odd_frame = !self.odd_frame;
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
        self.scanline_trigger = false;
        self.odd_frame = false;
    }

    pub fn get_colour_from_pallet_ram(
        &self,
        cart: &mut Cartridge,
        pallete: u8,
        pixel: u8,
    ) -> Color {
        self.pallete_screen[(self.ppu_read(
            cart,
            0x3F00_u16
                .wrapping_add(((pallete).wrapping_shl(2)) as u16)
                .wrapping_add(pixel as u16),
            false,
        ) & 0x3F) as usize]
    }
}
