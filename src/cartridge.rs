use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    rc::Rc,
};

use crate::{mapper::RW, mapper_000::Mapper000};

pub struct Cartridge {
    prg_memory: Vec<u8>,
    chr_memory: Vec<u8>,
    mapper_id: u8,
    pub prg_banks: u8,
    pub chr_banks: u8,
    pub image_valid: bool,
    pub mirror: Mirror,
    mapper: Rc<dyn RW>,
}

#[repr(C)]
struct INesHeader {
    name: [u8; 4],
    prg_rom_chunks: u8,
    chr_rom_chunks: u8,
    mapper1: u8,
    mapper2: u8,
    prg_ram_size: u8,
    tv_system1: u8,
    tv_system2: u8,
    unused: [u8; 5],
}

pub enum Mirror {
    Vertical,
    Horizontal,
    OneScreenLo,
    OneScreenHi,
}

impl Cartridge {
    pub fn new(file_name: &str) -> Self {
        let file_path = Path::new(file_name);
        let mut file = match File::open(&file_path) {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file: {}", e),
        };

        let mut header = INesHeader {
            name: [0; 4],
            prg_rom_chunks: 0,
            chr_rom_chunks: 0,
            mapper1: 0,
            mapper2: 0,
            prg_ram_size: 0,
            tv_system1: 0,
            tv_system2: 0,
            unused: [0; 5],
        };

        let header_size = std::mem::size_of::<INesHeader>();
        unsafe {
            let header_slice =
                std::slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            file.read_exact(header_slice).unwrap();
        }

        if (header.mapper1 & 0x04) > 0 {
            file.seek(SeekFrom::Current(512)).unwrap();
        }

        let mapper_id = header.mapper2.wrapping_shr(4).wrapping_shl(4) | header.mapper1.wrapping_shr(4);
        let mirror = if (header.mapper1 & 0x01) > 0 {
            Mirror::Vertical
        } else {
            Mirror::Horizontal
        };

        let n_file_type = 1;

        let mut prg_memory: Vec<u8> = Vec::new();
        let mut chr_memory: Vec<u8> = Vec::new();
        let mut prg_banks: u8 = 0;
        let mut chr_banks: u8 = 0;

        match n_file_type {
            0 => {}
            1 => {
                prg_banks = header.prg_rom_chunks;
                prg_memory.resize((prg_banks as usize) * (16 * 1024), 0);
                if let Err(error) = file.read(&mut prg_memory) {
                    println!("{:?}", error);
                }

                chr_banks = header.chr_rom_chunks;
                chr_memory.resize((chr_banks as usize).max(1) * (8 * 1024), 0);
                if let Err(error) = file.read(&mut chr_memory) {
                    println!("{:?}", error);
                }
            }
            2 => {}
            _ => {}
        }

        let mut mapper: Rc<dyn RW> = Rc::new(Mapper000::new(0, 0));

        match mapper_id {
            0 => {
                mapper = Rc::new(Mapper000::new(prg_banks, chr_banks));
            }
            _ => {}
        }

        Cartridge {
            prg_memory,
            chr_memory,
            mapper_id,
            prg_banks,
            chr_banks,
            image_valid: true,
            mirror,
            mapper,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.cpu_map_write(addr, &mut mapped_addr, &data) {
            self.prg_memory[mapped_addr as usize] = data;
            true
        } else {
            false
        }
    }

    pub fn cpu_read(&self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.cpu_map_read(addr, &mut mapped_addr) {
            *data = self.prg_memory[mapped_addr as usize];
            true
        } else {
            false
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.ppu_map_write(self, addr, &mut mapped_addr) {
            self.chr_memory[mapped_addr as usize] = data;
            true
        } else {
            false
        }
    }

    pub fn ppu_read(&self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.ppu_map_read(addr, &mut mapped_addr) {
            *data = self.chr_memory[mapped_addr as usize];
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.mapper.reset();
    }
}
