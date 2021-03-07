use memory::Memory;
use std::fs::File;
use std::io::prelude::*;

mod cartridge_metadata;
use cartridge_metadata::CartridgeMetadata;
pub use cartridge_metadata::Mirroring;

mod mapper;
use mapper::*;
use mapper0::Mapper0;

pub struct Cartridge {
    meta: Option<CartridgeMetadata>,
    mapper: Option<Box<dyn Mapper>>,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            meta: None,
            mapper: None,
        }
    }

    pub fn from_file(file: File) -> Result<Self, &'static str> {
        let mut bytes = file.bytes();
        let header = match bytes.by_ref().take(0x10).collect() {
            Ok(header) => header,
            Err(_) => return Err("hit EOF while reading file header"),
        };
        let meta = CartridgeMetadata::from_header(header)?;

        let prg_size = 0x4000 * (meta.n_prg_banks as usize);
        let prg_data = match bytes.by_ref().take(prg_size).collect() {
            Ok(data) => data,
            Err(_) => return Err("hit EOF while reading program ROM"),
        };
        let chr_size = 0x2000 * (meta.n_chr_banks as usize);
        let chr_data = match bytes.by_ref().take(chr_size).collect() {
            Ok(data) => data,
            Err(_) => return Err("hit EOF while reading character ROM"),
        };

        let mapper = Box::from(match meta.mapper_num {
            0 => Mapper0::new(meta.n_prg_banks, meta.n_chr_banks, prg_data, chr_data),
            _ => return Err("unsupported mapper"),
        });

        Ok(Self {
            meta: Some(meta),
            mapper: Some(mapper),
        })
    }

    pub fn get_nametable_mirroring(&self) -> Mirroring {
        let default = if let Some(some_meta) = &self.meta {
            some_meta.hardwired_mirroring
        } else {
            Mirroring::Horizontal
        };
        if let Some(some_mapper) = &self.mapper {
            some_mapper.get_nametable_mirroring().unwrap_or(default)
        } else {
            default
        }
    }
}

impl Memory for Cartridge {
    fn read(&mut self, addr: u16) -> u8 {
        if let Some(some_mapper) = &mut self.mapper {
            some_mapper.read(addr)
        } else {
            0x00
        }
    }

    fn peek(&self, addr: u16) -> u8 {
        if let Some(some_mapper) = &self.mapper {
            some_mapper.peek(addr)
        } else {
            0x00
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if let Some(some_mapper) = &mut self.mapper {
            some_mapper.write(addr, data);
        }
    }
}
