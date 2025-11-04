use std::io::{Cursor, Read};
use byteorder::ReadBytesExt;
use serde::Deserialize;
use tokio_util::bytes::{BufMut, BytesMut};
use crate::codec::Codec;
use crate::error::{CResult, Error};

#[derive(Clone, Copy)]
pub struct BytesCodec {

}

impl BytesCodec {
    pub fn new() -> Self {
        BytesCodec {

        }
    }

    pub fn encode<T>(&self, value: &T) -> CResult<Vec<u8>>
        where T: ?Sized + serde::Serialize {

        let encoded= serde_json::to_string(&value);
        match encoded {
            Ok(serde) => {
                let bytes = serde.as_bytes();

                let mut buf = BytesMut::with_capacity(8 + bytes.len());
                // len  8 byute
                buf.put_u64(bytes.len() as u64);
                buf.put(bytes);

                Ok(buf.to_vec())
            }
            Err(e) => {
                Err(Error::Internal(e.to_string()))
            }
        }
    }

    pub fn decode_bytes<R>(&self, value: &[u8], decode_len: bool) -> CResult<R> where R: for<'a> Deserialize<'a> {

        let mut bytes = Vec::new();
        if decode_len {
            let mut cursor = Cursor::new(value);

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
    }

    pub fn decode_cursor<R>(&self, cursor: &mut Cursor<&[u8]>) -> CResult<Option<R>> where R: for<'a> Deserialize<'a> {
        if cursor.position() >= cursor.get_ref().len() as u64 {
            return Ok(None);
        }

        let len = cursor.read_u64::<byteorder::BigEndian>().unwrap() as usize;
        let mut by = vec![0; len];
        cursor.read_exact(&mut by).unwrap();

        match self.decode_bytes(by.as_slice(), false) {
            Ok(r) => {
                Ok(Some(r))
            },
            Err(err) => {
                Err(Error::Parse(err.to_string()))
            }
        }
    }
}

impl Codec for BytesCodec {
    fn codec_name<T>(&self) -> String {
        "BytesCodec".to_string()
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Read};
    use byteorder::ReadBytesExt;
    use bytes::{BufMut, BytesMut};
    use serde_derive::{Deserialize, Serialize};
    use crate::codec::bytes_codec::BytesCodec;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Persion {
        name: String,

        age: i16,

        address: String,
    }

    #[test]
    fn test_decode_bytes() {
        let codec = BytesCodec::new();

        // encode
        let mut persion_list = Vec::<Persion>::new();
        let mut buf = BytesMut::with_capacity(1024);
        for i in 0..88 {
            let p = Persion {
                name: format!("name{}", i),
                age: i + 1,
                address: format!("address{}", i),
            };
            persion_list.push(p.clone());

            let b = codec.encode(&p).unwrap();
            buf.put(b.as_slice());
        }
        let get_bytes = buf.as_ref();

        let mut i_for_test = 0;
        // decode
        let mut cursor = Cursor::new(get_bytes);
        loop {
            if cursor.position() >= cursor.get_ref().len() as u64 {
                break;
            }

            let len = cursor.read_u64::<byteorder::BigEndian>().unwrap() as usize;
            let mut by = vec![0; len];
            cursor.read_exact(&mut by).unwrap();

            let r: Persion = codec.decode_bytes(&by, false).unwrap();
            println!("codec.decode: {:?}", r);

            let cache_p = persion_list.get(i_for_test).unwrap();
            assert_eq!(&r.name, &cache_p.name);
            assert_eq!(&r.address, &cache_p.address);
            assert_eq!(&r.age, &cache_p.age);

            i_for_test += 1;
        }

        assert_eq!(1, 1);
    }

    #[test]
    fn test_decode_cursor() {
        let codec = BytesCodec::new();
        let rng = rand::thread_rng();

        // encode
        let mut persion_list = Vec::<Persion>::new();
        let mut buf = BytesMut::with_capacity(1024);
        for i in 0..66 {
            let p = Persion {
                name: format!("name{}", i),
                age: i + 1,
                address: format!("address{}", i),
            };
            persion_list.push(p.clone());

            let b = codec.encode(&p).unwrap();
            buf.put(b.as_slice());
        }
        let get_bytes = buf.as_ref();

        let mut i_for_test = 0;
        // decode
        let mut cursor = Cursor::new(get_bytes);
        loop {
            let p: Option<Persion> = codec.decode_cursor(&mut cursor).unwrap();
            if p.is_none() {
                break;
            }

            let r: Persion = p.unwrap();
            println!("codec.decode: {:?}", r);

            let cache_p = persion_list.get(i_for_test).unwrap();
            assert_eq!(&r.name, &cache_p.name);
            assert_eq!(&r.address, &cache_p.address);
            assert_eq!(&r.age, &cache_p.age);

            i_for_test += 1;
        }
        assert!(cursor.position() >= cursor.get_ref().len() as u64);
    }
}
