// Integration tests for Iron Shield
// This is a placeholder for Playwright integration tests
// Actual Playwright tests would go here when the API is properly integrated

use serde_json::json;
use std::fs;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::net::TcpStream;

// Helper function to start the Iron Shield server
fn start_server() -> Child {
    Command::new("cargo")
        .args(["run", "--", "3001"]) // Use port 3001 for testing to avoid conflicts
        .spawn()
        .expect("Failed to start Iron Shield server")
}

// Helper to wait for the server to be ready
async fn wait_for_server_ready() {
    for _ in 0..30 {
        if TcpStream::connect("127.0.0.1:3001").await.is_ok() {
            tokio::time::sleep(Duration::from_millis(500)).await;
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Server did not start within 30 seconds");
}

#[tokio::test]
async fn test_server_startup() {
    // Start the server in a separate process
    let mut server_process = start_server();

    // Wait for the server to be ready
    wait_for_server_ready().await;

    // Test that the server responds to HTTP requests
    let response = reqwest::get("http://localhost:3001").await;
    assert!(response.is_ok());
    let _status = response.unwrap().status();
    // Terminate the server process
    server_process.kill().unwrap();
    server_process.wait().unwrap();
}

#[tokio::test]
async fn test_config_loading() {
    // Create a temporary config file for testing
    let test_config = json!({
        "site_name": "Test Dashboard",
        "clock": "None",
        "sites": [
            {
                "name": "Google",
                "url": "https://www.google.com",
                "tags": ["search", "popular"]
            },
            {
                "name": "GitHub",
                "url": "https://www.github.com",
                "tags": ["development", "code"]
            }
        ]
    });

    fs::write("config.json5", format!("{test_config}")).unwrap();

    // Start the server in a separate process
    let mut server_process = start_server();

    // Wait for the server to be ready
    wait_for_server_ready().await;

    // Test that the server responds with the custom config
    let response = reqwest::get("http://localhost:3001").await;
    assert!(response.is_ok());
    let text = response.unwrap().text().await.unwrap();
    assert!(text.contains("Test Dashboard"));
    assert!(text.contains("Google"));
    assert!(text.contains("GitHub"));

    // Terminate the server process
    server_process.kill().unwrap();
    server_process.wait().unwrap();

    // Clean up test config file
    fs::remove_file("config.json5").unwrap();
}

// Additional test demonstrating how Playwright tests would work when available
#[tokio::test]
async fn playwright_test_template() {
    // This is a placeholder to show how a Playwright test would be structured
    // when the Rust Playwright API is fully functional and integrated

    // The test would:
    // 1. Start the Iron Shield server
    // 2. Launch a browser using Playwright
    // 3. Navigate to the application
    // 4. Wait for 1 minute for sse events
    // 5. See the uptime for Google should be 100%
    // 6. Close the browser and terminate the server

    // For now, we just verify that the test compiles
}
