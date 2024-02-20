use serde::de;
use crate::error::CResult;

pub mod json_codec;
pub mod bytes_codec;
pub mod keycodec;

pub trait Codec {
    /// 序列化
    fn encode<T>(&self, value: &T) -> CResult<Vec<u8>>
        where T: ?Sized + serde::Serialize;

    /// 反序列化
    fn decode<R>(&self, value: &[u8], contains_len: bool) -> CResult<R>
        where R: for<'a> de::Deserialize<'a>;
}
