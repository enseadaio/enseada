use actix_web::web::Json;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
    })
}