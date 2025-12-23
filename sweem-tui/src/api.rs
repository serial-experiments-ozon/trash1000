//! API client for the SWEeM REST API.
//!
//! This module provides an async HTTP client for communicating with the backend.
//! All methods are non-blocking and designed to run in a separate Tokio task.
//! Supports full CRUD operations for Clients, Projects, and Users.

use anyhow::{Context, Result};
use reqwest::Client;
use uuid::Uuid;

use crate::models::{
    ClientDto, CreateClientDto, CreateProjectDto, CreateUserDto, PaginatedResult, ProjectDto,
    UpdateClientDto, UpdateProjectDto, UpdateUserDto, UserDto,
};

/// Default API base URL
pub const DEFAULT_BASE_URL: &str = "http://localhost:5094";

/// API client for the SWEeM backend
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    /// Create a new API client with the specified base URL
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    /// Create a new API client with the default base URL
    pub fn with_default_url() -> Result<Self> {
        Self::new(DEFAULT_BASE_URL)
    }

    // ============================================
    // Projects CRUD
    // ============================================

    /// Fetch all projects with pagination
    pub async fn fetch_projects(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResult<ProjectDto>> {
        let url = format!(
            "{}/projects?page={}&pageSize={}",
            self.base_url, page, page_size
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to projects endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse projects response")
    }

    /// Fetch all projects (unpaginated, fetches all pages)
    pub async fn fetch_all_projects(&self) -> Result<Vec<ProjectDto>> {
        let mut all_projects = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let result = self.fetch_projects(page, page_size).await?;
            all_projects.extend(result.items().to_vec());

            if !result.has_next {
                break;
            }
            page += 1;
        }

        Ok(all_projects)
    }

    /// Create a new project
    pub async fn create_project(&self, project: &CreateProjectDto) -> Result<Uuid> {
        let url = format!("{}/projects", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(project)
            .send()
            .await
            .context("Failed to send create project request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse create project response")
    }

    /// Update an existing project
    pub async fn update_project(&self, id: Uuid, project: &UpdateProjectDto) -> Result<ProjectDto> {
        let url = format!("{}/projects/{}", self.base_url, id);

        let response = self
            .client
            .put(&url)
            .json(project)
            .send()
            .await
            .context("Failed to send update project request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse update project response")
    }

    /// Delete a project
    pub async fn delete_project(&self, id: Uuid) -> Result<Uuid> {
        let url = format!("{}/projects/{}", self.base_url, id);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send delete project request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse delete project response")
    }

    // ============================================
    // Clients CRUD
    // ============================================

    /// Fetch all clients with pagination
    pub async fn fetch_clients(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResult<ClientDto>> {
        let url = format!(
            "{}/clients?page={}&pageSize={}",
            self.base_url, page, page_size
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to clients endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse clients response")
    }

    /// Fetch all clients (unpaginated, fetches all pages)
    pub async fn fetch_all_clients(&self) -> Result<Vec<ClientDto>> {
        let mut all_clients = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let result = self.fetch_clients(page, page_size).await?;
            all_clients.extend(result.items().to_vec());

            if !result.has_next {
                break;
            }
            page += 1;
        }

        Ok(all_clients)
    }

    /// Create a new client
    pub async fn create_client(&self, client_dto: &CreateClientDto) -> Result<Uuid> {
        let url = format!("{}/clients", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(client_dto)
            .send()
            .await
            .context("Failed to send create client request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse create client response")
    }

    /// Update an existing client
    pub async fn update_client(&self, id: Uuid, client_dto: &UpdateClientDto) -> Result<ClientDto> {
        let url = format!("{}/clients/{}", self.base_url, id);

        let response = self
            .client
            .put(&url)
            .json(client_dto)
            .send()
            .await
            .context("Failed to send update client request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse update client response")
    }

    /// Delete a client
    pub async fn delete_client(&self, id: Uuid) -> Result<Uuid> {
        let url = format!("{}/clients/{}", self.base_url, id);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send delete client request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse delete client response")
    }

    // ============================================
    // Users CRUD
    // ============================================

    /// Fetch all users with pagination
    pub async fn fetch_users(&self, page: i32, page_size: i32) -> Result<PaginatedResult<UserDto>> {
        let url = format!(
            "{}/users?page={}&pageSize={}",
            self.base_url, page, page_size
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to users endpoint")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse users response")
    }

    /// Fetch all users (unpaginated, fetches all pages)
    pub async fn fetch_all_users(&self) -> Result<Vec<UserDto>> {
        let mut all_users = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let result = self.fetch_users(page, page_size).await?;
            all_users.extend(result.items().to_vec());

            if !result.has_next {
                break;
            }
            page += 1;
        }

        Ok(all_users)
    }

    /// Create a new user
    pub async fn create_user(&self, user: &CreateUserDto) -> Result<Uuid> {
        let url = format!("{}/users", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(user)
            .send()
            .await
            .context("Failed to send create user request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse create user response")
    }

    /// Update an existing user
    pub async fn update_user(&self, id: Uuid, user: &UpdateUserDto) -> Result<UserDto> {
        let url = format!("{}/users/{}", self.base_url, id);

        let response = self
            .client
            .put(&url)
            .json(user)
            .send()
            .await
            .context("Failed to send update user request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse update user response")
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> Result<Uuid> {
        let url = format!("{}/users/{}", self.base_url, id);

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send delete user request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        response
            .json()
            .await
            .context("Failed to parse delete user response")
    }

    // ============================================
    // Utility
    // ============================================

    /// Health check - attempts to fetch first page of projects
    pub async fn health_check(&self) -> Result<bool> {
        match self.fetch_projects(1, 1).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Messages sent from API worker to the main TUI thread
#[derive(Debug, Clone)]
pub enum ApiMessage {
    /// Projects data has been loaded
    ProjectsLoaded(Vec<ProjectDto>),
    /// Clients data has been loaded
    ClientsLoaded(Vec<ClientDto>),
    /// Users data has been loaded
    UsersLoaded(Vec<UserDto>),
    /// An error occurred during API communication
    Error(String),
    /// API connection status changed
    ConnectionStatus(bool),
    /// Entity created successfully
    Created(EntityType, Uuid),
    /// Entity updated successfully
    Updated(EntityType),
    /// Entity deleted successfully
    Deleted(EntityType, Uuid),
}

/// Entity types for CRUD operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Client,
    Project,
    User,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Client => write!(f, "Client"),
            EntityType::Project => write!(f, "Project"),
            EntityType::User => write!(f, "User"),
        }
    }
}

/// Commands sent from TUI to the API worker
#[derive(Debug, Clone)]
pub enum ApiCommand {
    /// Request to refresh all data
    RefreshAll,
    /// Request to refresh projects only
    RefreshProjects,
    /// Request to refresh clients only
    RefreshClients,
    /// Request to refresh users only
    RefreshUsers,
    /// Check API connection status
    CheckConnection,
    /// Shutdown the API worker
    Shutdown,
    // CRUD Commands
    /// Create a new client
    CreateClient(CreateClientDto),
    /// Update an existing client
    UpdateClient(Uuid, UpdateClientDto),
    /// Delete a client
    DeleteClient(Uuid),
    /// Create a new project
    CreateProject(CreateProjectDto),
    /// Update an existing project
    UpdateProject(Uuid, UpdateProjectDto),
    /// Delete a project
    DeleteProject(Uuid),
    /// Create a new user
    CreateUser(CreateUserDto),
    /// Update an existing user
    UpdateUser(Uuid, UpdateUserDto),
    /// Delete a user
    DeleteUser(Uuid),
}
