use std::sync::Arc;
use crate::application::dtos::{CreateOrderInput, OrderOutput};
use crate::domain::entities::{Order, OrderItem};
use crate::domain::errors::DomainError;
use crate::domain::repositories::OrderRepository;

pub struct CreateOrderUseCase {
    repo: Arc<dyn OrderRepository>,
}

impl CreateOrderUseCase {
    pub fn new(repo: Arc<dyn OrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: CreateOrderInput) -> Result<OrderOutput, DomainError> {
        let items: Vec<OrderItem> = input.items.into_iter().map(|i| OrderItem {
            product_id: i.product_id,
            quantity: i.quantity,
            unit_price: i.unit_price,
        }).collect();

        let order = Order::new(input.customer_id, items)?;
        self.repo.save(&order).await?;

        Ok(OrderOutput {
            public_id: order.public_id.to_string(),
            customer_id: order.customer_id,
            status: order.status,
            total: order.total(),
            created_at: order.created_at.to_rfc3339(),
        })
    }
}
