use anchor_client::{solana_client::rpc_client::RpcClient, solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
    signer::Signer, system_program,
}, Client, ClientError, Cluster, DynSigner, Program};

use solana_smart_contract::{instruction as ix};
use solana_smart_contract::accounts as accounts;
use std::{rc::Rc};

use bls12_381::{pairing, G1Affine, G2Affine};
use serde::{Deserialize, Serialize};
use serde::de::{Deserializer, Error as DeError};
use serde::ser::Serializer;
use solana_sdk::instruction::Instruction;
use warp::{Filter, Rejection, Reply, http::StatusCode, reject::Reject, reject};
use warp::reply::Json;
use std::{sync::Arc};
use anchor_lang::prelude::Pubkey;
use solana_sdk::signature::Signature;
use warp::hyper::body::HttpBody;

#[derive(Debug)]
struct HexArray<const N: usize>([u8; N]);

impl<const N: usize> Serialize for HexArray<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de, const N: usize> Deserialize<'de> for HexArray<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(DeError::custom)?;
        if bytes.len() != N {
            return Err(DeError::custom(format!(
                "Invalid length: expected {} bytes, got {} bytes",
                N,
                bytes.len()
            )));
        }
        let mut array = [0u8; N];
        array.copy_from_slice(&bytes);
        Ok(HexArray(array))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestPayload {
    #[serde(with = "hex_array_96")]
    g_compressed: [u8; 96],
    #[serde(with = "hex_array_48")]
    sigma_compressed: [u8; 48],
    #[serde(with = "hex_array_96")]
    v_compressed: [u8; 96],
    #[serde(with = "hex_array_48")]
    multiplication_sum_compressed: [u8; 48],
}

mod hex_array_96 {
    use serde::{Deserialize, Serialize};
    use super::HexArray;

    pub fn serialize<S>(value: &[u8; 96], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        HexArray(*value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 96], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let array = HexArray::<96>::deserialize(deserializer)?;
        Ok(array.0)
    }
}

mod hex_array_64 {
    use serde::{Deserialize, Serialize};
    use super::HexArray;

    pub fn serialize<S>(value: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        HexArray(*value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let array = HexArray::<64>::deserialize(deserializer)?;
        Ok(array.0)
    }
}

mod hex_array_48 {
    use serde::{Deserialize, Serialize};
    use super::HexArray;

    pub fn serialize<S>(value: &[u8; 48], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        HexArray(*value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 48], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let array = HexArray::<48>::deserialize(deserializer)?;
        Ok(array.0)
    }
}

impl RequestPayload {
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Custom rejection type for errors
#[derive(Debug)]
struct ClientRejection(String);

impl Reject for ClientRejection {}

/// Handles errors and converts them into proper JSON responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if let Some(ClientRejection(client_err)) = err.find::<ClientRejection>() {
        let json = warp::reply::json(&format!("Client error: {:?}", client_err));
        return Ok(warp::reply::with_status(json, StatusCode::BAD_REQUEST));
    }

