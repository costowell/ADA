use anyhow::anyhow;
use log::{debug, error};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;
use serde_aux::prelude::*;
use serde_json::json;

/// API interpreted directly from bartender:
/// https://github.com/ComputerScienceHouse/bartender/blob/e99ffc23129821e6ca6b2d93c6771f1410f3b742/src/routes/compat/users.rs

const DRINK_ENDPOINT: &str = "https://drink.csh.rit.edu";

pub struct DrinkApi {
    secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrinkUser {
    pub uid: String,
    pub cn: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub drink_balance: i64,
}

impl DrinkApi {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    fn http(&self) -> Client {
        let mut map = HeaderMap::new();
        map.insert(
            "X-Auth-Token",
            HeaderValue::from_str(self.secret.clone().as_str()).unwrap(),
        );
        Client::builder().default_headers(map).build().unwrap()
    }

    pub async fn get_credits(&self, uid: &str) -> anyhow::Result<DrinkUser> {
        let http = self.http();
        let res: serde_json::Value = http
            .get(format!("{}/users/credits?uid={}", DRINK_ENDPOINT, uid))
            .send()
            .await?
            .json()
            .await?;
        let message = res.get("message").unwrap();
        if let Some(user) = res.get("user") {
            debug!("Got credits for '{}': {}", uid, message);
            let user = serde_json::from_value::<DrinkUser>(user.clone())?;
            Ok(user)
        } else {
            Err(anyhow!("Failed to get credits for '{}': {}", uid, message))
        }
    }

    pub async fn set_credits(&self, uid: &str, credits: i64) -> anyhow::Result<()> {
        let http = self.http();
        let res = http
            .put(format!("{}/users/credits", DRINK_ENDPOINT))
            .json(&json!({
                "uid": uid,
                "drinkBalance": credits
            }))
            .send()
            .await?;
        let status = res.status();
        let json: serde_json::Value = res.json().await?;

        if status.is_success() {
            debug!("Success: {}", json.get("message").unwrap());
        } else {
            error!("Error: {}", json.get("message").unwrap());
        }

        Ok(())
    }
}
