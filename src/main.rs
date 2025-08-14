use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use lambda_web::{is_running_on_lambda, LambdaError};
use serde_json::Value;
use tracing::info;

mod api;
mod database;
mod engine;
mod errors;
mod handlers;
mod types;

use api::router::create_router;

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    info!("Received event: {}", serde_json::to_string_pretty(&event.payload)?);
    
    let router = create_router().await?;
    
    // Convert Lambda event to HTTP request and process through router
    let response = router.call(event).await?;
    
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    if is_running_on_lambda() {
        // Running on AWS Lambda
        info!("Starting Pokemon Adventure API on AWS Lambda");
        run(service_fn(function_handler)).await
    } else {
        // Running locally for development/testing
        info!("Starting Pokemon Adventure API locally");
        run(service_fn(function_handler)).await
    }
}