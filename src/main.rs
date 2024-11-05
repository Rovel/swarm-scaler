use reqwest::{Client, Certificate, Identity};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Proposal { node_id: String },
    Confirmation { node_id: String },
    ScaleComplete,
}

#[derive(Debug, Deserialize)]
struct Node {
    ID: String,
    Status: NodeStatus,
    Spec: NodeSpec,
}

#[derive(Debug, Deserialize)]
struct NodeStatus {
    State: String,
    Addr: String,
}

#[derive(Debug, Deserialize)]
struct NodeSpec {
    Role: String,
}

async fn get_secure_client() -> Result<Client, Box<dyn std::error::Error>> {
    // Load CA certificate
    let ca_cert = fs::read("/tls/ca.pem")?;
    let ca = Certificate::from_pem(&ca_cert)?;

    // Load client certificate and key
    let client_cert = fs::read("/tls/client-cert.pem")?;
    let client_key = fs::read("/tls/client-key.pem")?;
    let identity = Identity::from_pem(&[client_cert, client_key].concat())?;

    // Configure reqwest client with TLS and identity
    let client = Client::builder()
        .add_root_certificate(ca)
        .identity(identity)
        .build()?;

    Ok(client)
}

async fn get_manager_addresses(client: &Client) -> Result<Vec<SocketAddr>, reqwest::Error> {
    let url = "https://localhost:2376/nodes";
    let response = client.get(url).send().await?;

    let nodes: Vec<Node> = response.json().await?;
    let manager_addresses = nodes
        .into_iter()
        .filter(|node| node.Spec.Role == "manager" && node.Status.State == "ready")
        .filter_map(|node| node.Status.Addr.parse::<SocketAddr>().ok())
        .collect::<Vec<_>>();

    Ok(manager_addresses)
}

async fn send_message(socket: &UdpSocket, addr: &SocketAddr, message: &Message) {
    let serialized = serde_json::to_string(message).unwrap();
    let _ = socket.send_to(serialized.as_bytes(), addr).await;
}

async fn receive_messages(socket: &UdpSocket) -> Option<Message> {
    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf).await {
        Ok((size, _)) => {
            let message: Message = serde_json::from_slice(&buf[..size]).unwrap();
            Some(message)
        }
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() {
    let client = get_secure_client().await.expect("Failed to create HTTPS client");

    let service_id = env::var("SERVICE_ID").expect("SERVICE_ID must be set");
    let cpu_threshold: u64 = env::var("CPU_THRESHOLD")
        .unwrap_or_else(|_| "800000000".to_string())
        .parse()
        .expect("CPU_THRESHOLD must be a number");

    // Local address for UDP communication
    let local_addr = env::var("LOCAL_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
    let socket = UdpSocket::bind(local_addr.clone()).await.expect("Failed to bind to address");

    loop {
        // Get list of manager nodes
        let manager_addresses = match get_manager_addresses(&client).await {
            Ok(addrs) => addrs,
            Err(err) => {
                eprintln!("Failed to retrieve manager addresses: {}", err);
                continue;
            }
        };

        // Send proposals and await confirmations
        if manager_addresses.len() > 1 {
            for addr in &manager_addresses {
                if addr != &local_addr.parse::<SocketAddr>().unwrap() {
                    let message = Message::Proposal { node_id: local_addr.clone() };
                    send_message(&socket, addr, &message).await;
                }
            }

            let mut confirmations = 1; // Start with 1 (self-confirmed)
            while confirmations < (manager_addresses.len() / 2 + 1) {
                if let Some(message) = receive_messages(&socket).await {
                    if let Message::Confirmation { node_id: _ } = message {
                        confirmations += 1;
                    }
                }
            }

            // If quorum is reached, proceed with scaling
            if confirmations >= (manager_addresses.len() / 2 + 1) {
                println!("Scaling decision reached and action taken.");

                // Notify peers of scaling completion
                for addr in &manager_addresses {
                    let message = Message::ScaleComplete;
                    send_message(&socket, addr, &message).await;
                }
            }
        }

        // Wait before checking metrics again
        sleep(Duration::from_secs(10)).await;
    }
}
