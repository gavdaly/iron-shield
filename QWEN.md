# Iron Shield Project Documentation

## Project Overview

Iron Shield is a Rust-based web application that serves as a customizable launcher & uptime monitor. The application provides a clean, minimal interface with integrated features like a digital clock and bookmark-style links to monitored websites.

The project is built using the Axum web framework and utilizes the Askama templating engine for server-side rendering. It follows a modular architecture with separate concerns for configuration, site management, and web server functionality.

## Architecture

The application follows a modular design with the following components:

### Core Modules

- `main.rs`: Application entry point that initializes and runs the server
- `server.rs`: Handles HTTP server setup and routing
- `index.rs`: Generates the main index page using template data
- `config.rs`: Manages application configuration and settings
- `uptime.rs`: Handles uptime monitoring functionality
- `error.rs`: Error handling and management
- `lib.rs`: Library module containing shared functionality

### Static Assets

- `static/`: Contains CSS and JavaScript files for frontend styling and interactivity
- `templates/`: Contains Askama templates for server-side rendering

### Dependencies

The project uses several key Rust crates:

- `axum`: Web framework for building the server
- `askama`: Template engine for HTML generation with Axum integration
- `askama_axum`: Axum integration for Askama templates
- `serde`: Serialization/deserialization support with derive features
- `serde_json`: JSON serialization/deserialization
- `tokio`: Asynchronous runtime with full features
- `tokio-stream`: Stream utilities for Tokio
- `tower-http`: HTTP service utilities with filesystem support
- `tracing`: Application tracing and logging
- `tracing-subscriber`: Tracing subscriber with environment filtering
- `json5`: JSON5 format support
- `reqwest`: HTTP client with JSON support
- `rand`: Random number generation
- `notify`: File system watching and notifications

## Features

### Clock Display

- Supports both standard and military time formats
- Updates in real-time using JavaScript
- Configurable via the application settings

### Site Bookmarks

- Organized list of bookmarked websites
- Each site includes a name, URL, icon, and tags
- Tags provide categorization for the sites

## Building and Running

To build and run the Iron Shield application:

1. **Prerequisites**:
   - Rust programming language (latest stable version)
   - Cargo package manager
   - Just command runner

2. **Build and Run**:

   ```bash
   just run
   ```

   This will compile the project and start the server.

3. **Access the Application**:
   - The application will be available at `http://0.0.0.0:3000`
   - The server will display a message confirming the launch address

4. **Development**:
   - For development, you can use `just check` to format, lint with clippy, and run tests
   - Use `just test` to run unit tests
   - Use `just test-integration` to run integration tests with Playwright
   - Use `just fmt` to format the code according to Rust standards
   - Use `just clippy` to run the linter with pedantic warnings

5. **Additional Just Commands**:
   - `just build`: Compile the project without running it
   - `just clean`: Remove build artifacts
   - `just check`: Run format, clippy, and test in sequence
   - `just test-integration`: Run integration tests specifically

## Integration Testing

The project includes integration tests to verify the application's behavior, with a framework in place for browser-based tests using Playwright.rs.

### Running Integration Tests

To run the integration tests:

```bash
just test-integration
```

### Current Integration Tests

The current integration tests include:

1. `test_server_startup` - Verifies that the server starts and responds to HTTP requests
2. `test_config_loading` - Verifies that custom configurations are loaded correctly
3. `playwright_test_template` - A template showing how Playwright tests would be structured

### Adding More Integration Tests

To add more integration tests:

1. Add new test functions to `tests/playwright_test.rs`
2. Use the existing test functions as a template
3. Remember to start the server and wait for it to be ready before testing
4. Always clean up resources (terminate server processes, remove temporary config files)
5. Consider using temporary configuration files that don't affect the main config

### Adding Playwright Tests

When the Playwright.rs API for Rust is fully ready for use, you can add browser automation tests by:

1. Adding the necessary Playwright dependency with appropriate features
2. Using the async API to launch browsers and interact with the UI
3. Performing UI assertions and interactions
4. Taking screenshots for visual regression testing if needed

### Important Notes

- Integration tests start the actual Iron Shield server on port 3001
- Tests perform HTTP requests to verify functionality
- The tests clean up after themselves, but make sure to terminate server processes
- Test configuration files are removed after tests to avoid conflicts

## Configuration

The application configuration is currently hardcoded in the `config.rs` file but follows a pattern that could support external configuration. The configuration includes:

- `site_name`: Name displayed in the page title
- `clock`: Time format (valid values are `TwelveHour` for standard 12-hour format with AM/PM, `TwentyFourHour` for military time format, or `NoClock` to display no clock). The default is `NoClock`.

## Development Conventions

### Code Structure

- Modules are organized by functionality rather than technical layer
- Each module handles a specific concern within the application
- Configuration is centralized to make changes easier

### Styling

- Minimal dark theme with #333 background and #ccc text
- Responsive design elements using flexbox
- Tag styling with blue backgrounds and rounded corners

### Frontend JavaScript

- Simple DOM manipulation for clock functionality
- Event handling for page lifecycle
- Data attributes used to pass configuration from backend to frontend

### Template System

- Askama templates provide type-safe HTML generation
- Template variables are passed from backend structs
- Conditional rendering using template match expressions

## File Structure

```
iron_shield/
├── Cargo.toml          # Project dependencies and metadata
├── Cargo.lock          # Locked dependency versions
├── Justfile            # Just command definitions
├── src/                # Source code
│   ├── main.rs         # Application entry point
│   ├── server.rs       # HTTP server and routing
│   ├── index.rs        # Index page generation
│   ├── config.rs       # Configuration management
│   ├── uptime.rs       # Uptime monitoring functionality
│   ├── error.rs        # Error handling and management
│   └─── lib.rs         # Library module with shared functionality
├── templates/          # HTML templates
│   └── index.html      # Main page template
└── static/             # Static assets
│   ├── style.css       # CSS styling
│   └── script.js       # Client-side JavaScript
└── QWEN.md            # This documentation file
```

## Potential Enhancements

- External configuration file support (JSON, YAML, etc.)
- Database integration for persistent site and configuration storage
- User authentication and multi-user support
- More sophisticated weather integration
- Bookmark management UI
- Additional theme options
