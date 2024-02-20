use std::io::{Cursor, Read, Write};
use byteorder::ReadBytesExt;
use bytes::{BufMut, BytesMut};
use serde::de;
use tokio::io::AsyncWriteExt;
use crate::codec::Codec;
use crate::error::{CResult, Error};

#[derive(Clone, Copy)]
pub struct JsonCodec {

}

impl Codec for JsonCodec {
    fn encode<T>(&self, value: &T) -> CResult<Vec<u8>>
        where T: ?Sized + serde::Serialize {

        let str = serde_json::to_string(&value);

        match str {
            Ok(serde) => {
                let bytes = serde.as_bytes();

                let mut buf = BytesMut::with_capacity(bytes.len() + 8);
                // len  8 byute
                buf.put_u64(bytes.len() as u64);
                buf.put(bytes);

                Ok(buf.to_vec())
            }
            Err(err) => {
                Err(Error::Internal(err.to_string()))
            }
        }
    }

    fn decode<R>(&self, value: &[u8], contains_len: bool) -> CResult<R>
        where R: for<'a> de::Deserialize<'a> {

        let mut cursor = Cursor::new(value);

        let len = if contains_len {
            8 + value.len()
        } else {
            value.len()
        };

        let mut bytes = Vec::new();
        if contains_len {
            let len = cursor.read_u64::<byteorder::BigEndian>().unwrap() as usize;
            let mut b = vec![0; len];
            cursor.read_exact(&mut b).unwrap();
            bytes = b.to_vec();
        } else {
            bytes = value.to_vec();
        }

        let str = String::from_utf8(bytes).unwrap();
        let r: serde_json::Result<R> = serde_json::from_str(&str);
        match r {
            Ok(r) => {
                Ok(r)
            }
            Err(err) => {
                Err(Error::Parse(err.to_string()))
            }
        }
        //
        // Err(Error::Abort)
    }
}

impl JsonCodec {
    pub fn new() -> Self {
        JsonCodec {

        }
    }
}