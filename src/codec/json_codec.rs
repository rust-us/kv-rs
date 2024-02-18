use crate::codec::{Deserialize, Serialize};
use crate::error::{CResult, Error};

pub struct JsonCodec {

}

impl Serialize<String> for JsonCodec {
    fn encode<T>(&self, value: &T) -> CResult<String>
        where T: ?Sized + serde::Serialize {
        let serialized = serde_json::to_string(&value);

        match serialized {
            Ok(serde) => {
                Ok(serde)
            }
            Err(err) => {
                Err(Error::Internal(err.to_string()))
            }
        }
    }
}

impl Deserialize for JsonCodec {
    // fn deserde<T>(&self, value: String) -> CResult<String> {
    //
    //     let rs: Result<String, serde_json::Error> = serde_json::from_str(value.as_str());
    //
    //     let a = rs.unwrap();
    //
    //     Ok(a)
    // }
}

impl JsonCodec {
    pub fn new() -> Self {
        JsonCodec {

        }
    }
}