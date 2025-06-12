use std::{
    cell::RefCell, fs::File, io::{Read, Seek, SeekFrom}, path::Path, rc::Rc
};

use crate::{mapper::RW, mapper_000::Mapper000, mapper_004::Mapper004};

pub struct Cartridge {
    prg_memory: Vec<u8>,
    chr_memory: Vec<u8>,
    pub prg_banks: u8,
    pub chr_banks: u8,
    pub image_valid: bool,
    pub hw_mirror: Mirror,
    mapper: Rc<RefCell<dyn RW>>,
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

#[derive(Clone, PartialEq)]
pub enum Mirror {
    Hardware,
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
        let hw_mirror = if (header.mapper1 & 0x01) > 0 {
            Mirror::Vertical
        } else {
            Mirror::Horizontal
        };

        let mut file_type = 1;

        let mut prg_memory: Vec<u8> = Vec::new();
        let mut chr_memory: Vec<u8> = Vec::new();
        let mut prg_banks: u8 = 0;
        let mut chr_banks: u8 = 0;

        if (header.mapper2 & 0x0C) == 0x08 {
            file_type = 2;
        }

        match file_type {
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
            2 => {
                prg_banks = ((header.prg_ram_size & 0x07).wrapping_shl(8) | header.prg_rom_chunks) as u8;
                prg_memory.resize((prg_banks as usize) * (16 * 1024), 0);
                if let Err(error) = file.read(&mut prg_memory) {
                    println!("{:?}", error);
                }

                chr_banks = ((header.prg_ram_size & 0x38).wrapping_shr(3).wrapping_shl(8) | header.chr_rom_chunks) as u8;
                chr_memory.resize((chr_banks as usize).max(1) * (8 * 1024), 0);
                if let Err(error) = file.read(&mut chr_memory) {
                    println!("{:?}", error);
                }
            }
            _ => {}
        }

        let mut mapper: Rc<RefCell<dyn RW>> = Rc::new(RefCell::new(Mapper000::new(0, 0)));

        match mapper_id {
            0 => {
                mapper = Rc::new(RefCell::new(Mapper000::new(prg_banks, chr_banks)));
            }
            4 => {
                mapper = Rc::new(RefCell::new(Mapper004::new(prg_banks, chr_banks)));
            }
            _ => {
                println!("Mapper {} not yet implemented", mapper_id);
            }
        }

        Cartridge {
            prg_memory,
            chr_memory,
            prg_banks,
            chr_banks,
            image_valid: true,
            hw_mirror,
            mapper,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.borrow_mut().cpu_map_write(addr, &mut mapped_addr, &data) {
            if mapped_addr == 0xFFFFFFFF{
                return true;
            } else {
            self.prg_memory[mapped_addr as usize] = data;
            }

            true
        } else {
            false
        }
    }

    pub fn cpu_read(&self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.borrow().cpu_map_read(addr, &mut mapped_addr, data) {
            if mapped_addr == 0xFFFFFFFF{
                return true;
            } else {
            *data = self.prg_memory[mapped_addr as usize];
            }
            
            true
        } else {
            false
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.borrow().ppu_map_write(self, addr, &mut mapped_addr) {
            self.chr_memory[mapped_addr as usize] = data;
            true
        } else {
            false
        }
    }

    pub fn ppu_read(&self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr: u32 = 0;
        if self.mapper.borrow().ppu_map_read(addr, &mut mapped_addr) {
            if self.chr_memory.len() <= (mapped_addr as usize) {
                return false;
            }
            *data = self.chr_memory[mapped_addr as usize];
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        if let Ok(mut mapper) = self.mapper.try_borrow_mut() { mapper.reset() }
    }

    pub fn mirror(&self) -> Mirror {
        let mirror: Mirror = self.mapper.borrow().mirror();

        if mirror == Mirror::Hardware {
            self.hw_mirror.clone()
        } else {
            mirror
        }
    }

    pub fn get_mapper(&self) -> Rc<RefCell<dyn RW>> {
        self.mapper.clone()
    }
}
