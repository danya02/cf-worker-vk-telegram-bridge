pub mod anyhow_result;
mod vk_side;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_service::Service;
use worker::*;

#[derive(Clone)]
pub struct AppState {
    env: Env,
    //ctx: Arc<Context>,
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/vk", post(vk_side::handle_request))
        .with_state(state)
}

// Multiple calls to `init` will cause a panic as a tracing subscriber is already set.
// So we use the `start` event to initialize our tracing subscriber when the worker starts.
#[event(start)]
fn start() {
    use tracing_subscriber::fmt::format::Pretty;
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::prelude::*;
    use tracing_web::{performance_layer, MakeConsoleWriter};
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(AppState {
        env,
        //ctx: Arc::new(ctx),
    })
    .call(req)
    .await?)
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}
