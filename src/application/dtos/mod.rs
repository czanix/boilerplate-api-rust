use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateOrderInput {
    pub customer_id: String,
    pub items: Vec<ItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct ItemInput {
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Serialize)]
pub struct OrderOutput {
    pub public_id: String,
    pub customer_id: String,
    pub status: String,
    pub total: f64,
    pub created_at: String,
}
