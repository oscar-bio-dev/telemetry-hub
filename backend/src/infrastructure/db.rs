// Copyright (c) 2026 Oscar Mora / EcoWest Instrument. All rights reserved.
//
// This file is part of the oscar-bio-dev platform.
// Unauthorized copying of this file, via any medium, is strictly prohibited.
// Proprietary and confidential.

//! Conexión a `TimescaleDB` / `PostgreSQL`.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

/// Inicializa el pool de conexiones asíncronas a la base de datos.
///
/// # Errors
/// Retorna error si `DATABASE_URL` no está definida o si no puede conectar a la DB.
pub async fn init_db_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing::info!("Conectando a la base de datos...");

    let pool = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;

    Ok(pool)
}
