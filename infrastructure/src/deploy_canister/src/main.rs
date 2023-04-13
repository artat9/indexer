use candid::{CandidType, Decode, Encode, Nat};
use ic_agent::{
    export::Principal,
    identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity},
    Agent, Identity,
};
use ring;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::Deserialize;
#[derive(CandidType)]
struct Argument {
    amount: Option<Nat>,
}

#[derive(CandidType, Deserialize)]
struct CreateCanisterResult {
    canister_id: Principal,
}
/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(_event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn create_a_canister() -> Result<Principal, Box<dyn std::error::Error>> {
    let url = format!("http://52.90.145.132:36875");
    let rng = ring::rand::SystemRandom::new();
    let key_pair = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .expect("Could not generate a key pair.");
    let identity = BasicIdentity::from_key_pair(
        ring::signature::Ed25519KeyPair::from_pkcs8(key_pair.as_ref())
            .expect("Could not read the key pair."),
    );
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(identity)
        .build()?;
    // Only do the following call when not contacting the IC main net (e.g. a local emulator).
    // This is important as the main net public key is static and a rogue network could return
    // a different key.
    // If you know the root key ahead of time, you can use `agent.set_root_key(root_key)?;`.
    agent.fetch_root_key().await?;
    let management_canister_id = Principal::from_text("aaaaa-aa")?;
    // Create a call to the management canister to create a new canister ID,
    // and wait for a result.
    // The effective canister id must belong to the canister ranges of the subnet at which the canister is created.
    let effective_canister_id = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

    let response = agent
        .update(
            &management_canister_id,
            "provisional_create_canister_with_cycles",
        )
        .with_effective_canister_id(effective_canister_id)
        .with_arg(&Encode!(&Argument { amount: None })?)
        .call_and_wait()
        .await?;
    let result = Decode!(response.as_slice(), CreateCanisterResult)?;
    let canister_id: Principal = result.canister_id;
    Ok(canister_id)
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    create_a_canister().await;
    Ok(())
    //tracing_subscriber::fmt()
    //    .with_max_level(tracing::Level::INFO)
    //    // disable printing the name of the module in every log line.
    //    .with_target(false)
    //    // disabling time is handy because CloudWatch will add the ingestion time.
    //    .without_time()
    //    .init();
    //
    //run(service_fn(function_handler)).await
}
