extern crate bytes;
#[macro_use]
extern crate failure;

mod error;
mod types;

use self::error::{ColferError, ColferResult};

pub const COLFER_SIZE_MAX: usize = 16 * 1024 * 1024;
pub const COLFER_LIST_MAX: usize = 64 * 1024;

pub trait ColferSerializable<'a> {
    fn colf_marshal_to(&self, buf: &mut Vec<u8>) -> usize;
    fn colf_marshal_len(&self) -> ColferResult<usize>;
    fn colf_unmarshal(&mut self, data: &'a [u8]) -> ColferResult<usize>;

    fn colf_marshal_binary(&self) -> ColferResult<Vec<u8>> {
        let l = self.colf_marshal_len()?;
        let mut data = Vec::with_capacity(l);
        let _ = self.colf_marshal_to(&mut data);
        Ok(data)
    }

    fn colf_unmarshal_binary(&mut self, data: &'a [u8]) -> ColferResult<usize> {
        let byte = self.colf_unmarshal(data)?;
        if byte >= data.len() {
            Ok(byte)
        } else {
            Err(ColferError::Tail { byte })
        }
    }
}

pub use self::types::ColferTypes;
