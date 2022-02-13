use crate::{
    randomizer::{RomWrite, RomRead},
    error::KatamRandoError,
};
use std::fs::File;

pub struct RomFile<'a> {
    pub rom_file: &'a mut File,
}

impl<'a> RomRead for RomFile<'a> {
    fn read_rom(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        self.rom_file.read_to_end(buf).map_err(|e| KatamRandoError::RomIO(e))
    }
}

impl<'a> RomWrite for RomFile<'a> {
    fn write_rom(&mut self, buf: &[u8]) -> Result<()> {
        self.rom_file.write_all(buf).map_err(|e| KatamRandoError::RomIO(e))
    }
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
