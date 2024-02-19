use std::io::{Cursor, Read};
use byteorder::ReadBytesExt;
use bytes::{BufMut, BytesMut};
use serde::de;
use crate::codec::Codec;
use crate::error::{CResult, Error};

#[derive(Clone, Copy)]
pub struct JsonCodec {

}

impl Codec for JsonCodec {
    fn encode<T>(&self, value: &T) -> CResult<Vec<u8>>
        where T: ?Sized + serde::Serialize {

        let serialized = serde_json::to_string(&value);

        match serialized {
            Ok(serde) => {
                let bytes = serde.as_bytes();

                let mut buf = BytesMut::with_capacity(bytes.len() + 8);
                buf.put_u64(bytes.len() as u64);
                buf.put(bytes);

                Ok(buf.to_vec())
            }
            Err(err) => {
                Err(Error::Internal(err.to_string()))
            }
        }
    }

    fn decode<'a, R>(&self, value: &'a [u8]) -> CResult<R>
        where R: de::Deserialize<'a> {

        let mut cursor = Cursor::new(value);
        let len = cursor.read_u64::<byteorder::LittleEndian>().unwrap() as usize;
        let mut bytes = vec![0; len];
        cursor.read_exact(&mut bytes).unwrap();

        let r = String::from_utf8(bytes).unwrap();
        let rs = serde_json::from_slice(&r.as_bytes());
        match rs {
            Ok(r) => {
                Ok(r)
            }
            Err(err) => {
                Err(Error::Parse(err.to_string()))
            }
        }
    }
}

impl JsonCodec {
    pub fn new() -> Self {
        JsonCodec {

        }
    }
}