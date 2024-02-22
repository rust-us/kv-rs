use std::io::{Read, Write};
use serde::{de, Deserialize};
use tokio::io::AsyncWriteExt;
use crate::codec::Codec;
use crate::error::{CResult, Error};

#[derive(Clone, Copy)]
pub struct JsonCodec {

}

impl JsonCodec {
    pub fn new() -> Self {
        JsonCodec {

        }
    }

    pub fn encode<T>(&self, value: &T) -> CResult<String>
        where T: ?Sized + serde::Serialize {

        let str = serde_json::to_string(&value);

        match str {
            Ok(serde) => {
                Ok(serde)
            }
            Err(err) => {
                Err(Error::Internal(err.to_string()))
            }
        }
    }

    pub fn decode<R>(&self, value: &String) -> CResult<R>
        where R: for<'a> de::Deserialize<'a> {

        let r: serde_json::Result<R> = serde_json::from_str(&value);
        match r {
            Ok(r) => {
                Ok(r)
            }
            Err(err) => {
                Err(Error::Parse(err.to_string()))
            }
        }
    }
}

impl Codec for JsonCodec {
    fn codec_name<T>(&self) -> String {
        "JsonCodec".to_string()
    }
}

#[cfg(test)]
mod test {
    use serde_derive::{Deserialize, Serialize};
    use crate::codec::json_codec::JsonCodec;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Persion {
        name: String,

        age: i16,

        address: String,
    }

    #[test]
    fn test_json_decode() {
        let codec = JsonCodec::new();
        let rng = rand::thread_rng();

        // encode
        let mut persion_list = Vec::<Persion>::new();
        let mut buf = Vec::<String>::new();
        for i in 0..88 {
            let p = Persion {
                name: format!("name{}", i),
                age: i + 1,
                address: format!("address{}", i),
            };
            persion_list.push(p.clone());

            let b = codec.encode(&p).unwrap();
            buf.push(b);
        }

        let mut i_for_test = 0;
        // decode
        for p in buf {
            let r: Persion = codec.decode(&p).unwrap();
            println!("codec.decode: {:?}", r);

            let cache_p = persion_list.get(i_for_test).unwrap();
            assert_eq!(&r.name, &cache_p.name);
            assert_eq!(&r.address, &cache_p.address);
            assert_eq!(&r.age, &cache_p.age);

            i_for_test += 1;
        }

        assert_eq!(1, 1);
    }

}
