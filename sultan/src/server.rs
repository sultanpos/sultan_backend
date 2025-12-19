use axum::{Json, Router, http, http::StatusCode, response::IntoResponse};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use sqlx::{Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::{fs::File, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::{config::AppConfig, web::AppState};
