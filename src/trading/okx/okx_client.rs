use std::env;

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

// ... (保持 Ticker、Balance 和 ErrorResponse 结构体的定义不变)
#[derive(Serialize, Deserialize, Debug)]
pub struct Ticker {
    last: Option<String>,
    // 其他字段...
}

#[derive(Serialize, Deserialize)]
struct OkxApiErrorResponse {
    msg: String,
    code: String,
}

// 通用的响应结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct OkxApiResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: T,
}

pub struct OkxClient {
    client: Client,
    api_key: String,
    api_secret: String,
    passphrase: String,
}

impl OkxClient {
    fn new(api_key: String, api_secret: String, passphrase: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5)) // 设置请求超时时间为3秒
            .build()
            .unwrap();
        OkxClient {
            client,
            api_key,
            api_secret,
            passphrase,
        }
    }

    fn generate_signature(
        &self,
        timestamp: &str,
        method: &Method,
        path: &str,
        body: &str,
    ) -> String {
        let sign_payload = format!("{}{}{}{}", timestamp, method.as_str(), path, body);
        let mut hmac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
        hmac.update(sign_payload.as_bytes());
        let signature = base64::encode(hmac.finalize().into_bytes());
        signature
    }

    pub async fn send_request<T: for<'a> Deserialize<'a>>(
        &self,
        method: Method,
        path: &str,
        body: &str,
    ) -> Result<T, anyhow::Error> {
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S.%3fZ")
            .to_string();
        let signature = self.generate_signature(&timestamp, &method, path, body);

        let exp_time = chrono::Utc::now().timestamp_millis() + 500;

        let url = format!("https://www.okx.com{}", path);
        println!("request okx url:{}", url);
        let is_simulated_trading = env::var("IS_SIMULATED_TRADING").unwrap_or(1.to_string());

        let request_builder = self
            .client
            .request(method, &url)
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", signature)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .header("expTime", exp_time.to_string())
            // expTime 	String 	否 	请求有效截止时间。Unix时间戳的毫秒数格式，如 1597026383085
            //设置是否是模拟盘
            .body(body.to_string());

        let request_builder = if is_simulated_trading == "1" {
            request_builder.header("x-simulated-trading", &is_simulated_trading)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;

        let status_code = response.status();
        let response_body = response.text().await?;
        // info!("path:{},okx_response: {}", path, response_body);
        if status_code == StatusCode::OK {
            // println!("okx response body:{:#?}", &response_body);
            let result: OkxApiResponse<T> = serde_json::from_str(&response_body)?;
            // println!("result 1111:{:?}", result);
            Ok(result.data)
        } else {
            let error: OkxApiErrorResponse = serde_json::from_str(&response_body)?;
            println!("okx response body:{:#?}", &response_body);
            Err(anyhow!("请求失败: {}", error.msg))
        }
    }
}

pub fn get_okx_client() -> OkxClient {
    let api_key = env::var("OKX_API_KEY").expect("OKX_API_KEY 不能配置为空");
    let api_secret = env::var("OKX_API_SECRET").expect("OKX_API_SECRET 不能配置为空");
    let passphrase = env::var("OKX_PASSPHRASE").expect("OKX_PASSPHRASE 不能配置为空");

    let okx_client = OkxClient::new(api_key, api_secret, passphrase);
    okx_client
}
