use std::fs::File;
use crate::{
    common::WriteData,
    error::Error,
};
use anyhow;

pub trait RomWriter {
    fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()>;
}

pub struct Rom {
    rom_file: File
}

impl Rom {
    pub fn new(rom_file: File) -> Self {
        Self { rom_file }
    }

    fn write_byte(&self, buffer: &mut [u8], byte: u8, address: usize) -> Validated<(), error::Error> {
        match *buffer.get_mut(address) {
            Some(elem) => Good(*elem = byte),
            None => Validated::fail(//TODO: put error here),
        }
    }

    fn write_bytes(&self, buffer: &mut [u8], bytes: &[u8], address: usize) -> Validated<(), error::Error> {
        bytes.iter().enumerate().map(|(index, byte)| {
            self.write_byte(buffer, *byte, address + index)
        }).collect::<Validated<(), error::Error>>()
    }

    fn write_addresses(buffer: &mut [u8], bytes: &[u8], addresses: &[usize]) -> Validated<(), error::Error> {
        addresses
            .into_iter()
            .map(|address| self.write_bytes(buffer, bytes, *address))
            .collect();
    }
}

impl RomWriter for Rom {
    fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        self.rom_file.read_to_end(&mut buffer)?;
        data.into_iter().map(|wd| {
            self.write_addresses(&mut buffer, &wd.bytes, &wd.target_addresses)
        }).collect().to_result()?;
        self.rom_file.write_all(&buffer)?;
        Ok(())
    }
}
