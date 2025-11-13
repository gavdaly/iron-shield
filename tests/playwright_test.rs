// Integration tests for Iron Shield
// This is a placeholder for Playwright integration tests
// Actual Playwright tests would go here when the API is properly integrated

use serde_json::json;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::net::TcpStream;

// Helper function to find an available port
fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to an available port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

// Helper function to start the Iron Shield server
fn start_server(port: u16, config_file: Option<&PathBuf>) -> Child {
    let mut command = Command::new("cargo");
    command.args(["run", "--", &port.to_string()]);

    if let Some(path) = config_file {
        command.arg(
            path.to_str()
                .expect("Failed to convert config file path to string"),
        );
    }

    command.spawn().expect("Failed to start Iron Shield server")
}

// Helper to wait for the server to be ready
async fn wait_for_server_ready(port: u16) {
    for _ in 0..30 {
        if TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .is_ok()
        {
            tokio::time::sleep(Duration::from_millis(500)).await;
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Server did not start within 30 seconds");
}

#[tokio::test]
async fn test_server_startup() {
    let port = find_available_port();
    // Start the server in a separate process
    let mut server_process = start_server(port, None);

    // Wait for the server to be ready
    wait_for_server_ready(port).await;

    // Test that the server responds to HTTP requests
    let response = reqwest::get(format!("http://localhost:{port}")).await;
    assert!(response.is_ok());
    let _status = response.expect("Failed to get HTTP response").status();
    // Terminate the server process
    server_process
        .kill()
        .expect("Failed to kill server process");
    server_process
        .wait()
        .expect("Failed to wait for server process to exit");
}

#[tokio::test]
async fn test_config_loading() {
    let port = find_available_port();
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

    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    let temp_config_path = temp_file.path().to_path_buf();
    fs::write(&temp_config_path, format!("{test_config}"))
        .expect("Failed to write temporary config file");

    let mut server_process = start_server(port, Some(&temp_config_path));

    // Wait for the server to be ready
    wait_for_server_ready(port).await;

    // Test that the server responds with the custom config
    let response = reqwest::get(format!("http://localhost:{port}")).await;
    assert!(response.is_ok());
    let text = response
        .expect("Failed to get HTTP response")
        .text()
        .await
        .expect("Failed to get response text");
    assert!(text.contains("Test Dashboard"));
    assert!(text.contains("Google"));
    assert!(text.contains("GitHub"));

    // Terminate the server process
    server_process
        .kill()
        .expect("Failed to kill server process in test_config_loading");
    server_process
        .wait()
        .expect("Failed to wait for server process to exit in test_config_loading");

    // temp_file will be automatically deleted when it goes out of scope
}

#[tokio::test]
#[cfg(target_arch = "x86_64")]
async fn integration_start_up() {
    use playwright::Playwright;

    let port = find_available_port();

    // 1. Create a temporary config file for testing
    let test_config = json!({
        "site_name": "Integration Test Dashboard",
        "clock": "None",
        "sites": [
            {
                "name": "Google",
                "url": "https://www.google.com",
                "tags": ["search", "popular"]
            }
        ]
    });

    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    let temp_config_path = temp_file.path().to_path_buf();
    fs::write(&temp_config_path, format!("{test_config}"))
        .expect("Failed to write temporary config file");

    // 2. Start the Iron Shield server
    let mut server_process = start_server(port, Some(&temp_config_path));

    // Wait for the server to be ready
    wait_for_server_ready(port).await;

    let location = format!("http://localhost:{}", port);

    let playwright = Playwright::initialize().await.expect("Have playwright");
    playwright.prepare().expect("To have browser");
    let chromium = playwright.chromium();
    let browser = chromium
        .launcher()
        .headless(true)
        .launch()
        .await
        .expect("to get browser");
    let context = browser
        .context_builder()
        .build()
        .await
        .expect("to get context");
    let page = context.new_page().await.expect("to create a page");

    page.goto_builder(&location)
        .goto()
        .await
        .expect("Failed to navigate to page in Playwright test");

    // 5. Wait for 1 minute for sse events and 6. See the uptime for Google should be 100%
    // This selector looks for a div that contains the text 'Google' and then within that div,
    // it looks for any element that contains the text '100%'.
    // This implicitly checks if JavaScript is running and SSE events have updated the UI.
    page.wait_for_selector_builder("div:has-text('Google') >> text='100%'")
        .timeout(60000.0) // 1 minute timeout
        .wait_for_selector()
        .await
        .expect("Google uptime did not reach 100% within 1 minute");

    // 7. Close the browser and terminate the server
    browser
        .close()
        .await
        .expect("Failed to close browser in Playwright test");
    server_process
        .kill()
        .expect("Failed to kill server process in final cleanup");
    server_process
        .wait()
        .expect("Failed to wait for server process to exit in final cleanup");

    // temp_file will be automatically deleted when it goes out of scope
}
