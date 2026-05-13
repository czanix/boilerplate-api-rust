use async_trait::async_trait;
use crate::domain::entities::Order;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn save(&self, order: &Order) -> Result<(), DomainError>;
    async fn find_by_public_id(&self, public_id: uuid::Uuid) -> Result<Option<Order>, DomainError>;
    async fn update(&self, order: &Order) -> Result<(), DomainError>;
}
