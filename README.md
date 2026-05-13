# Czanix Boilerplate — API Rust

> Axum + SQLx + Tokio. Performance de C com ergonomia moderna. Zero runtime overhead, zero garbage collection, zero null pointer em produção.

[![Rust](https://img.shields.io/badge/Rust-1.85-000000?style=flat&logo=rust&logoColor=white)](https://rust-lang.org)
[![Axum](https://img.shields.io/badge/Axum-0.7-orange?style=flat)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16?style=flat&logo=postgresql&logoColor=white)](https://postgresql.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Tech Reference](https://img.shields.io/badge/Czanix-Tech%20Reference-gold)](https://czanix.com/pt/stack)

---

## Por que Rust para APIs?

Não é modismo. É cálculo frio:

| Métrica | Go | Node.js | Rust |
|---------|-----|---------|------|
| Latência P99 | ~2ms | ~8ms | ~0.5ms |
| Memória (idle) | ~15MB | ~50MB | ~3MB |
| Throughput (req/s) | ~80k | ~30k | ~150k |
| Null pointer crash | Possível | Possível | **Impossível** |

**Quando usar Rust:**
- APIs de alta performance (fintech, real-time, gaming)
- Serviços com budget de infra apertado (menos memória = menos custo)
- Edge computing / WASM
- Quando o custo de um bug em produção é muito alto

**Quando NÃO usar:**
- CRUD simples onde time-to-market importa mais que performance
- Time que não tem experiência com ownership/lifetimes
- Prototipação rápida → use Go ou Node.js primeiro, migre depois

Saber quando não usar uma tecnologia é o que separa engenheiro sênior de entusiasta.

---

## Estrutura

```
src/
├── domain/                          # Zero dependências externas
│   ├── entities/
│   │   └── order.rs                 # Struct pura + impl com regras
│   ├── repositories/
│   │   └── order_repository.rs      # Trait (interface Rust)
│   ├── errors.rs                    # Enum tipado — cada erro é explícito
│   └── mod.rs
│
├── application/                     # Casos de uso
│   ├── use_cases/
│   │   ├── create_order.rs
│   │   └── cancel_order.rs
│   ├── dtos/
│   │   ├── create_order_input.rs
│   │   └── order_output.rs
│   └── mod.rs
│
├── infrastructure/                  # Mundo externo
│   ├── database/
│   │   ├── connection.rs            # Pool SQLx
│   │   └── pg_order_repository.rs   # Implementação do trait
│   ├── cache/
│   │   └── redis.rs
│   └── mod.rs
│
├── presentation/                    # HTTP layer
│   ├── handlers/
│   │   └── order_handler.rs         # Axum handlers
│   ├── middleware/
│   │   ├── auth.rs
│   │   ├── rate_limit.rs
│   │   └── security_headers.rs
│   ├── extractors/
│   │   └── validated_json.rs        # Custom extractor com validação
│   ├── router.rs
│   └── mod.rs
│
├── config.rs                        # Configuração tipada
├── lib.rs
└── main.rs                          # Entrypoint — wire everything
```

---

## Início rápido

```bash
# 1. Clone
git clone https://github.com/czanix/boilerplate-api-rust.git meu-projeto
cd meu-projeto

# 2. Build (primeira vez compila deps — leva ~2min)
cargo build

# 3. Ambiente
cp .env.example .env

# 4. Banco + Cache
docker compose up -d

# 5. Migrations
sqlx migrate run

# 6. Desenvolvimento (hot reload)
cargo watch -x run
```

---

## Result Pattern — Rust já tem nativamente

```rust
// Rust tem Result<T, E> como tipo nativo — o compilador FORÇA o tratamento
// errors.rs — cada erro de domínio é um variant do enum
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Pedido sem itens")]
    EmptyOrder,

    #[error("Cliente {0} não encontrado")]
    CustomerNotFound(Uuid),

    #[error("Estoque insuficiente para produto {product_id}")]
    InsufficientStock { product_id: Uuid },
}

// use case — retorna Result, não faz panic
pub async fn execute(&self, input: CreateOrderInput) -> Result<OrderOutput, DomainError> {
    if input.items.is_empty() {
        return Err(DomainError::EmptyOrder);
    }

    let order = Order::new(input.customer_id, input.items);
    self.repository.save(&order).await?;

    Ok(OrderOutput::from(order))
}

// handler — pattern matching exaustivo
async fn create_order(
    State(state): State<AppState>,
    ValidatedJson(input): ValidatedJson<CreateOrderInput>,
) -> impl IntoResponse {
    match state.create_order.execute(input).await {
        Ok(output) => (StatusCode::CREATED, Json(output)).into_response(),
        Err(DomainError::EmptyOrder) => {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": "Pedido sem itens"}))).into_response()
        }
        Err(e) => {
            tracing::error!(?e, "unexpected error");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

**O compilador não deixa você esquecer de tratar um erro.** Diferente de `try/catch` onde o esquecimento é silencioso, em Rust é erro de compilação.

---

## Ownership — por que isso importa para APIs

```rust
// Connection pool é Arc<Pool> — compartilhado entre tasks sem data race
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,        // Arc internamente — thread-safe
    pub redis: RedisPool,
    pub create_order: Arc<CreateOrderUseCase>,
}

// Cada request é uma task Tokio — concorrência massiva sem GC
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        db: pool.clone(),
        create_order: Arc::new(CreateOrderUseCase::new(
            PgOrderRepository::new(pool.clone()),
        )),
        // ...
    };

    let app = Router::new()
        .route("/orders", post(create_order).get(list_orders))
        .layer(SecurityHeadersLayer)
        .layer(RateLimitLayer::new(100, Duration::from_secs(60)))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

---

## SQLx — queries verificadas em compile time

```rust
// O compilador verifica a query contra o banco DURANTE a compilação
// Se a coluna não existe ou o tipo está errado, não compila
pub async fn find_by_public_id(&self, public_id: Uuid) -> Result<Option<Order>, sqlx::Error> {
    sqlx::query_as!(
        OrderRow,
        r#"
        SELECT id, public_id, customer_id, status, created_at, updated_at
        FROM orders
        WHERE public_id = $1 AND deleted_at IS NULL
        "#,
        public_id
    )
    .fetch_optional(&self.pool)
    .await
    .map(|row| row.map(Into::into))
}
```

**Tipo errado? Coluna renomeada? Erro de compilação, não erro em produção às 3h da manhã.**

---

## Observabilidade — tracing nativo

```rust
// tracing — structured logging com spans
#[tracing::instrument(skip(self), fields(order_id))]
pub async fn execute(&self, input: CreateOrderInput) -> Result<OrderOutput, DomainError> {
    let order = Order::new(input.customer_id, input.items);
    tracing::Span::current().record("order_id", order.public_id.to_string());

    tracing::info!(
        customer_id = %input.customer_id,
        item_count = input.items.len(),
        "creating order"
    );

    self.repository.save(&order).await?;

    tracing::info!("order created successfully");
    Ok(OrderOutput::from(order))
}
```

---

## Docker — binário estático de 15MB

```dockerfile
# Multi-stage build — imagem final sem compilador
FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/api /api
EXPOSE 3000
ENTRYPOINT ["/api"]
```

**Imagem final: ~20MB.** Sem runtime, sem interpretador, sem GC. Cold start em Lambda/Cloud Run: <100ms.

---

## Testes

```bash
cargo test                          # Todos
cargo test --lib                    # Só unit tests
cargo test -- --test-threads=1      # Integração (sequencial)
cargo llvm-cov                      # Coverage com llvm-cov
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_order_returns_error() {
        let repo = MockOrderRepository::new();
        let use_case = CreateOrderUseCase::new(Arc::new(repo));

        let input = CreateOrderInput {
            customer_id: Uuid::new_v4(),
            items: vec![],
        };

        let result = tokio_test::block_on(use_case.execute(input));

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::EmptyOrder));
    }
}
```

---

## Architecture Decision Records (ADRs)

Decisões arquiteturais documentadas com contexto, motivo e trade-offs:

- [ADR-001: INT/BIGINT PK + UUID público](docs/adrs/001-bigint-pk-uuid-public.md)
- [ADR-002: Result Pattern vs Exceptions](docs/adrs/002-result-pattern-over-exceptions.md)
- [ADR-003: Clean Architecture com limites pragmáticos](docs/adrs/003-clean-architecture-boundaries.md)
- [ADR-004: Princípios de Modelagem de Dados](docs/adrs/004-database-design-principles.md)
- [ADR-005: Partitioning para Tabelas de Alto Volume](docs/adrs/005-table-partitioning.md)
- [ADR-006: Connection Pooling e Pool Sizing](docs/adrs/006-connection-pooling.md)
- [ADR-007: VACUUM, Autovacuum e Bloat Prevention](docs/adrs/007-vacuum-autovacuum.md)
- [ADR-008: Read Replicas e Separação de Leitura/Escrita](docs/adrs/008-read-replicas.md)
- [ADR-009: Observabilidade e Testes de Carga](docs/adrs/009-observability-load-testing.md)
- [ADR-010: LLMOps, Agentes e Resiliência Extrema (Top 0.01%)](docs/adrs/010-top-tier-engineering.md)

---

## Referência técnica

- [Guia de Backend & Arquitetura](https://czanix.com/pt/stack/backend)
- [Catálogo de Trade-offs](https://czanix.com/pt/stack/tradeoffs)
- [Tech Radar — Tecnologias Emergentes](https://czanix.com/pt/stack/tech-radar)
- [DevOps & CI/CD](https://czanix.com/pt/stack/devops)

---

## Licença

MIT — use, adapte, melhore. Se ajudou, [deixa uma estrela](https://github.com/czanix/boilerplate-api-rust) ⭐

---

<div align="center">
<sub>Desenvolvido e mantido por <a href="https://czanix.com">Cesar Zanis</a> — Czanix</sub>
</div>
