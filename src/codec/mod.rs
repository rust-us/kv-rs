pub mod json_codec;
pub mod bytes_codec;
mod bytes_codec2;

pub trait Codec {
    fn codec_name<T>(&self) -> String;
}
