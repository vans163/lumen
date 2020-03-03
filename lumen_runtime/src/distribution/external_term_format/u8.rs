use std::convert::TryInto;
use std::mem;

use liblumen_alloc::erts::exception::InternalResult;

use super::try_split_at;

pub fn decode(bytes: &[u8]) -> InternalResult<(u8, &[u8])> {
    try_split_at(bytes, mem::size_of::<u8>()).map(|(len_bytes, after_len_bytes)| {
        let len_array = len_bytes.try_into().unwrap();
        let len_u8 = u8::from_be_bytes(len_array);

        (len_u8, after_len_bytes)
    })
}
