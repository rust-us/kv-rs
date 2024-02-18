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
    fn decode<'a, R>(&self, value: &'a [u8]) -> CResult<R>
        where R: de::Deserialize<'a>;
}
