//! SWEeM TUI - Terminal User Interface for SWEeM Project Management
//!
//! A modern TUI frontend with Kanagawa Dragon theme aesthetic,
//! featuring floating ash particles and full CRUD operations.

mod api;
mod app;
mod models;
mod particles;
mod theme;
mod timeline;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use api::{ApiClient, ApiCommand, ApiMessage, EntityType};
use app::App;

/// Frame rate for animations (approximately 30 FPS)
const FRAME_DURATION: Duration = Duration::from_millis(33);

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install().ok();

    // Parse command line arguments for API URL
    let args: Vec<String> = std::env::args().collect();
    let api_url = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or(api::DEFAULT_BASE_URL);

    // Run the TUI
    run_tui(api_url).await
}

/// Run the TUI application
async fn run_tui(api_url: &str) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create communication channels
    let (api_tx, mut api_rx) = mpsc::channel::<ApiMessage>(32);
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<ApiCommand>(32);

    // Create API client and spawn worker task
    let api_client = ApiClient::new(api_url)?;
    let api_client_clone = api_client.clone();
    let api_task = tokio::spawn(async move {
        run_api_worker(api_client_clone, api_tx, &mut cmd_rx).await
    });

    // Send initial refresh command
    cmd_tx.send(ApiCommand::RefreshAll).await.ok();

    // Create application state
    let mut app = App::new();

    // Main event loop
    let result = run_event_loop(&mut terminal, &mut app, &mut api_rx, &cmd_tx).await;

    // Cleanup
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    // Wait for API task to finish
    api_task.abort();

    result
}

