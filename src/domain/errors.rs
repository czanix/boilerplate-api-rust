use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Pedido deve ter pelo menos um item")]
    EmptyOrder,

    #[error("Quantidade deve ser positiva")]
    InvalidQuantity,

    #[error("Pedido já cancelado")]
    AlreadyCancelled,

    #[error("Não é possível cancelar pedido entregue")]
    DeliveredCancel,

    #[error("Pedido não encontrado")]
    NotFound,

    #[error("Erro interno: {0}")]
    Internal(String),
}
