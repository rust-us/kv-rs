use serde::de;
use crate::error::CResult;

pub mod json_codec;
pub mod bytes_codec;
pub mod keycodec;

pub trait Serialize<V> {
    /// 序列化
    fn serde<T>(&self, value: &T) -> CResult<V>
        where T: ?Sized + serde::Serialize;
}

pub trait Deserialize {
    // fn deserde<T>(&self, value: &T) -> CResult<String>;
}