    // Handle other errors
    Ok(warp::reply::with_status(
        warp::reply::json(&"Internal Server Error"),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

// Function to generate and print Keypairs
fn generate_keypairs() -> (Arc<Keypair>, Arc<Keypair>, Arc<Keypair>, Arc<Keypair>) {
    let payer = Arc::new(Keypair::new());
    let server = Arc::new(Keypair::new());
    let mediator = Arc::new(Keypair::new());
    let program_context = Arc::new(Keypair::new());

    println!("Generated Keypairs:");
    println!("   Payer: {}", payer.pubkey());
    println!("   Server: {}", server.pubkey());
    println!("   Mediator: {}", mediator.pubkey());
    println!("   Program Context: {}", program_context.pubkey());

    (payer, server, mediator, program_context)
}

// Function to request airdrop for a given public key
async fn request_airdrop(connection: &RpcClient, payer_pubkey: &Pubkey, amount: u64) -> Signature {
    println!("\nRequesting {} SOL airdrop to payer", amount);
    let airdrop_signature = connection.request_airdrop(payer_pubkey, amount).unwrap();
    airdrop_signature
}

// Function to confirm transaction completion
async fn confirm_airdrop(connection: &RpcClient, airdrop_signature: &Signature) {
    while !connection.confirm_transaction(airdrop_signature).unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    println!("   Airdrop confirmed!");
}

// Function to create a program client
fn create_program_client(payer: Arc<Keypair>) -> Client<Arc<Keypair>> {
    Client::new_with_options(
        Cluster::Localnet,
        Arc::clone(&payer),
        CommitmentConfig::confirmed(),
    )
}

async fn initialize_instruction(program: &Program<Rc<Keypair>>, mediator: Rc<Keypair>, program_context: Rc<Keypair>) -> Result<Instruction, ClientError> {
    let initialize_ix = program
        .request()
        .accounts(accounts::Initialize {
            payer: program.payer(),
            mediator: mediator.pubkey(),
            program_context: program_context.pubkey(),
            system_program: system_program::ID,
        })
        .args(ix::Initialize)
        .instructions()?
        .remove(0);

    Ok(initialize_ix)
}

async fn set_client_curve_points_instruction(program: &Program<Rc<Keypair>>, mediator: Rc<Keypair>, program_context: Rc<Keypair>) -> Result<Instruction, ClientError> {
    let g_norm: [u8; 96] = [1; 96]; // Example array, replace with actual data
    let v_norm: [u8; 96] = [1; 96]; // Example array, replace with actual data

    let set_client_curve_points_ix = program
        .request()
        .accounts(accounts::SetClientCurvePoints {
            payer: program.payer(),
            mediator: mediator.pubkey(),
            program_context: program_context.pubkey(),
            system_program: system_program::ID,
        })
        .args(ix::SetClientCurvePoints {
            g_norm,
            v_norm,
        })
        .instructions()?
        .remove(0);

    Ok(set_client_curve_points_ix)
}

async fn end_subscription_instruction(program: &Program<Rc<Keypair>>, program_context: Rc<Keypair>) -> Result<Instruction, ClientError> {
    let end_subscription_ix = program
        .request()
        .accounts(accounts::EndSubscription {
            program_context: program_context.pubkey(),
        })
        .args(ix::EndSubscription)
        .instructions()?
        .remove(0);

    Ok(end_subscription_ix)
}

// Main function to send all instructions
async fn send_instructions(program: &Program<Rc<Keypair>>, mediator: Rc<Keypair>, program_context: Rc<Keypair>) -> Result<Signature, ClientError> {
    // Build the instructions
    let initialize_ix = initialize_instruction(program, mediator.clone(), program_context.clone()).await?;
    let set_client_curve_points_ix = set_client_curve_points_instruction(program, mediator.clone(), program_context.clone()).await?;
    let end_subscription_ix = end_subscription_instruction(program, program_context.clone()).await?;

    // Send the transaction with the instructions
    let signature = program
        .request()
        .instruction(initialize_ix)
        .instruction(set_client_curve_points_ix)
        .instruction(end_subscription_ix)
        .signer(&mediator)
        .signer(&program_context)
        .send()
        .await?;

    println!("   Transaction confirmed: {}", signature);
    Ok(signature)
}

// Function to fetch and print account data
async fn fetch_account_data(program: &Program<Rc<Keypair>>, program_context: Rc<Keypair>, mediator: Rc<Keypair>) {
    let program_context_account: solana_smart_contract::ProgramContext = program.account::<solana_smart_contract::ProgramContext>(program_context.pubkey()).await.unwrap();
    let mediator_account: solana_smart_contract::Mediator = program.account::<solana_smart_contract::Mediator>(mediator.pubkey()).await.unwrap();

    println!("   Counter value: {}", program_context_account.mediator_balance);
    println!("   Counter value: {}", mediator_account.balance);
}

async fn initialize_instruction_endpoint(program: &Program<Arc<Keypair>>, payer: Arc<Keypair>, mediator: Arc<Keypair>, program_context: Arc<Keypair>) -> Result<&str, Rejection> {
    let initialize_ix = program
        .request()
        .accounts(accounts::Initialize {
            payer: payer.pubkey(),
            mediator: mediator.pubkey(),
            program_context: program_context.pubkey(),
            system_program: system_program::ID,
        })
        .args(ix::Initialize)
        .instructions()
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?
        .remove(0);

    let signature = program
        .request()
        .instruction(initialize_ix)
        .signer(&payer)
        .signer(&mediator)
        .signer(&program_context)
        .send()
        .await
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;

    println!("   Initialize Instruction Confirmed: {}", signature);
    Ok("signature")
}

// Endpoint for set client curve points instruction
async fn set_client_curve_points_instruction_endpoint(program: &Program<Arc<Keypair>>, payer: Arc<Keypair>, mediator: Arc<Keypair>, program_context: Arc<Keypair>) -> Result<Signature, Rejection> {
    let g_norm: [u8; 96] = [1; 96]; // Example array, replace with actual data
    let v_norm: [u8; 96] = [1; 96]; // Example array, replace with actual data

    let set_client_curve_points_ix = program
        .request()
        .accounts(accounts::SetClientCurvePoints {
            payer: payer.pubkey(),
            mediator: mediator.pubkey(),
            program_context: program_context.pubkey(),
            system_program: system_program::ID,
        })
        .args(ix::SetClientCurvePoints {
            g_norm,
            v_norm,
        })
        .instructions()
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?
        .remove(0);

    let signature = program
        .request()
        .instruction(set_client_curve_points_ix)
        .signer(&payer)
        .signer(&mediator)
        .signer(&program_context)
        .send()
        .await
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;

    println!("   Set Client Curve Points Instruction Confirmed: {}", signature);
    Ok(signature)
}

// Endpoint for set client curve points instruction
async fn end_subscription_instruction_endpoint(program: &Program<Arc<Keypair>>, payer: Arc<Keypair>, mediator: Arc<Keypair>, program_context: Arc<Keypair>) -> Result<Signature, Rejection> {
    let end_subscription_ix = program
        .request()
        .accounts(accounts::EndSubscription {
            program_context: program_context.pubkey(),
        })
        .args(ix::EndSubscription)
        .instructions()
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?
        .remove(0);

    let signature = program
        .request()
        .instruction(end_subscription_ix)
        .signer(&payer)
        .signer(&mediator)
        .signer(&program_context)
        .send()
        .await
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;

    println!("   Set Client Curve Points Instruction Confirmed: {}", signature);
    Ok(signature)
}

// #[tokio::main]
// async fn main() -> Result<(), ClientError> { // Corrected return type
//     // Initialize connection and keypairs
//     let connection = RpcClient::new_with_commitment("http://127.0.0.1:8899", CommitmentConfig::confirmed());
//     let (payer, server, mediator, program_context) = generate_keypairs();
//
//     // Request airdrop to payer
//     let airdrop_signature = request_airdrop(&connection, &payer.pubkey(), 10 * LAMPORTS_PER_SOL).await;
//     confirm_airdrop(&connection, &airdrop_signature).await;
//
//     // Create program client
//     let provider = create_program_client(Arc::clone(&payer));
//     let program = provider.program(solana_smart_contract::ID)?;
//
//     initialize_instruction_endpoint(&program, payer, mediator, program_context).await;
//
//     // let provider2 = create_program_client(Rc::clone(&payer));
//     // let program2 = provider2.program(solana_smart_contract::ID)?;
//     // set_client_curve_points_instruction_endpoint(&program2, Rc::clone(&payer), Rc::clone(&mediator), Rc::clone(&program_context)).await;
//
//     // Send transaction instructions
//     // let _signature = send_instructions(&program, Rc::clone(&mediator), Rc::clone(&program_context)).await;
//
//     // Fetch and display account data
//     // fetch_account_data(&program, Rc::clone(&program_context), Rc::clone(&mediator)).await;
//
//     // Request airdrop to server
//     request_airdrop(&connection, &server.pubkey(), 10 * LAMPORTS_PER_SOL).await;
//
//     Ok(())
// }



// Define a struct to handle the incoming request body (amount of SOL)
#[derive(Serialize, Deserialize, Debug)]
struct AirdropRequest {
    #[serde(with = "hex_array_64")]
    payer_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    amount_sol: u64,  // Amount of SOL to airdrop
}

// Define a struct for the response, if needed
#[derive(Serialize)]
struct AirdropResponse {
    message: String
}

async fn airdrop_handler(payload: AirdropRequest) -> Result<Json, Rejection> {
    let payer = deserialize_keypair_from_bytes(&payload.payer_keypair_as_hex);
    println!("Serialized keypair bytes: {:?}", payer.pubkey());

    println!("\nRequesting {} SOL airdrop to payer", payload.amount_sol);

    let connection = RpcClient::new_with_commitment("http://127.0.0.1:8899", CommitmentConfig::confirmed());

    // Request airdrop to payer
    let airdrop_signature = request_airdrop(&connection, &payer.pubkey(), payload.amount_sol * LAMPORTS_PER_SOL).await;
    confirm_airdrop(&connection, &airdrop_signature).await;

    println!("   Airdrop confirmed!");

    // Return a JSON response with the key and message
    Ok(warp::reply::json(&AirdropResponse {
        message: format!("Airdrop of {} SOL confirmed", payload.amount_sol),
    }))
}

// Define a struct to handle the incoming request body (amount of SOL)
#[derive(Serialize, Deserialize, Debug)]
struct InitializeSubscriptionRequest {
    #[serde(with = "hex_array_64")]
    payer_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    mediator_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    program_context_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
}

// Define a struct for the response, if needed
#[derive(Serialize)]
struct InitializeSubscriptionResponse {
    message: String
}

async fn initialize_subscription_handler(payload: InitializeSubscriptionRequest) -> Result<Json, Rejection> {
    let payer = Arc::new(deserialize_keypair_from_bytes(&payload.payer_keypair_as_hex));
    let mediator = Arc::new(deserialize_keypair_from_bytes(&payload.mediator_keypair_as_hex));
    let program_context = Arc::new(deserialize_keypair_from_bytes(&payload.program_context_keypair_as_hex));

    let payer_clone = Arc::clone(&payer);

    // Create program client
    let provider = create_program_client(payer);

    let program = provider
        .program(solana_smart_contract::ID)
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;  // Convert ClientError to warp::Rejection

    initialize_instruction_endpoint(&program, payer_clone, mediator, program_context).await;

    // Return a JSON response with the key and message
    Ok(warp::reply::json(&AirdropResponse {
        message: format!("Airdrop of {} SOL confirmed", 5),
    }))
}

// Define a struct to handle the incoming request body (amount of SOL)
#[derive(Serialize, Deserialize, Debug)]
struct SetClientCurvePointsRequest {
    #[serde(with = "hex_array_64")]
    payer_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    mediator_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    program_context_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
}

// Define a struct for the response, if needed
#[derive(Serialize)]
struct SetClientCurvePointsResponse {
    message: String
}

async fn set_client_curve_points_handler(payload: SetClientCurvePointsRequest) -> Result<Json, Rejection> {
    let payer = Arc::new(deserialize_keypair_from_bytes(&payload.payer_keypair_as_hex));
    let mediator = Arc::new(deserialize_keypair_from_bytes(&payload.mediator_keypair_as_hex));
    let program_context = Arc::new(deserialize_keypair_from_bytes(&payload.program_context_keypair_as_hex));

    let payer_clone = Arc::clone(&payer);

    // Create program client
    let provider = create_program_client(payer);

    let program = provider
        .program(solana_smart_contract::ID)
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;  // Convert ClientError to warp::Rejection

    set_client_curve_points_instruction_endpoint(&program, payer_clone, mediator, program_context).await;

    // Return a JSON response with the key and message
    Ok(warp::reply::json(&SetClientCurvePointsResponse {
        message: format!("Airdrop of {} SOL confirmed", 5),
    }))
}

// Define a struct to handle the incoming request body (amount of SOL)
#[derive(Serialize, Deserialize, Debug)]
struct EndSubscriptionRequest {
    #[serde(with = "hex_array_64")]
    payer_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    mediator_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
    #[serde(with = "hex_array_64")]
    program_context_keypair_as_hex: [u8; 64], // Serialized keypair (private + public key)
}

// Define a struct for the response, if needed
#[derive(Serialize)]
struct EndSubscriptionResponse {
    message: String
}

async fn end_subscription_handler(payload: EndSubscriptionRequest) -> Result<Json, Rejection> {
    let payer = Arc::new(deserialize_keypair_from_bytes(&payload.payer_keypair_as_hex));
    let mediator = Arc::new(deserialize_keypair_from_bytes(&payload.mediator_keypair_as_hex));
    let program_context = Arc::new(deserialize_keypair_from_bytes(&payload.program_context_keypair_as_hex));

    let payer_clone = Arc::clone(&payer);

    // Create program client
    let provider = create_program_client(payer);

    let program = provider
        .program(solana_smart_contract::ID)
        .map_err(|err| reject::custom(ClientRejection(err.to_string())))?;  // Convert ClientError to warp::Rejection

    set_client_curve_points_instruction_endpoint(&program, payer_clone, mediator, program_context).await;

    // Return a JSON response with the key and message
    Ok(warp::reply::json(&EndSubscriptionResponse {
        message: format!("Airdrop of {} SOL confirmed", 5),
    }))
}

/// Serializes the `Keypair` to bytes (secret + public key).
fn serialize_keypair(keypair: &Keypair) -> String {
    hex::encode(keypair.to_bytes())
}

/// Deserializes the bytes back into a `Keypair`.
fn deserialize_keypair_from_hex(hex_keypair: &str) -> Keypair {
    let keypair_bytes = hex::decode(hex_keypair).expect("Invalid hex");
    Keypair::from_bytes(&keypair_bytes).expect("Invalid keypair bytes")
}

/// Deserializes the bytes back into a `Keypair`.
fn deserialize_keypair_from_bytes(bytes: &[u8]) -> Keypair {
    Keypair::from_bytes(bytes).expect("Failed to recover keypair from bytes")
}

pub const PAYER_PUBKEY: &str = "dcf858909bc5bcbefc152eaf0561eb1d6e6c223328f0e06d8ce855f987a07e327bc14a0545e1a150fdf695cf889c482569f0ff42b30fe63b037cf8679c7c7a09";
pub const SERVER_PUBKEY: &str = "356f41cbba468c243804af4ef2cfdb127b6212cf4ac9de552c5a74fdf6c218164ce9382dc9cf36ff00009e7ced94a2a15a34e2908b431e7b998553fa67d1184e";
pub const MEDIATOR_PUBKEY: &str = "91b298f138a41ab2b641a82e9c00c1505ecda791256f93e32beb814b980a070d9b20c29e5ed265c30606f226ccd53af629af477726504ef5d6bd2756715d96dd";
pub const PROGRAM_CONTEXT_PUBKEY: &str = "9abcabbe576bc4c95fb7338bc941c79d89cd1163599d648f3374a81615277917d43d3c2fce05680bfe423fe37f35b9b2e6b6bf4cc08e3bd66f5c09ae9f649118";

#[tokio::main]
async fn main() {
    let (payer, server, mediator, program_context) = generate_keypairs();

    let verify = warp::path("verify")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: RequestPayload| {
            if let Err(e) = body.validate() {
                return warp::reply::json(&format!("Validation error: {}", e));
            }

            let g_norm = G2Affine::from_compressed(&body.g_compressed).unwrap();
            let σ_norm = G1Affine::from_compressed(&body.sigma_compressed).unwrap();
            let v_norm = G2Affine::from_compressed(&body.v_compressed).unwrap();
            let multiplication_sum_norm = G1Affine::from_compressed(&body.multiplication_sum_compressed).unwrap();

            let left_pairing = pairing(&σ_norm, &g_norm);
            let right_pairing = pairing(&multiplication_sum_norm, &v_norm);

            let is_verified = left_pairing.eq(&right_pairing);
            println!("{}", is_verified);

            warp::reply::json(&if is_verified { "Verified" } else { "Not Verified" })
        });

    let airdrop = warp::path("airdrop")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(airdrop_handler);

    let initialize_subscription = warp::path("initializeSubscription")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(initialize_subscription_handler);

    // let set_client_curve_points = warp::path("setClientCurvePoints")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and_then(set_client_curve_points_handler);

    // let extend_subscription = warp::path("extendSubscription")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and_then(extend_subscription_handler);

    // let end_subscription = warp::path("endSubscription")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .and_then(end_subscription_handler);

    let routes = verify
        .or(airdrop)
        .or(initialize_subscription);
        // .or(set_client_curve_points)
        // // .or(extend_subscription)
        // .or(end_subscription);

    println!("Server running at http://127.0.0.1:3030/");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
