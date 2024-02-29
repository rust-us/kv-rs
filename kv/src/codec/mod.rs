pub mod json_codec;
pub mod bytes_codec;
mod bytes_codec2;

/// Define a codec type and implement the Codec trait
pub trait Codec {
    fn codec_name<T>(&self) -> String;
}
