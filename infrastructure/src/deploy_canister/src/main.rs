use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use std::io::{self, Write};
use std::process::Command;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    execute().await?;
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .header("Access-Control-Allow-Origin", "*")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn execute() -> Result<String, Error> {
    let out = Command::new("dfx")
        .current_dir("/indexer")
        .args(&[
            "canister",
            "create",
            "--all",
            "--network",
            "http://52.90.145.132:36875",
        ])
        .output()
        .map_err(|e| e.to_string())?;
    io::stdout().write_all(&out.stdout).unwrap();
    println!("status: {}", out.status);
    let out = Command::new("rm")
        .current_dir("/indexer/.dfx")
        .args(&["-rf", "http___52_90_145_132_36875"])
        .output()
        .map_err(|e| e.to_string())?;
    println!("status: {}", out.status);
    Command::new("rm")
        .current_dir("/indexer")
        .args(&["canister", "install", "--all"])
        .output()
        .map_err(|e| e.to_string())?;
    return Ok(("OK").to_string());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    //tracing_subscriber::fmt()
    //    .with_max_level(tracing::Level::INFO)
    //    // disable printing the name of the module in every log line.
    //    .with_target(false)
    //    // disabling time is handy because CloudWatch will add the ingestion time.
    //    .without_time()
    //    .init();
    //
    //run(service_fn(function_handler)).await
    execute().await?;
    Ok(())
}
