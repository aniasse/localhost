use std::collections::HashMap;

pub fn parse_request(request: &str) -> (String, String, HashMap<String, String>, String) {
    let mut headers = HashMap::new();
    let mut lines = request.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.get(0).unwrap_or(&"").to_string();
    let path = parts.get(1).unwrap_or(&"").to_string();

    let lines_clone = lines.clone();

    for line in lines {
        if line.is_empty() {
            break;
        }
        let header: Vec<&str> = line.splitn(2, ':').collect();
        if header.len() == 2 {
            headers.insert(header[0].trim().to_string(), header[1].trim().to_string());
        }
    }

    let body = lines_clone.collect::<Vec<&str>>().join("\n");
    (method, path, headers, body)
}

pub fn create_response(status_code: u16, status_text: &str, body: String) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_text,
        body.len(),
        body
    )
}