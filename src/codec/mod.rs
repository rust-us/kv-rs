pub mod json_codec;
pub mod bytes_codec;

pub trait Codec {
    fn codec_name<T>(&self) -> String;
}
