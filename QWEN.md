# Iron Shield Project Documentation

## Project Overview

Iron Shield is a Rust-based web application that serves as a customizable start page or dashboard. The application provides a clean, minimal interface with integrated features like a digital clock, search engine selection, weather information, and bookmark-style links to frequently used websites.

The project is built using the Axum web framework and utilizes the Askama templating engine for server-side rendering. It follows a modular architecture with separate concerns for configuration, site management, and web server functionality.

## Architecture

The application follows a modular design with the following components:

### Core Modules
- `main.rs`: Application entry point that initializes and runs the server
- `server.rs`: Handles HTTP server setup and routing
- `index.rs`: Generates the main index page using template data
- `config.rs`: Manages application configuration and settings
- `sites.rs`: Handles site bookmarks and related data

### Static Assets
- `static/`: Contains CSS and JavaScript files for frontend styling and interactivity
- `templates/`: Contains Askama templates for server-side rendering

### Dependencies
The project uses several key Rust crates:
- `axum`: Web framework for building the server
- `askama`: Template engine for HTML generation
- `serde`: Serialization/deserialization support
- `tokio`: Asynchronous runtime
- `tower-http`: HTTP service utilities
- `json5`: JSON5 format support

## Features

### Clock Display
- Supports both standard and military time formats
- Updates in real-time using JavaScript
- Configurable via the application settings

### Search Engine Integration
- Configurable search engines with names, URLs, and icons
- Selectable from a dropdown in the search form
- Customizable through configuration

### Weather Information
- Location-based weather display
- Configurable latitude and longitude coordinates
- Metric/imperial unit support

### Site Bookmarks
- Organized list of bookmarked websites
- Each site includes a name, URL, icon, and tags
- Tags provide categorization for the sites

## Building and Running

To build and run the Iron Shield application:

1. **Prerequisites**:
   - Rust programming language (latest stable version)
   - Cargo package manager

2. **Build and Run**:
   ```bash
   cargo run
   ```
   This will compile the project and start the server.

3. **Access the Application**:
   - The application will be available at `http://0.0.0.0:3000`
   - The server will display a message confirming the launch address

4. **Development**:
   - For development, you can use `cargo check` to verify code compilation
   - Use `cargo test` to run any tests (though none are currently implemented)
   - Use `cargo fmt` to format the code according to Rust standards

## Configuration

The application configuration is currently hardcoded in the `config.rs` file but follows a pattern that could support external configuration. The configuration includes:

- `site_name`: Name displayed in the page title
- `clock`: Time format (standard or military)
- `search_engines`: List of available search engines with their properties
- `weather`: Location and unit settings for weather information

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
├── src/                # Source code
│   ├── main.rs         # Application entry point
│   ├── server.rs       # HTTP server and routing
│   ├── index.rs        # Index page generation
│   ├── config.rs       # Configuration management
│   └── sites.rs        # Site bookmark management
├── templates/          # HTML templates
│   └── index.html      # Main page template
├── static/             # Static assets
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