pub mod govee;
pub enum HttpMethod { Get, Put }

pub fn digest_sha256(string: &str) -> String {
    sha256::digest(string)
}

pub async fn send_api_request(url: &str, method: HttpMethod) -> Option<reqwest::Response> {
    let client =  reqwest::Client::new();
    let mut request = match method {
        HttpMethod::Get => client.get(url),
        HttpMethod::Put => client.put(url)
    };

    // TODO headers as function argument
    request = request.header("Govee-API-Key", crate::res::secrets::govee::API_KEY);
    //request = request.header("Content-Type", "application/json");

    if let HttpMethod::Put = method {
        // TODO append body to request
    }
    
    // send request and try to return Some(Response)
    request.send().await.ok()
}