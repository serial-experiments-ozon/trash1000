//! Domain models for the SWEeM API.
//!
//! These structs match the OpenAPI schema and use serde for JSON deserialization.
//! DateOnly from C# is mapped to NaiveDate in Rust.
//! Includes both read DTOs and write DTOs for CRUD operations.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User role enumeration (Manager = 0, Admin = 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(from = "i32", into = "i32")]
pub enum Role {
    #[default]
    Manager = 0,
    Admin = 1,
}

impl Role {
    /// Get all available roles
    pub fn all() -> &'static [Role] {
        &[Role::Manager, Role::Admin]
    }

    /// Cycle to the next role
    pub fn next(&self) -> Self {
        match self {
            Role::Manager => Role::Admin,
            Role::Admin => Role::Manager,
        }
    }
}

impl From<i32> for Role {
    fn from(value: i32) -> Self {
        match value {
            1 => Role::Admin,
            _ => Role::Manager,
        }
    }
}

impl From<Role> for i32 {
    fn from(role: Role) -> Self {
        role as i32
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Manager => write!(f, "Manager"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}

// ============================================
// Client DTOs
// ============================================

/// Client data transfer object (read)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDto {
    pub id: Uuid,
    pub name: Option<String>,
    pub address: Option<String>,
    pub projects_total: i32,
    pub projects_completed: i32,
}

impl ClientDto {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unnamed Client")
    }
}

/// Create client DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientDto {
    pub name: Option<String>,
    pub address: Option<String>,
    #[serde(default)]
    pub projects_total: i32,
    #[serde(default)]
    pub projects_completed: i32,
}

impl CreateClientDto {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        Ok(())
    }
}

/// Update client DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClientDto {
    pub name: Option<String>,
    pub address: Option<String>,
    #[serde(default)]
    pub projects_total: i32,
    #[serde(default)]
    pub projects_completed: i32,
}

impl UpdateClientDto {
    pub fn from_client(client: &ClientDto) -> Self {
        Self {
            name: client.name.clone(),
            address: client.address.clone(),
            projects_total: client.projects_total,
            projects_completed: client.projects_completed,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        Ok(())
    }
}

// ============================================
// Project DTOs
// ============================================

/// Project data transfer object (read)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: Uuid,
    pub client_id: Uuid,
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub planned_end_date: NaiveDate,
    pub actual_end_date: Option<NaiveDate>,
    pub manager_id: Uuid,
}

impl ProjectDto {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unnamed Project")
    }

    /// Calculate project duration in days
    pub fn duration_days(&self) -> i64 {
        (self.planned_end_date - self.start_date).num_days()
    }

    /// Check if project is completed
    pub fn is_completed(&self) -> bool {
        self.actual_end_date.is_some()
    }

    /// Check if project is overdue (past planned end date but not completed)
    pub fn is_overdue(&self) -> bool {
        if self.is_completed() {
            return false;
        }
        let today = chrono::Local::now().date_naive();
        today > self.planned_end_date
    }
}

/// Create project DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectDto {
    pub client_id: Uuid,
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub planned_end_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_end_date: Option<NaiveDate>,
    pub manager_id: Uuid,
}

impl Default for CreateProjectDto {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        Self {
            client_id: Uuid::nil(),
            name: None,
            start_date: today,
            planned_end_date: today + chrono::Duration::days(30),
            actual_end_date: None,
            manager_id: Uuid::nil(),
        }
    }
}

impl CreateProjectDto {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        if self.client_id.is_nil() {
            return Err("Client is required");
        }
        if self.manager_id.is_nil() {
            return Err("Manager is required");
        }
        if self.planned_end_date < self.start_date {
            return Err("End date must be after start date");
        }
        Ok(())
    }
}

/// Update project DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectDto {
    pub client_id: Uuid,
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub planned_end_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_end_date: Option<NaiveDate>,
    pub manager_id: Uuid,
}

impl UpdateProjectDto {
    pub fn from_project(project: &ProjectDto) -> Self {
        Self {
            client_id: project.client_id,
            name: project.name.clone(),
            start_date: project.start_date,
            planned_end_date: project.planned_end_date,
            actual_end_date: project.actual_end_date,
            manager_id: project.manager_id,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        if self.client_id.is_nil() {
            return Err("Client is required");
        }
        if self.manager_id.is_nil() {
            return Err("Manager is required");
        }
        if self.planned_end_date < self.start_date {
            return Err("End date must be after start date");
        }
        Ok(())
    }
}

// ============================================
// User DTOs
// ============================================

/// User data transfer object (read)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub name: Option<String>,
    pub login: Option<String>,
    pub role: Role,
}

impl UserDto {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unnamed User")
    }

    /// Check if user is a manager (can be assigned to projects)
    pub fn is_manager(&self) -> bool {
        self.role == Role::Manager
    }
}

/// Create user DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDto {
    pub name: Option<String>,
    pub login: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub role: Role,
}

impl CreateUserDto {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        if self.login.as_ref().map_or(true, |l| l.trim().is_empty()) {
            return Err("Login is required");
        }
        if self.password.as_ref().map_or(true, |p| p.is_empty()) {
            return Err("Password is required");
        }
        if self.password.as_ref().map_or(false, |p| p.len() < 4) {
            return Err("Password must be at least 4 characters");
        }
        Ok(())
    }
}

/// Update user DTO (write)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub role: Role,
}

impl UpdateUserDto {
    pub fn from_user(user: &UserDto) -> Self {
        Self {
            name: user.name.clone(),
            login: user.login.clone(),
            password: None, // Don't include existing password
            role: user.role,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err("Name is required");
        }
        if self.login.as_ref().map_or(true, |l| l.trim().is_empty()) {
            return Err("Login is required");
        }
        // Password is optional for updates
        if let Some(ref p) = self.password {
            if !p.is_empty() && p.len() < 4 {
                return Err("Password must be at least 4 characters");
            }
        }
        Ok(())
    }
}

// ============================================
// Pagination
// ============================================

/// Generic paginated result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResult<T> {
    pub items: Option<Vec<T>>,
    pub page: i32,
    pub page_size: i32,
    pub total_count: i32,
    pub total_pages: i32,
    pub has_previous: bool,
    pub has_next: bool,
}

impl<T> PaginatedResult<T> {
    pub fn items(&self) -> &[T] {
        self.items.as_deref().unwrap_or(&[])
    }
}

// ============================================
// Error handling
// ============================================

/// Problem details for API error responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: Option<String>,
    pub title: Option<String>,
    pub status: Option<i32>,
    pub detail: Option<String>,
    pub instance: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_serialization() {
        assert_eq!(Role::from(0), Role::Manager);
        assert_eq!(Role::from(1), Role::Admin);
        assert_eq!(i32::from(Role::Manager), 0);
        assert_eq!(i32::from(Role::Admin), 1);
    }

    #[test]
    fn test_project_duration() {
        let project = ProjectDto {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: Some("Test".to_string()),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            planned_end_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            actual_end_date: None,
            manager_id: Uuid::new_v4(),
        };
        assert_eq!(project.duration_days(), 30);
    }

    #[test]
    fn test_create_client_validation() {
        let mut dto = CreateClientDto::new();
        assert!(dto.validate().is_err());

        dto.name = Some("Test Client".to_string());
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_create_user_validation() {
        let mut dto = CreateUserDto::new();
        assert!(dto.validate().is_err());

        dto.name = Some("Test User".to_string());
        assert!(dto.validate().is_err());

        dto.login = Some("testuser".to_string());
        assert!(dto.validate().is_err());

        dto.password = Some("pass".to_string());
        assert!(dto.validate().is_ok());
    }
}
