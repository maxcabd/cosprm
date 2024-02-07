pub mod add_entry;

pub mod nucc_binary_handler;

use crc::{Crc, CRC_32_BZIP2};

pub fn calc_crc32(data: &str) -> [u8; 4] {
    let crc = Crc::<u32>::new(&CRC_32_BZIP2);
    let mut digest = crc.digest();
    digest.update(data.as_bytes());
    digest.finalize().to_le_bytes()
}
