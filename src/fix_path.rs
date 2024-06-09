use core::fmt::Debug;
use core::mem;

use object::endian::{LittleEndian as LE, U16Bytes};
use object::{LittleEndian, pe, ReadRef, U32};
use object::pod::Pod;
use object::read::{Bytes, ReadError, Result};
use object::read::pe::DataDirectories;
use object::pod;

use super::ImageNtHeaders;

// object::pod::unsafe_impl_pod!(
//     FixDataHeader
// );

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FixDataHeader {
    pub version: U32<LE>,
    pub fix_path_size: U32<LE>,
    pub idata_name_table_size: U32<LE>,
    pub didata_name_table_size: U32<LE>,
}

// #[derive(Debug, Clone, Copy)]
// pub struct DataDirectories<'data> {
//     entries: &'data [pe::ImageDataDirectory],
// }
//
// pub fn parse(data: &'data [u8], number: u32) -> Result<Self> {
//     let entries = data
//         .read_slice_at(0, number as usize)
//         .read_error("Invalid PE number of RVA and sizes")?;
//     Ok(DataDirectories { entries })
// }

// pub fn read_fix_path_header<'data, R: ReadRef<'data>>(
//     data: R,
//     offset: u32
// ) -> Result<(FixDataHeader, str)> {
//     // let fix_path_header = data
//     //     .read_at::<FixDataHeader>(offset as u64)
//     //     .read_error("Invalid PE number of RVA and sizes")?;
//     // Ok(fix_path_header)
//     //Err("asdf")
//     Ok(())
// }

