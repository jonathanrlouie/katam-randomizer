use crate::{
    randomizer::{self, Graph, KatamRandoError, RomWriter},
    game_data::{Address, Destination, StringID, RomDataMaps},
};
use itertools::Itertools;
use std::{collections::HashMap, fmt, fs::File};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error writing byte {byte:#04x} at address {address}")]
struct ByteWriteError {
    byte: u8,
    address: Address,
}

#[derive(Error, Debug)]
struct WriteAddressesError(Vec<ByteWriteError>);

impl fmt::Display for WriteAddressesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined: String = self.0
            .iter()
            .map(|err| err.to_string())
            .join("\n");
        write!(f, "{}", joined)
    }
}

pub struct Rom<'a> {
    rom_file: &'a mut File,
    rom_data_maps: RomDataMaps
}

impl<'a> Rom<'a> {
    pub fn new(rom_file: &'a mut File, rom_data_maps: RomDataMaps) -> Self {
        Self { rom_file, rom_data_maps }
    }
}

impl<'a> RomWriter for Rom<'a> {
    fn write_data<N, E>(&mut self, graph: impl Graph<N, E>) -> randomizer::Result<()> {
        let mut buffer = Vec::new();
        self.rom_file.read_to_end(&mut buffer)?;

        for (start_node_id, end_node_id) in graph.get_edges() {
            println!("edge: {}, {}", start_node_id, end_node_id);

            let addresses_to_replace = self.rom_data_maps.start_map.get(&start_node_id)
                .ok_or_else(|| Error::MissingDoorDataNode(start_node_id))?;
            let dest = self.rom_data_maps.end_map.get(&end_node_id)
                .ok_or_else(|| Error::MissingDoorDataNode(end_node_id))?;
            write_addresses(&mut buffer, dest, addresses_to_replace);
        }

        self.rom_file.write_all(&buffer)?;
        Ok(())
    }
}

fn write_byte(buffer: &mut [u8], byte: u8, address: Address, errors: &mut Vec<ByteWriteError>) {
    match buffer.get_mut(address) {
        Some(elem) => *elem = byte,
        None => errors.push(ByteWriteError { byte, address })
    }
}

fn write_bytes(buffer: &mut [u8], bytes: &[u8], address: Address, errors: &mut Vec<ByteWriteError>) {
    bytes.iter()
        .enumerate()
        .foreach(|(idx, byte)| write_byte(buffer, *byte, address + idx, errors));
}

fn write_addresses(
    buffer: &mut [u8],
    bytes: &[u8],
    addresses: &[Address],
) -> Result<(), WriteAddressesError> {
    let mut errors: Vec<ByteWriteError> = vec![];
    addresses.iter()
        .foreach(|address| write_bytes(buffer, bytes, *address, &mut errors));

    if !errors.is_empty {
        return Err(WriteAddressesError(errors));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0];
        let result: Result<(), WriteAddressesError> =
            write_addresses(&mut buffer, &bytes, &addresses);
        result.map_err(|_| {
            "Error occurred when writing bytes to addresses, but no error was expected.".to_string()
        })?;
        assert_eq!([0x03, 0x87, 0x22, 0xAD], buffer);
        Ok(())
    }

    #[test]
    fn test_write_out_of_bounds() {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [4];
        let result = write_addresses(&mut buffer, &bytes, &addresses);
        match result {
            Ok(_) => panic!("Writing bytes succeeded, but should not have."),
            Err(errs) => {
                assert_eq!(errs.0.len(), 2);
                assert_eq!(
                    errs.0[0].to_string(),
                    "Error writing byte 0x03 at address 4"
                );
                assert_eq!(
                    errs.0[1].to_string(),
                    "Error writing byte 0x87 at address 5"
                );
            }
        }
    }

    #[test]
    fn test_write_partially_out_of_bounds() {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [3];
        let result = write_addresses(&mut buffer, &bytes, &addresses);
        match result {
            Ok(_) => panic!("Writing bytes succeeded, but should not have."),
            Err(errs) => {
                assert_eq!(errs.0.len(), 1);
                assert_eq!(
                    errs.0[0].to_string(),
                    "Error writing byte 0x87 at address 4"
                );
            }
        }
    }

    #[test]
    fn test_write_multiple_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0, 2];
        let result: Result<(), WriteAddressesError> =
            write_addresses(&mut buffer, &bytes, &addresses);
        result.map_err(|_| {
            "Error occurred when writing bytes to addresses, but no error was expected.".to_string()
        })?;
        assert_eq!([0x03, 0x87, 0x03, 0x87], buffer);
        Ok(())
    }

    #[test]
    fn test_write_overlapping_addresses() -> Result<(), String> {
        let mut buffer = [0x00, 0x01, 0x22, 0xAD];
        let bytes = [0x03, 0x87];
        let addresses = [0, 1];
        let result: Result<(), WriteAddressesError> =
            write_addresses(&mut buffer, &bytes, &addresses);
        result.map_err(|_| {
            "Error occurred when writing bytes to addresses, but no error was expected.".to_string()
        })?;
        assert_eq!([0x03, 0x03, 0x87, 0xAD], buffer);
        Ok(())
    }
}
