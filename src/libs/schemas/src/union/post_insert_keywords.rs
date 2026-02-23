use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Req {
    pub keywords: Vec<String>,
    pub window_size: usize,
    pub weight_increase: u8,
}
