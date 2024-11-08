use axum::{
    extract::{Json, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use worker::Env;

use crate::{anyhow_result::AppResult, AppState};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VkRequest {
    Confirmation {
        group_id: u64,
        secret: String,
    },
    MessageNew {
        object: MessageObjectType,
        secret: String,
    },
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct MessageObjectType {
    pub message: MessageType,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct MessageType {
    pub id: i64,
    pub date: i64,
    pub peer_id: i64,
    pub from_id: i64,
    pub text: String,
}

#[axum_macros::debug_handler]
pub async fn handle_request(
    headers: HeaderMap,
    State(AppState { env, .. }): State<AppState>,
    Json(data): Json<VkRequest>,
) -> AppResult<Response> {
    let token = env.secret("VK_GROUP_CALLBACK_API_SECRET")?.to_string();
    match data {
        VkRequest::Confirmation { group_id, secret } => {
            assert_token(&token, &secret)?;
            let my_group = env.secret("VK_GROUP_ID")?.to_string();
            let group_confirm_token = env.secret("VK_GROUP_CONFIRM_TOKEN")?;
            if group_id.to_string() == my_group {
                return Ok(group_confirm_token.to_string().into_response());
            } else {
                return Ok((axum::http::StatusCode::UNAUTHORIZED, "Wrong group ID").into_response());
            }
        }

        VkRequest::MessageNew { ref object, secret } => {
            assert_token(&token, &secret)?;
            let vk_group_token = env.secret("VK_GROUP_ACCESS_TOKEN")?.to_string();

            tracing::info!("Received message: {:?}", object);
            let msg = format!("Received: {:?}", &object);
            send_message_to_vk(&vk_group_token, &msg, object.message.peer_id).await?;
        }
    }

    return Ok("ok".into_response());
}

pub async fn send_message_to_vk(vk_group_token: &str, text: &str, peer_id: i64) -> AppResult<()> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.vk.com/method/messages.send")
        .query(&[("v", "5.199")])
        .query(&[("peer_id", peer_id.to_string())])
        .query(&[("message", text)])
        .query(&[("random_id", "0")])
        .query(&[("access_token", vk_group_token)])
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("VK API error: {}", res.status()))?
    }
}

fn assert_token(true_token: &str, provided_token: &str) -> AppResult<()> {
    if true_token == provided_token {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Secret token provided is not correct"))?
    }
}
