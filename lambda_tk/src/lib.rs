#![feature(let_else)]

pub mod model;

//use std::fmt::Debug;
//use std::future::Future;
use anyhow::anyhow;
use aws_lambda_events::apigw::ApiGatewayWebsocketProxyRequest;
use lambda_runtime::LambdaEvent;
use tracing::{debug,warn};

pub use lambda_tk_macro::main;
use crate::model::WebsocketMessage;

pub fn apig_ws_extract_message_content<'a, T: serde::de::Deserialize<'a>>(
    evt: &'a LambdaEvent<ApiGatewayWebsocketProxyRequest>,
) -> anyhow::Result<T, anyhow::Error> {
    debug!(?evt, "Raw incoming event");

    let Some(body) = &evt.payload.body else {
        warn!("missing body in incoming API GW Request");
        return Err(anyhow!("missing body in incoming API GW Request"));
    };

    let msg: WebsocketMessage<T> = serde_json::from_str(&body)?;
    Ok(msg.data)
}

// TODO: get this working to use in sa-solver-sns-result-ws-dispatcher
/*
async fn lambda_sns_mapper<T: serde::de::DeserializeOwned + Debug, F, Fut>(
    handler: F,
    event: LambdaEvent<SnsEvent>,
    app_ctx: &AppContext
) -> Result<(), lambda_runtime::Error>
    where F: Fn(T, &AppContext) -> Fut,
          Fut: Future<Output=Result<(), lambda_runtime::Error>>
{
    for record in event.payload.records {
        if let Some(msg_raw) = record.sns.message {
            //tracing::debug!(%msg_raw, "raw message");
            let decoded = serde_json::from_str(&msg_raw)?;
            tracing::debug!(?decoded, "decoded message");

            handler(decoded, app_ctx);
        }
    }
    Ok(())
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
