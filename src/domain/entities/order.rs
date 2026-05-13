use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Option<i64>,
    pub public_id: Uuid,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(customer_id: String, items: Vec<OrderItem>) -> Result<Self, DomainError> {
        if items.is_empty() {
            return Err(DomainError::EmptyOrder);
        }
        for item in &items {
            if item.quantity <= 0 {
                return Err(DomainError::InvalidQuantity);
            }
        }

        Ok(Self {
            id: None,
            public_id: Uuid::new_v4(),
            customer_id,
            items,
            status: "pending".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn total(&self) -> f64 {
        self.items.iter().map(|i| i.quantity as f64 * i.unit_price).sum()
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status == "delivered" { return Err(DomainError::DeliveredCancel); }
        if self.status == "cancelled" { return Err(DomainError::AlreadyCancelled); }
        self.status = "cancelled".to_string();
        self.updated_at = Utc::now();
        Ok(())
    }
}
