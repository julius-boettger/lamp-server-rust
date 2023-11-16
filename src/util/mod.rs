pub mod govee;
pub enum HttpMethod { Get, Put }

pub fn digest_sha256(string: &str) -> String {
    sha256::digest(string)
}

pub async fn send_api_request(method: HttpMethod, url: &str, headers: Option<Vec<(&str, &str)>>) -> Result<serde_json::Value, &'static str> {
    let client =  reqwest::Client::new();
    let mut request = match method {
        HttpMethod::Get => client.get(url),
        HttpMethod::Put => client.put(url)
    };

    // set headers (if given)
    if let Some(headers) = headers {
        for header in headers {
            request = request.header(header.0, header.1);
        }
    }

    // set body (if given)
    if let HttpMethod::Put = method {
        // TODO set body for request
    }
    
    let result = request.send().await;
    let Ok(response) = result else {
        // TODO improve error message
        return Err("request failed");
    };

    let json = response.json::<serde_json::Value>().await;
    if let Ok(json) = json {
        Ok(json)
    } else {
        // TODO improve error message
        Err("parsing response json failed")
    }
}