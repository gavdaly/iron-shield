use iron_shield::{
    config::{Config, Site},
    server::run,
};
use reqwest::StatusCode;
use std::{
    io::{self, Write},
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use tempfile::NamedTempFile;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

// Helper function to find an available port
async fn find_available_port() -> Option<u16> {
    use tokio::net::TcpListener;
    for port in 8000..9000 {
        match TcpListener::bind(SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port)).await {
            Ok(listener) => {
                return Some(
                    listener
                        .local_addr()
                        .expect("Failed to get local address of listener")
                        .port(),
                )
            }
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                eprintln!(
                    "Skipping server integration test because binding to {port} failed: {err}"
                );
                return None;
            }
            Err(_) => {}
        }
    }
    panic!("No available port found");
}

#[tokio::test]
async fn test_server_starts_and_serves_index() {
    // Create a temporary config file
    let mut config_file = NamedTempFile::new().expect("Failed to create temp config file");
    let test_config = Config {
        site_name: "Test Site".to_string(),
        clock: iron_shield::config::Clock::None,
        opentelemetry_endpoint: None,
        sites: vec![Site {
            name: "example.com".to_string(),
            url: "http://example.com".to_string(),
            category: "Test".to_string(),
            tags: vec!["test".to_string()],
            monitor_interval_secs: iron_shield::config::DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
            uptime_percentage: 0.0,
        }],
    };
    let config_content =
        serde_json::to_string_pretty(&test_config).expect("Failed to serialize test config");
    config_file
        .write_all(config_content.as_bytes())
        .expect("Failed to write to temp config file");
    let config_path = config_file.path().to_path_buf();

    let Some(port) = find_available_port().await else {
        return;
    };
    let server_address = format!("http://127.0.0.1:{port}");
    let cancel_token = CancellationToken::new();

    // Spawn the server in a background task
    let server_handle = tokio::spawn({
        let cancel_token = cancel_token.clone();
        async move {
            run(port, Some(config_path), cancel_token)
                .await
                .expect("Server failed to start");
        }
    });

    // Give the server a moment to start up
    sleep(Duration::from_secs(1)).await;

    // Make a request to the index page
    let client = reqwest::Client::new();
    let response = client
        .get(&server_address)
        .send()
        .await
        .expect("Failed to send request to server");

    assert_eq!(response.status(), StatusCode::OK);

    // Trigger graceful shutdown
    cancel_token.cancel();

    // Wait for the server to shut down
    server_handle.await.expect("Server task failed");
}
