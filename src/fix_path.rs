// use core::fmt::Debug;
// use core::mem;
//
// use object::endian::{LittleEndian as LE, U16Bytes};
// use object::{LittleEndian, pe, ReadRef};
// use object::pod::Pod;
// use object::read::{Bytes, ReadError, Result};

// use super::ImageNtHeaders;

//
// pub fn read_version<'data, R: ReadRef<'data>>(
//     data: R,
//     offset: u32
// ) -> object::Result<(u32)> {
//     let hint = data
//         .skip(offset as usize)
//         .read::<U16Bytes<LE>>()
//         .read_error("Missing PE import thunk hint")?
//         .get(LE);
//     Ok((hint))
//
//     // let offset = address.wrapping_sub(self.section_address);
//     // let mut data = self.section_data;
//     // data.skip(offset as usize)
//     //     .read_error("Invalid PE import thunk address")?;
//     // let hint = data
//     //     .read::<U16Bytes<LE>>()
//     //     .read_error("Missing PE import thunk hint")?
//     //     .get(LE);
//     // let name = data
//     //     .read_string()
//     //     .read_error("Missing PE import thunk name")?;
//     // Ok((hint, name))
// }
