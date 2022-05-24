use aws_sdk_apigatewaymanagement::types::Blob;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketMessage<T> {
    pub action: String,
    pub data: T
}

impl<'a, T: serde::de::Deserialize<'a>> WebsocketMessage<T> {
    pub fn new(action: &'a str, data: T) -> WebsocketMessage<T> {
        WebsocketMessage {
            action: action.to_string(),
            data
        }
    }
}

impl<'a, T: serde::ser::Serialize> From<WebsocketMessage<T>> for Blob {
    fn from(item: WebsocketMessage<T>) -> Self {
        Blob::new(serde_json::to_string(&item).unwrap().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use aws_sdk_apigatewaymanagement::types::Blob;
    use serde::Serialize;

    use crate::WebsocketMessage;

    #[derive(Serialize, Debug)]
    struct Thing {
        foo: Option<String>,
    }

    #[test]
    fn test_into_blob() {
        let msg_obj = WebsocketMessage {
            action: "Thing".to_string(),
            data: Thing {
                foo: Some("Bar".to_string())
            }
        };

        let blob: Blob = msg_obj.into();
        println!("blob: {:?}", &blob);
    }
}