/// Run the API worker task
async fn run_api_worker(
    client: ApiClient,
    tx: mpsc::Sender<ApiMessage>,
    rx: &mut mpsc::Receiver<ApiCommand>,
) {
    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    ApiCommand::RefreshAll => {
                        // Check connection
                        let connected = client.health_check().await.unwrap_or(false);
                        tx.send(ApiMessage::ConnectionStatus(connected)).await.ok();

                        if connected {
                            // Fetch all data concurrently
                            let (projects, clients, users) = tokio::join!(
                                client.fetch_all_projects(),
                                client.fetch_all_clients(),
                                client.fetch_all_users()
                            );

                            // Send results
                            match projects {
                                Ok(data) => { tx.send(ApiMessage::ProjectsLoaded(data)).await.ok(); }
                                Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                            }
                            match clients {
                                Ok(data) => { tx.send(ApiMessage::ClientsLoaded(data)).await.ok(); }
                                Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                            }
                            match users {
                                Ok(data) => { tx.send(ApiMessage::UsersLoaded(data)).await.ok(); }
                                Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                            }
                        } else {
                            tx.send(ApiMessage::Error("Cannot connect to API".to_string())).await.ok();
                        }
                    }
                    ApiCommand::RefreshProjects => {
                        match client.fetch_all_projects().await {
                            Ok(data) => { tx.send(ApiMessage::ProjectsLoaded(data)).await.ok(); }
                            Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                        }
                    }
                    ApiCommand::RefreshClients => {
                        match client.fetch_all_clients().await {
                            Ok(data) => { tx.send(ApiMessage::ClientsLoaded(data)).await.ok(); }
                            Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                        }
                    }
                    ApiCommand::RefreshUsers => {
                        match client.fetch_all_users().await {
                            Ok(data) => { tx.send(ApiMessage::UsersLoaded(data)).await.ok(); }
                            Err(e) => { tx.send(ApiMessage::Error(e.to_string())).await.ok(); }
                        }
                    }
                    ApiCommand::CheckConnection => {
                        let connected = client.health_check().await.unwrap_or(false);
                        tx.send(ApiMessage::ConnectionStatus(connected)).await.ok();
                    }
                    ApiCommand::Shutdown => {
                        break;
                    }
                    // CRUD operations for Clients
                    ApiCommand::CreateClient(dto) => {
                        match client.create_client(&dto).await {
                            Ok(id) => {
                                tx.send(ApiMessage::Created(EntityType::Client, id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Create client failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::UpdateClient(id, dto) => {
                        match client.update_client(id, &dto).await {
                            Ok(_) => {
                                tx.send(ApiMessage::Updated(EntityType::Client)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Update client failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::DeleteClient(id) => {
                        match client.delete_client(id).await {
                            Ok(deleted_id) => {
                                tx.send(ApiMessage::Deleted(EntityType::Client, deleted_id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Delete client failed: {}", e))).await.ok();
                            }
                        }
                    }
                    // CRUD operations for Projects
                    ApiCommand::CreateProject(dto) => {
                        match client.create_project(&dto).await {
                            Ok(id) => {
                                tx.send(ApiMessage::Created(EntityType::Project, id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Create project failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::UpdateProject(id, dto) => {
                        match client.update_project(id, &dto).await {
                            Ok(_) => {
                                tx.send(ApiMessage::Updated(EntityType::Project)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Update project failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::DeleteProject(id) => {
                        match client.delete_project(id).await {
                            Ok(deleted_id) => {
                                tx.send(ApiMessage::Deleted(EntityType::Project, deleted_id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Delete project failed: {}", e))).await.ok();
                            }
                        }
                    }
                    // CRUD operations for Users
                    ApiCommand::CreateUser(dto) => {
                        match client.create_user(&dto).await {
                            Ok(id) => {
                                tx.send(ApiMessage::Created(EntityType::User, id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Create user failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::UpdateUser(id, dto) => {
                        match client.update_user(id, &dto).await {
                            Ok(_) => {
                                tx.send(ApiMessage::Updated(EntityType::User)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Update user failed: {}", e))).await.ok();
                            }
                        }
                    }
                    ApiCommand::DeleteUser(id) => {
                        match client.delete_user(id).await {
                            Ok(deleted_id) => {
                                tx.send(ApiMessage::Deleted(EntityType::User, deleted_id)).await.ok();
                            }
                            Err(e) => {
                                tx.send(ApiMessage::Error(format!("Delete user failed: {}", e))).await.ok();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Run the main event loop
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    api_rx: &mut mpsc::Receiver<ApiMessage>,
    cmd_tx: &mpsc::Sender<ApiCommand>,
) -> Result<()> {
    loop {
        // Get terminal size for particle updates
        let size = terminal.size()?;

        // Update animations
        app.tick(size.width, size.height);

        // Render the UI
        terminal.draw(|frame| ui::render(frame, app))?;

        // Check for API messages (non-blocking)
        while let Ok(msg) = api_rx.try_recv() {
            // After CRUD operations, refresh the relevant data
            let should_refresh = match &msg {
                ApiMessage::Created(entity_type, _) | ApiMessage::Deleted(entity_type, _) => {
                    Some(*entity_type)
                }
                ApiMessage::Updated(entity_type) => Some(*entity_type),
                _ => None,
            };

            app.handle_api_message(msg);

            // Trigger data refresh after mutations
            if let Some(entity_type) = should_refresh {
                let refresh_cmd = match entity_type {
                    EntityType::Client => ApiCommand::RefreshClients,
                    EntityType::Project => ApiCommand::RefreshProjects,
                    EntityType::User => ApiCommand::RefreshUsers,
                };
                cmd_tx.send(refresh_cmd).await.ok();
                // Also refresh related entities for project dropdown updates
                if entity_type == EntityType::Client || entity_type == EntityType::User {
                    cmd_tx.send(ApiCommand::RefreshProjects).await.ok();
                }
            }
        }

        // Handle input events with timeout for animation
        if event::poll(FRAME_DURATION)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    if let Some(cmd) = app.handle_key(key) {
                        cmd_tx.send(cmd).await.ok();
                    }
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
