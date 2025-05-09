use hmac::Mac;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ... (保持 Ticker、Balance 和 ErrorResponse 结构体的定义不变)
#[derive(Serialize, Deserialize, Debug)]
pub struct Ticker {
    last: Option<String>,
    // 其他字段...
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    msg: String,
    code: String,
}

pub(crate) struct OkxWebsocket {
    client: Client,
    api_key: String,
    api_secret: String,
    passphrase: String,
}

impl OkxWebsocket {
    fn new(api_key: String, api_secret: String, passphrase: String) -> Self {
        OkxWebsocket {
            client: Client::new(),
            api_key,
            api_secret,
            passphrase,
        }
    }

    // fn generate_signature(&self, timestamp: &str, method: &Method, path: &str, body: &str) -> String {
    //     let sign_payload = format!("{}{}{}{}", timestamp, method.as_str(), path, body);
    //     let mut hmac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).unwrap();
    //     hmac.update(sign_payload.as_bytes());
    //     let signature = base64::encode(hmac.finalize().into_bytes());
    //     signature
    // }
    //
    //     pub(crate) async fn send_request<T: for<'a> Deserialize<'a>>(
    //         &self,
    //         method: Method,
    //         path: &str,
    //         body: &str,
    //     ) -> Result<T> {
    //         let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string();
    //         let signature = self.generate_signature(&timestamp, &method, path, body);
    //
    //         let url = format!("https://www.okx.com{}", path);
    //         let request_builder = self.client
    //             .request(method, &url)
    //             .header("OK-ACCESS-KEY", &self.api_key)
    //             .header("OK-ACCESS-SIGN", signature)
    //             .header("OK-ACCESS-TIMESTAMP", timestamp)
    //             .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
    //             .header("Content-Type", "application/json")
    //             //设置是否是模拟盘
    //             .header("x-simulated-trading", 1)
    //             .body(body.to_string());
    //
    //         let is_simulated_trading = env::var("IS_SIMULATED_TRADING").unwrap_or(1.to_string());
    //         let request_builder = if is_simulated_trading == "1" {
    //             request_builder.header("x-simulated-trading", &is_simulated_trading)
    //         } else {
    //             request_builder
    //         };
    //
    //         let response = request_builder.send()
    //             .await?;
    //
    //         let status_code = response.status();
    //         let response_body = response.text().await?;
    //
    //         if status_code == StatusCode::OK {
    //             // println!("okx_response OK: {}", response_body);
    //             info!("okx_response OK");
    //             let result: T = serde_json::from_str(&response_body)?;
    //             Ok(result)
    //         } else {
    //             let error: ErrorResponse = serde_json::from_str(&response_body)?;
    //             Err(anyhow!("请求失败: {}", error.msg))
    //         }
    //     }

    pub fn get_okx_client() -> OkxWebsocket {
        //真实交易
        // let api_key = "e9f4ac8d-42cf-4616-a870-ba59398a75fd".to_string();
        // let api_secret = "2ACC57D2AD7A1FF5683D80F6E62E5A73".to_string();
        // let passphrase = "Fwc_okx_520".to_string();

        //模拟交易
        // 模拟盘的请求的header里面需要添加 "x-simulated-trading: 1"。
        let api_key = "b6bf48c4-a1fc-45e0-b3f0-f0a544549a67".to_string();
        let api_secret = "63373C32B1B7F6DBFB659A428E859564".to_string();
        let passphrase = "Fwc_okx_520".to_string();

        let okx_client = OkxWebsocket::new(api_key, api_secret, passphrase);
        okx_client
    }
}
