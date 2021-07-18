use std::{
    fmt,
    fs::File,
    io::{Read, Write},
};
use crate::common::{ToResult, WriteData, Address};
use validated::Validated::{self, Good, Fail};
use anyhow;
use thiserror::Error;
use itertools::Itertools;

#[derive(Error, Debug)]
struct ByteWriteError {
    byte: u8,
    address: Address,
}

impl fmt::Display for ByteWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error writing byte {} at address {}", self.byte, self.address)
    }
}

#[derive(Error, Debug)]
struct WriteAddressesError {
    byte_write_errors: Vec<ByteWriteError>
}

impl fmt::Display for WriteAddressesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined: String = self.byte_write_errors
            .into_iter()
            .map(|err| err.to_string())
            .join("\n");
        write!(f, "{}", joined)
    }
}

impl ToResult<(), WriteAddressesError> for Validated<(), ByteWriteError> {
    fn to_result(self) -> Result<(), WriteAddressesError> {
        match self {
            Good(_) => Ok(()),
            Fail(errs) => Err(WriteAddressesError {
                byte_write_errors: errs.into()
            })
        }
    }
}

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

    fn write_byte(&self, buffer: &mut [u8], byte: u8, address: Address) -> Validated<(), ByteWriteError> {
        match buffer.get_mut(address) {
            Some(elem) => Good(*elem = byte),
            None => Validated::fail(ByteWriteError { byte, address }),
        }
    }

    fn write_bytes(&self, buffer: &mut [u8], bytes: &[u8], address: Address) -> Validated<(), ByteWriteError> {
        bytes
            .iter()
            .enumerate()
            .map(|(idx, byte)| self.write_byte(buffer, *byte, address + idx))
            .collect()
    }

    fn write_addresses(&self, buffer: &mut [u8], bytes: &[u8], addresses: &[Address]) -> Validated<(), ByteWriteError> {
        addresses
            .into_iter()
            .map(|address| self.write_bytes(buffer, bytes, *address))
            .collect()
    }
}

impl RomWriter for Rom {
    fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        self.rom_file.read_to_end(&mut buffer)?;
        data.into_iter().map(|wd| {
            self.write_addresses(&mut buffer, &wd.bytes, &wd.target_addresses)
        }).collect::<Validated<(), ByteWriteError>>().to_result()?;
        self.rom_file.write_all(&buffer)?;
        Ok(())
    }
}
