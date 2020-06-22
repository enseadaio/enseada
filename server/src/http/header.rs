use actix_web::HttpRequest;

pub fn accept(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(http::header::ACCEPT)
        .and_then(|accept| accept.to_str().ok())
        .map(str::to_lowercase)
}

pub fn accept_or(req: &HttpRequest, or: &str) -> String {
    accept(req).unwrap_or_else(|| or.to_string())
}
