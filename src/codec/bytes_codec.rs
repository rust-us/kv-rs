// use tokio_util::bytes::{BufMut, BytesMut};
// use crate::codec::{Deserialize, Serialize};
// use crate::error::{CResult, Error};
//
// pub struct BytesCodec {
//
// }
//
// impl Serialize<&[u8]> for BytesCodec {
//     fn serde<T>(&self, value: &T) -> CResult<&[u8]> where T: ?Sized + serde::Serialize {
//         match serde_json::to_string(&value) {
//             Ok(ser) => {
//                 let len = ser.len(); // 8 byte
//
//                 let data = ser.as_bytes();
//
//                 let mut buf = BytesMut::with_capacity(len + data.len());
//                 buf.put_u64(len as u64);
//                 buf.put(data);
//
//                 Ok(buf.as_ref())
//             }
//             Err(e) => {
//                 Err(Error::Internal(e.to_string()))
//             }
//         }
//     }
// }
//
// impl Deserialize for BytesCodec {
// }
//
// impl BytesCodec {
//     pub fn new() -> Self {
//         BytesCodec {
//
//         }
//     }
// }
