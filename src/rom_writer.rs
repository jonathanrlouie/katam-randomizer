use std::fs::File;
use crate::{
    common::{DoorDataMaps, StringID},
    error::Error,
};
use anyhow;

pub trait RomWriter {
    fn write_addresses(&mut self, bytes: &[u8], addresses: &[usize]) -> anyhow::Result<()>;
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
        }.collect::<Validated<(), error::Error>>()
    }

}

impl RomWriter for Rom {
    fn write_addresses(&mut self, bytes: &[u8], addresses: &[usize]) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        self.rom_file.read_to_end(&mut buffer)?;
        addresses.into_iter().map(|address| self.write_bytes(&mut buffer, bytes, *address)).collect().to_result()?;
        self.rom_file.write_all(&buffer)?;
        Ok(())
    }
}
