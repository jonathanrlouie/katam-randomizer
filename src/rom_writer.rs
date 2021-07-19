use crate::common::{Address, IntoResult, WriteData};
use itertools::Itertools;
use std::{
    fmt,
    fs::File,
    io::{Read, Write},
};
use thiserror::Error;
use validated::Validated::{self, Fail, Good};

#[derive(Error, Debug)]
#[error("Error writing byte {byte:#04x} at address {address}")]
struct ByteWriteError {
    byte: u8,
    address: Address,
}

#[derive(Error, Debug)]
struct WriteAddressesError {
    byte_write_errors: Vec<ByteWriteError>,
}

impl fmt::Display for WriteAddressesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined: String = self
            .byte_write_errors
            .iter()
            .map(|err| err.to_string())
            .join("\n");
        write!(f, "{}", joined)
    }
}

impl IntoResult<(), WriteAddressesError> for Validated<(), ByteWriteError> {
    fn into_result(self) -> Result<(), WriteAddressesError> {
        match self {
            Good(_) => Ok(()),
            Fail(errs) => Err(WriteAddressesError {
                byte_write_errors: errs.into(),
            }),
        }
    }
}

pub trait RomWriter {
    fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()>;
}

pub struct Rom {
    rom_file: File,
}

impl Rom {
    pub fn new(rom_file: File) -> Self {
        Self { rom_file }
    }
}

impl RomWriter for Rom {
    fn write_data(&mut self, data: &[WriteData]) -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        self.rom_file.read_to_end(&mut buffer)?;
        data.iter()
            .map(|wd| write_addresses(&mut buffer, &wd.bytes, &wd.target_addresses))
            .collect::<Validated<(), ByteWriteError>>()
            .into_result()?;
        self.rom_file.write_all(&buffer)?;
        Ok(())
    }
}

fn write_byte(buffer: &mut [u8], byte: u8, address: Address) -> Validated<(), ByteWriteError> {
    match buffer.get_mut(address) {
        Some(elem) => {
            *elem = byte;
            Good(())
        }
        None => Validated::fail(ByteWriteError { byte, address }),
    }
}

fn write_bytes(buffer: &mut [u8], bytes: &[u8], address: Address) -> Validated<(), ByteWriteError> {
    bytes
        .iter()
        .enumerate()
        .map(|(idx, byte)| write_byte(buffer, *byte, address + idx))
        .collect()
}

fn write_addresses(
    buffer: &mut [u8],
    bytes: &[u8],
    addresses: &[Address],
) -> Validated<(), ByteWriteError> {
    addresses
        .iter()
        .map(|address| write_bytes(buffer, bytes, *address))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0];
        let result = write_addresses(&mut buffer, &bytes, &addresses)
            .into_result()
            .map_err(|_| {
                "Error occurred when writing bytes to addresses, but no error was expected."
                    .to_string()
            })?;
        assert_eq!([0x03, 0x87, 0x22, 0xAD], buffer);
        Ok(())
    }

    #[test]
    fn test_write_out_of_bounds() {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [4];
        let result = write_addresses(&mut buffer, &bytes, &addresses).into_result();
        match result {
            Ok(_) => panic!("Writing bytes succeeded, but should not have."),
            Err(err) => {
                assert_eq!(err.byte_write_errors.len(), 2);
                assert_eq!(err.byte_write_errors[0].to_string(), "Error writing byte 0x03 at address 4");
                assert_eq!(err.byte_write_errors[1].to_string(), "Error writing byte 0x87 at address 5");
            }
        }
    }

    #[test]
    fn test_write_partially_out_of_bounds() {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [3];
        let result = write_addresses(&mut buffer, &bytes, &addresses).into_result();
        match result {
            Ok(_) => panic!("Writing bytes succeeded, but should not have."),
            Err(err) => {
                assert_eq!(err.byte_write_errors.len(), 1);
                assert_eq!(err.byte_write_errors[0].to_string(), "Error writing byte 0x87 at address 4");
            }
        }
    }

    #[test]
    fn test_write_multiple_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0, 2];
        let result = write_addresses(&mut buffer, &bytes, &addresses)
            .into_result()
            .map_err(|_| {
                "Error occurred when writing bytes to addresses, but no error was expected."
                    .to_string()
            })?;
        assert_eq!([0x03, 0x87, 0x03, 0x87], buffer);
        Ok(())
    }

    #[test]
    fn test_write_overlapping_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0, 1];
        let result = write_addresses(&mut buffer, &bytes, &addresses)
            .into_result()
            .map_err(|_| {
                "Error occurred when writing bytes to addresses, but no error was expected."
                    .to_string()
            })?;
        assert_eq!([0x03, 0x03, 0x87, 0xAD], buffer);
        Ok(())
    }
}
