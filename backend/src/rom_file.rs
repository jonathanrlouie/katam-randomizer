use crate::{
    graph::{DoorData, Graph},
    rom::{ByteWriteError, Rom, WriteAddressesError},
};
use std::{
    cmp::Eq,
    fmt::Debug,
    fs::File,
    hash::Hash,
    io::{Read, Write},
};

type Address = usize;

pub trait RomRead {
    fn read_rom(&mut self, buf: &mut Vec<u8>) -> Result<(), std::io::Error>;
}

pub trait RomWrite {
    fn write_rom(&mut self, buf: &[u8]) -> Result<(), std::io::Error>;
}

pub struct RomFile<'a, R: RomRead + RomWrite> {
    pub rom_file: &'a mut R,
}

impl RomRead for File {
    fn read_rom(&mut self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        self.read_to_end(buf)?;
        Ok(())
    }
}

impl RomWrite for File {
    fn write_rom(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.write_all(buf)
    }
}

impl<'a, R: RomRead + RomWrite> Rom for RomFile<'a, R> {
    fn write_data<N, E, G>(&mut self, graph: &mut G) -> Result<(), std::io::Error>
    where
        N: Debug + Eq + Hash,
        G: Graph<N, E> + DoorData<N>,
    {
        let mut buffer = Vec::new();
        self.rom_file.read_rom(&mut buffer)?;

        for (start_node_id, end_node_id) in graph.get_edges() {
            // TODO: Debug log level
            println!("edge: {:?}, {:?}", start_node_id, end_node_id);

            let addresses_to_replace = graph
                .door_data()
                .get(&start_node_id)
                .map(|t| t.1.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "No ROM addresses found for start node ID {:?}",
                        start_node_id
                    )
                });

            let dest = graph
                .door_data()
                .get(&end_node_id)
                .map(|t| t.0)
                .unwrap_or_else(|| {
                    panic!(
                        "No destination data found for end node ID {:?}",
                        end_node_id
                    )
                });

            write_addresses(&mut buffer, &dest, &addresses_to_replace)
                .unwrap_or_else(|e| panic!("Failed to write to rom addresses: {:?}", e));
        }

        self.rom_file.write_rom(&buffer)?;
        Ok(())
    }
}

fn write_byte(buffer: &mut [u8], byte: u8, address: Address, errors: &mut Vec<ByteWriteError>) {
    match buffer.get_mut(address) {
        Some(elem) => *elem = byte,
        None => errors.push(ByteWriteError { byte, address }),
    }
}

fn write_bytes(
    buffer: &mut [u8],
    bytes: &[u8],
    address: Address,
    errors: &mut Vec<ByteWriteError>,
) {
    bytes
        .iter()
        .enumerate()
        .for_each(|(idx, byte)| write_byte(buffer, *byte, address + idx, errors));
}

fn write_addresses(
    buffer: &mut [u8],
    bytes: &[u8],
    addresses: &[Address],
) -> std::result::Result<(), WriteAddressesError> {
    let mut errors: Vec<ByteWriteError> = vec![];
    addresses
        .iter()
        .for_each(|address| write_bytes(buffer, bytes, *address, &mut errors));

    if !errors.is_empty() {
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
