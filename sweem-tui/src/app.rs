//! Application state and event handling.
//!
//! This module implements the Elm Architecture pattern for state management,
//! with a centralized App struct holding all application state.
//! Includes form state for CRUD operations.

#![allow(dead_code)]

use std::time::{Duration, Instant};

use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::api::{ApiCommand, ApiMessage, EntityType};
use crate::models::{
    ClientDto, CreateClientDto, CreateProjectDto, CreateUserDto, ProjectDto, Role,
    UpdateClientDto, UpdateProjectDto, UpdateUserDto, UserDto,
};
use crate::particles::ParticleSystem;
use crate::timeline::TimelineState;

/// Active tab in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Clients list view
    Clients,
    /// Project timeline view (default)
    #[default]
    Timeline,
    /// Users list view
    Users,
}

impl Tab {
    /// Move to the next tab
    pub fn next(&self) -> Self {
        match self {
            Tab::Clients => Tab::Timeline,
            Tab::Timeline => Tab::Users,
            Tab::Users => Tab::Clients,
        }
    }

    /// Move to the previous tab
    pub fn previous(&self) -> Self {
        match self {
            Tab::Clients => Tab::Users,
            Tab::Timeline => Tab::Clients,
            Tab::Users => Tab::Timeline,
        }
    }

    /// Get the display name of the tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Clients => "Clients",
            Tab::Timeline => "Timeline",
            Tab::Users => "Users",
        }
    }
}

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Editing a form field
    Editing,
    /// Confirmation dialog (delete)
    Confirming,
}

/// Type of form being displayed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormType {
    /// Creating a new client
    CreateClient,
    /// Editing an existing client
    EditClient(Uuid),
    /// Creating a new project
    CreateProject,
    /// Editing an existing project
    EditProject(Uuid),
    /// Creating a new user
    CreateUser,
    /// Editing an existing user
    EditUser(Uuid),
}

/// Form field types for different entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    // Client fields
    ClientName,
    ClientAddress,
    // Project fields
    ProjectName,
    ProjectClient,
    ProjectManager,
    ProjectStartDate,
    ProjectEndDate,
    // User fields
    UserName,
    UserLogin,
    UserPassword,
    UserRole,
    // Form buttons
    SubmitButton,
    CancelButton,
}

impl FormField {
    /// Get all fields for client form
    pub fn client_fields() -> &'static [FormField] {
        &[
            FormField::ClientName,
            FormField::ClientAddress,
            FormField::SubmitButton,
            FormField::CancelButton,
        ]
    }

    /// Get all fields for project form
    pub fn project_fields() -> &'static [FormField] {
        &[
            FormField::ProjectName,
            FormField::ProjectClient,
            FormField::ProjectManager,
            FormField::ProjectStartDate,
            FormField::ProjectEndDate,
            FormField::SubmitButton,
            FormField::CancelButton,
        ]
    }

    /// Get all fields for user form
    pub fn user_fields() -> &'static [FormField] {
        &[
            FormField::UserName,
            FormField::UserLogin,
            FormField::UserPassword,
            FormField::UserRole,
            FormField::SubmitButton,
            FormField::CancelButton,
        ]
    }

    /// Get display label for the field
    pub fn label(&self) -> &'static str {
        match self {
            FormField::ClientName => "Name",
            FormField::ClientAddress => "Address",
            FormField::ProjectName => "Name",
            FormField::ProjectClient => "Client",
            FormField::ProjectManager => "Manager",
            FormField::ProjectStartDate => "Start Date",
            FormField::ProjectEndDate => "End Date",
            FormField::UserName => "Name",
            FormField::UserLogin => "Login",
            FormField::UserPassword => "Password",
            FormField::UserRole => "Role",
            FormField::SubmitButton => "Save",
            FormField::CancelButton => "Cancel",
        }
    }

    /// Check if this is a text input field
    pub fn is_text_input(&self) -> bool {
        matches!(
            self,
            FormField::ClientName
                | FormField::ClientAddress
                | FormField::ProjectName
                | FormField::UserName
                | FormField::UserLogin
                | FormField::UserPassword
        )
    }

    /// Check if this is a date picker field
    pub fn is_date_picker(&self) -> bool {
        matches!(
            self,
            FormField::ProjectStartDate | FormField::ProjectEndDate
        )
    }

    /// Check if this is a dropdown/selector field
    pub fn is_selector(&self) -> bool {
        matches!(
            self,
            FormField::ProjectClient | FormField::ProjectManager | FormField::UserRole
        )
    }

    /// Check if this is a button
    pub fn is_button(&self) -> bool {
        matches!(self, FormField::SubmitButton | FormField::CancelButton)
    }
}

/// State for the form modal
#[derive(Debug, Clone)]
pub struct FormState {
    /// Type of form
    pub form_type: FormType,
    /// Currently focused field index
    pub focused_field: usize,
    /// Current fields for the form
    pub fields: Vec<FormField>,
    /// Validation error message
    pub error: Option<String>,
    // Client form data
    pub client_name: String,
    pub client_address: String,
    // Project form data
    pub project_name: String,
    pub project_client_idx: usize,
    pub project_manager_idx: usize,
    pub project_start_date: String,
    pub project_end_date: String,
    // User form data
    pub user_name: String,
    pub user_login: String,
    pub user_password: String,
    pub user_role: Role,
    /// Whether dropdown is open
    pub dropdown_open: bool,
}

impl FormState {
    /// Create a new client creation form
    pub fn new_create_client() -> Self {
        Self {
            form_type: FormType::CreateClient,
            focused_field: 0,
            fields: FormField::client_fields().to_vec(),
            error: None,
            client_name: String::new(),
            client_address: String::new(),
            project_name: String::new(),
            project_client_idx: 0,
            project_manager_idx: 0,
            project_start_date: String::new(),
            project_end_date: String::new(),
            user_name: String::new(),
            user_login: String::new(),
            user_password: String::new(),
            user_role: Role::Manager,
            dropdown_open: false,
        }
    }

    /// Create an edit client form
    pub fn new_edit_client(client: &ClientDto) -> Self {
        Self {
            form_type: FormType::EditClient(client.id),
            focused_field: 0,
            fields: FormField::client_fields().to_vec(),
            error: None,
            client_name: client.name.clone().unwrap_or_default(),
            client_address: client.address.clone().unwrap_or_default(),
            project_name: String::new(),
            project_client_idx: 0,
            project_manager_idx: 0,
            project_start_date: String::new(),
            project_end_date: String::new(),
            user_name: String::new(),
            user_login: String::new(),
            user_password: String::new(),
            user_role: Role::Manager,
            dropdown_open: false,
        }
    }

    /// Create a new project creation form
    pub fn new_create_project() -> Self {
        let today = chrono::Local::now().date_naive();
        let end_date = today + chrono::Duration::days(30);
        Self {
            form_type: FormType::CreateProject,
            focused_field: 0,
            fields: FormField::project_fields().to_vec(),
            error: None,
            client_name: String::new(),
            client_address: String::new(),
            project_name: String::new(),
            project_client_idx: 0,
            project_manager_idx: 0,
            project_start_date: today.format("%Y-%m-%d").to_string(),
            project_end_date: end_date.format("%Y-%m-%d").to_string(),
            user_name: String::new(),
            user_login: String::new(),
            user_password: String::new(),
            user_role: Role::Manager,
            dropdown_open: false,
        }
    }

    /// Create an edit project form
    pub fn new_edit_project(project: &ProjectDto, clients: &[ClientDto], users: &[UserDto]) -> Self {
        let client_idx = clients.iter().position(|c| c.id == project.client_id).unwrap_or(0);
        let manager_idx = users.iter().position(|u| u.id == project.manager_id).unwrap_or(0);
        Self {
            form_type: FormType::EditProject(project.id),
            focused_field: 0,
            fields: FormField::project_fields().to_vec(),
            error: None,
            client_name: String::new(),
            client_address: String::new(),
            project_name: project.name.clone().unwrap_or_default(),
            project_client_idx: client_idx,
            project_manager_idx: manager_idx,
            project_start_date: project.start_date.format("%Y-%m-%d").to_string(),
            project_end_date: project.planned_end_date.format("%Y-%m-%d").to_string(),
            user_name: String::new(),
            user_login: String::new(),
            user_password: String::new(),
            user_role: Role::Manager,
            dropdown_open: false,
        }
    }

    /// Create a new user creation form
    pub fn new_create_user() -> Self {
        Self {
            form_type: FormType::CreateUser,
            focused_field: 0,
            fields: FormField::user_fields().to_vec(),
            error: None,
            client_name: String::new(),
            client_address: String::new(),
            project_name: String::new(),
            project_client_idx: 0,
            project_manager_idx: 0,
            project_start_date: String::new(),
            project_end_date: String::new(),
            user_name: String::new(),
            user_login: String::new(),
            user_password: String::new(),
            user_role: Role::Manager,
            dropdown_open: false,
        }
    }

    /// Create an edit user form
    pub fn new_edit_user(user: &UserDto) -> Self {
        Self {
            form_type: FormType::EditUser(user.id),
            focused_field: 0,
            fields: FormField::user_fields().to_vec(),
            error: None,
            client_name: String::new(),
            client_address: String::new(),
            project_name: String::new(),
            project_client_idx: 0,
            project_manager_idx: 0,
            project_start_date: String::new(),
            project_end_date: String::new(),
            user_name: user.name.clone().unwrap_or_default(),
            user_login: user.login.clone().unwrap_or_default(),
            user_password: String::new(),
            user_role: user.role,
            dropdown_open: false,
        }
    }

    /// Get the current focused field
    pub fn current_field(&self) -> FormField {
        self.fields[self.focused_field]
    }

    /// Move to the next field
    pub fn next_field(&mut self) {
        self.focused_field = (self.focused_field + 1) % self.fields.len();
        self.dropdown_open = false;
    }

    /// Move to the previous field
    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.checked_sub(1).unwrap_or(self.fields.len() - 1);
        self.dropdown_open = false;
    }

    /// Get mutable reference to the current text field (not date pickers or selectors)
    pub fn current_text_mut(&mut self) -> Option<&mut String> {
        match self.current_field() {
            FormField::ClientName => Some(&mut self.client_name),
            FormField::ClientAddress => Some(&mut self.client_address),
            FormField::ProjectName => Some(&mut self.project_name),
            FormField::UserName => Some(&mut self.user_name),
            FormField::UserLogin => Some(&mut self.user_login),
            FormField::UserPassword => Some(&mut self.user_password),
            // Date picker fields - use arrow keys instead of text input
            FormField::ProjectStartDate | FormField::ProjectEndDate => None,
            _ => None,
        }
    }

    /// Handle character input
    pub fn handle_char(&mut self, c: char) {
        if let Some(text) = self.current_text_mut() {
            text.push(c);
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if let Some(text) = self.current_text_mut() {
            text.pop();
        }
    }

    /// Increment the current date field by one day
    pub fn increment_date(&mut self) {
        match self.current_field() {
            FormField::ProjectStartDate => {
                self.project_start_date = Self::add_days_to_date_string(&self.project_start_date, 1);
            }
            FormField::ProjectEndDate => {
                self.project_end_date = Self::add_days_to_date_string(&self.project_end_date, 1);
            }
            _ => {}
        }
    }

    /// Decrement the current date field by one day
    pub fn decrement_date(&mut self) {
        match self.current_field() {
            FormField::ProjectStartDate => {
                self.project_start_date = Self::add_days_to_date_string(&self.project_start_date, -1);
            }
            FormField::ProjectEndDate => {
                self.project_end_date = Self::add_days_to_date_string(&self.project_end_date, -1);
            }
            _ => {}
        }
    }

    /// Add days to a date string in YYYY-MM-DD format
    fn add_days_to_date_string(date_str: &str, days: i64) -> String {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(|d| (d + chrono::Duration::days(days)).format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| {
                // If parsing fails, use today's date
                chrono::Local::now().date_naive().format("%Y-%m-%d").to_string()
            })
    }

    /// Build CreateClientDto from form state
    pub fn build_create_client(&self) -> CreateClientDto {
        CreateClientDto {
            name: Some(self.client_name.clone()),
            address: if self.client_address.is_empty() {
                None
            } else {
                Some(self.client_address.clone())
            },
            projects_total: 0,
            projects_completed: 0,
        }
    }

    /// Build UpdateClientDto from form state
    pub fn build_update_client(&self) -> UpdateClientDto {
        UpdateClientDto {
            name: Some(self.client_name.clone()),
            address: if self.client_address.is_empty() {
                None
            } else {
                Some(self.client_address.clone())
            },
            projects_total: 0,
            projects_completed: 0,
        }
    }

    /// Build CreateProjectDto from form state
    pub fn build_create_project(&self, clients: &[ClientDto], users: &[UserDto]) -> CreateProjectDto {
        let client_id = clients.get(self.project_client_idx).map(|c| c.id).unwrap_or(Uuid::nil());
        let manager_id = users.get(self.project_manager_idx).map(|u| u.id).unwrap_or(Uuid::nil());
        let start_date = NaiveDate::parse_from_str(&self.project_start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let end_date = NaiveDate::parse_from_str(&self.project_end_date, "%Y-%m-%d")
            .unwrap_or_else(|_| start_date + chrono::Duration::days(30));

        CreateProjectDto {
            client_id,
            name: Some(self.project_name.clone()),
            start_date,
            planned_end_date: end_date,
            actual_end_date: None,
            manager_id,
        }
    }

    /// Build UpdateProjectDto from form state
    pub fn build_update_project(&self, clients: &[ClientDto], users: &[UserDto]) -> UpdateProjectDto {
        let client_id = clients.get(self.project_client_idx).map(|c| c.id).unwrap_or(Uuid::nil());
        let manager_id = users.get(self.project_manager_idx).map(|u| u.id).unwrap_or(Uuid::nil());
        let start_date = NaiveDate::parse_from_str(&self.project_start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let end_date = NaiveDate::parse_from_str(&self.project_end_date, "%Y-%m-%d")
            .unwrap_or_else(|_| start_date + chrono::Duration::days(30));

        UpdateProjectDto {
            client_id,
            name: Some(self.project_name.clone()),
            start_date,
            planned_end_date: end_date,
            actual_end_date: None,
            manager_id,
        }
    }

    /// Build CreateUserDto from form state
    pub fn build_create_user(&self) -> CreateUserDto {
        CreateUserDto {
            name: Some(self.user_name.clone()),
            login: Some(self.user_login.clone()),
            password: Some(self.user_password.clone()),
            role: self.user_role,
        }
    }

    /// Build UpdateUserDto from form state
    pub fn build_update_user(&self) -> UpdateUserDto {
        UpdateUserDto {
            name: Some(self.user_name.clone()),
            login: Some(self.user_login.clone()),
            password: if self.user_password.is_empty() {
                None
            } else {
                Some(self.user_password.clone())
            },
            role: self.user_role,
        }
    }
}

/// Confirmation dialog state
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    /// Title of the dialog
    pub title: String,
    /// Message to display
    pub message: String,
    /// Entity type being deleted
    pub entity_type: EntityType,
    /// Entity ID being deleted
    pub entity_id: Uuid,
    /// Whether "Yes" is focused (false = "No" is focused)
    pub yes_focused: bool,
}

impl ConfirmDialog {
    pub fn new_delete(entity_type: EntityType, entity_id: Uuid, name: &str) -> Self {
        Self {
            title: format!("Delete {}", entity_type),
            message: format!("Are you sure you want to delete \"{}\"?\nThis action cannot be undone.", name),
            entity_type,
            entity_id,
            yes_focused: false,
        }
    }
}

/// Error popup state
#[derive(Debug, Clone)]
pub struct ErrorPopup {
    /// Error title
    pub title: String,
    /// Error message
    pub message: String,
    /// When the error was shown
    pub shown_at: Instant,
    /// Auto-dismiss duration (None for manual dismiss)
    pub auto_dismiss: Option<Duration>,
}

impl ErrorPopup {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            shown_at: Instant::now(),
            auto_dismiss: Some(Duration::from_secs(5)),
        }
    }

    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss {
            self.shown_at.elapsed() > duration
        } else {
            false
        }
    }
}

/// Log entry for the message area
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub message: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl LogEntry {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            timestamp: Instant::now(),
            message: message.into(),
            level: LogLevel::Info,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            timestamp: Instant::now(),
            message: message.into(),
            level: LogLevel::Success,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            timestamp: Instant::now(),
            message: message.into(),
            level: LogLevel::Warning,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            timestamp: Instant::now(),
            message: message.into(),
            level: LogLevel::Error,
        }
    }
}

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,

    /// Currently active tab
    pub active_tab: Tab,

    /// Current input mode
    pub input_mode: InputMode,

    /// Projects data
    pub projects: Vec<ProjectDto>,

    /// Clients data
    pub clients: Vec<ClientDto>,

    /// Users data
    pub users: Vec<UserDto>,

    /// Timeline widget state
    pub timeline_state: TimelineState,

    /// Particle system for background animation
    pub particle_system: ParticleSystem,

    /// Current error popup (if any)
    pub error_popup: Option<ErrorPopup>,

    /// Current form state (if any)
    pub form_state: Option<FormState>,

    /// Current confirm dialog (if any)
    pub confirm_dialog: Option<ConfirmDialog>,

    /// Log messages
    pub logs: Vec<LogEntry>,
    /// Maximum number of log entries to keep
    max_logs: usize,

    /// Selected index in lists (clients/users views)
    pub list_selected: usize,

    /// API connection status
    pub api_connected: bool,

    /// Last data refresh time
    pub last_refresh: Option<Instant>,

    /// Whether data is currently loading
    pub is_loading: bool,

    /// Frame counter for animations
    pub frame_count: u64,

    /// Show help overlay
    pub show_help: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let mut app = Self {
            should_quit: false,
            active_tab: Tab::Timeline,
            input_mode: InputMode::Normal,
            projects: Vec::new(),
            clients: Vec::new(),
            users: Vec::new(),
            timeline_state: TimelineState::default(),
            particle_system: ParticleSystem::default(),
            error_popup: None,
            form_state: None,
            confirm_dialog: None,
            logs: Vec::new(),
            max_logs: 100,
            list_selected: 0,
            api_connected: false,
            last_refresh: None,
            is_loading: true,
            frame_count: 0,
            show_help: false,
        };

        app.log(LogEntry::info("SWEeM TUI initialized"));
        app.log(LogEntry::info("Connecting to API..."));
        app
    }

    /// Add a log entry
    pub fn log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }

    /// Show an error popup
    pub fn show_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let title = title.into();
        let message = message.into();
        self.log(LogEntry::error(format!("{}: {}", title, message)));
        self.error_popup = Some(ErrorPopup::new(title, message));
    }

    /// Dismiss the current error popup
    pub fn dismiss_error(&mut self) {
        self.error_popup = None;
    }

    /// Open create form for current tab
    pub fn open_create_form(&mut self) {
        let form = match self.active_tab {
            Tab::Clients => FormState::new_create_client(),
            Tab::Timeline => FormState::new_create_project(),
            Tab::Users => FormState::new_create_user(),
        };
        self.form_state = Some(form);
        self.input_mode = InputMode::Editing;
    }

    /// Open edit form for selected item
    pub fn open_edit_form(&mut self) {
        let form = match self.active_tab {
            Tab::Clients => {
                if let Some(client) = self.clients.get(self.list_selected) {
                    Some(FormState::new_edit_client(client))
                } else {
                    None
                }
            }
            Tab::Timeline => {
                if let Some(idx) = self.timeline_state.selected_project {
                    if let Some(project) = self.projects.get(idx) {
                        Some(FormState::new_edit_project(project, &self.clients, &self.users))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Tab::Users => {
                if let Some(user) = self.users.get(self.list_selected) {
                    Some(FormState::new_edit_user(user))
                } else {
                    None
                }
            }
        };

        if let Some(form) = form {
            self.form_state = Some(form);
            self.input_mode = InputMode::Editing;
        }
    }

    /// Open delete confirmation dialog
    pub fn open_delete_confirm(&mut self) {
        let dialog = match self.active_tab {
            Tab::Clients => {
                if let Some(client) = self.clients.get(self.list_selected) {
                    Some(ConfirmDialog::new_delete(
                        EntityType::Client,
                        client.id,
                        client.display_name(),
                    ))
                } else {
                    None
                }
            }
            Tab::Timeline => {
                if let Some(idx) = self.timeline_state.selected_project {
                    if let Some(project) = self.projects.get(idx) {
                        Some(ConfirmDialog::new_delete(
                            EntityType::Project,
                            project.id,
                            project.display_name(),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Tab::Users => {
                if let Some(user) = self.users.get(self.list_selected) {
                    Some(ConfirmDialog::new_delete(
                        EntityType::User,
                        user.id,
                        user.display_name(),
                    ))
                } else {
                    None
                }
            }
        };

        if let Some(dialog) = dialog {
            self.confirm_dialog = Some(dialog);
            self.input_mode = InputMode::Confirming;
        }
    }

    /// Close the current form
    pub fn close_form(&mut self) {
        self.form_state = None;
        self.input_mode = InputMode::Normal;
    }

    /// Close the confirm dialog
    pub fn close_confirm(&mut self) {
        self.confirm_dialog = None;
        self.input_mode = InputMode::Normal;
    }

    /// Handle API messages
    pub fn handle_api_message(&mut self, message: ApiMessage) {
        match message {
            ApiMessage::ProjectsLoaded(projects) => {
                let count = projects.len();
                self.projects = projects;
                self.is_loading = false;
                self.last_refresh = Some(Instant::now());
                self.log(LogEntry::success(format!("Loaded {} projects", count)));

                // Auto-center timeline on first project or today when projects are loaded
                if !self.projects.is_empty() {
                    // Select first project if none selected
                    if self.timeline_state.selected_project.is_none() {
                        self.timeline_state.selected_project = Some(0);
                    }
                    // Jump to show the selected (or first) project
                    self.auto_center_timeline();
                }
            }
            ApiMessage::ClientsLoaded(clients) => {
                let count = clients.len();
                self.clients = clients;
                self.log(LogEntry::success(format!("Loaded {} clients", count)));
            }
            ApiMessage::UsersLoaded(users) => {
                let count = users.len();
                self.users = users;
                self.log(LogEntry::success(format!("Loaded {} users", count)));
            }
            ApiMessage::Error(error) => {
                self.is_loading = false;
                self.show_error("API Error", error);
            }
            ApiMessage::ConnectionStatus(connected) => {
                let was_connected = self.api_connected;
                self.api_connected = connected;

                if connected && !was_connected {
                    self.log(LogEntry::success("Connected to API"));
                } else if !connected && was_connected {
                    self.log(LogEntry::warning("Disconnected from API"));
                }
            }
            ApiMessage::Created(entity_type, id) => {
                self.log(LogEntry::success(format!("{} created ({})", entity_type, &id.to_string()[..8])));
                self.close_form();
            }
            ApiMessage::Updated(entity_type) => {
                self.log(LogEntry::success(format!("{} updated", entity_type)));
                self.close_form();
            }
            ApiMessage::Deleted(entity_type, id) => {
                self.log(LogEntry::success(format!("{} deleted ({})", entity_type, &id.to_string()[..8])));
                self.close_confirm();
            }
        }
    }

    /// Handle key events and return optional API command
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ApiCommand> {
        // Handle error popup dismissal
        if self.error_popup.is_some() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')) {
                self.dismiss_error();
            }
            return None;
        }

        // Handle help overlay
        if self.show_help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter) {
                self.show_help = false;
            }
            return None;
        }

        // Handle based on input mode
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Editing => self.handle_editing_key(key),
            InputMode::Confirming => self.handle_confirming_key(key),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_key(&mut self, key: KeyEvent) -> Option<ApiCommand> {
        // Global shortcuts
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                return Some(ApiCommand::Shutdown);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Some(ApiCommand::Shutdown);
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return None;
            }
            KeyCode::Char('p') => {
                self.particle_system.toggle_mode();
                let mode = self.particle_system.mode().name();
                self.log(LogEntry::info(format!("Particle mode: {}", mode)));
                return None;
            }
            KeyCode::Char('r') => {
                self.is_loading = true;
                self.log(LogEntry::info("Refreshing data..."));
                return Some(ApiCommand::RefreshAll);
            }
            KeyCode::Tab => {
                self.active_tab = self.active_tab.next();
                self.list_selected = 0;
                return None;
            }
            KeyCode::BackTab => {
                self.active_tab = self.active_tab.previous();
                self.list_selected = 0;
                return None;
            }
            // CRUD shortcuts
            KeyCode::Char('c') => {
                self.open_create_form();
                return None;
            }
            KeyCode::Char('e') => {
                self.open_edit_form();
                return None;
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                self.open_delete_confirm();
                return None;
            }
            _ => {}
        }

        // Tab-specific shortcuts
        match self.active_tab {
            Tab::Timeline => self.handle_timeline_key(key),
            Tab::Clients => self.handle_list_key(key, self.clients.len()),
            Tab::Users => self.handle_list_key(key, self.users.len()),
        }

        None
    }

    /// Handle keys in editing mode (form)
    fn handle_editing_key(&mut self, key: KeyEvent) -> Option<ApiCommand> {
        if self.form_state.is_none() {
            self.input_mode = InputMode::Normal;
            return None;
        }

        match key.code {
            KeyCode::Esc => {
                self.close_form();
                return None;
            }
            KeyCode::Tab => {
                if let Some(form) = &mut self.form_state {
                    form.next_field();
                }
                return None;
            }
            KeyCode::BackTab => {
                if let Some(form) = &mut self.form_state {
                    form.prev_field();
                }
                return None;
            }
            KeyCode::Enter => {
                return self.handle_form_submit();
            }
            KeyCode::Backspace => {
                if let Some(form) = &mut self.form_state {
                    form.handle_backspace();
                }
                return None;
            }
            KeyCode::Up => {
                if let Some(form) = &mut self.form_state {
                    let field = form.current_field();
                    if field.is_date_picker() {
                        // Date picker: Up increases the date
                        form.increment_date();
                    } else {
                        match field {
                            FormField::ProjectClient => {
                                if form.project_client_idx > 0 {
                                    form.project_client_idx -= 1;
                                }
                            }
                            FormField::ProjectManager => {
                                if form.project_manager_idx > 0 {
                                    form.project_manager_idx -= 1;
                                }
                            }
                            FormField::UserRole => {
                                form.user_role = form.user_role.next();
                            }
                            _ => {}
                        }
                    }
                }
                return None;
            }
            KeyCode::Down => {
                if let Some(form) = &mut self.form_state {
                    let field = form.current_field();
                    if field.is_date_picker() {
                        // Date picker: Down decreases the date
                        form.decrement_date();
                    } else {
                        match field {
                            FormField::ProjectClient => {
                                if form.project_client_idx < self.clients.len().saturating_sub(1) {
                                    form.project_client_idx += 1;
                                }
                            }
                            FormField::ProjectManager => {
                                if form.project_manager_idx < self.users.len().saturating_sub(1) {
                                    form.project_manager_idx += 1;
                                }
                            }
                            FormField::UserRole => {
                                form.user_role = form.user_role.next();
                            }
                            _ => {}
                        }
                    }
                }
                return None;
            }
            KeyCode::Left => {
                if let Some(form) = &mut self.form_state {
                    if form.current_field().is_date_picker() {
                        // Date picker: Left decreases by 7 days (one week)
                        for _ in 0..7 {
                            form.decrement_date();
                        }
                    }
                }
                return None;
            }
            KeyCode::Right => {
                if let Some(form) = &mut self.form_state {
                    if form.current_field().is_date_picker() {
                        // Date picker: Right increases by 7 days (one week)
                        for _ in 0..7 {
                            form.increment_date();
                        }
                    }
                }
                return None;
            }
            KeyCode::Char(c) => {
                if let Some(form) = &mut self.form_state {
                    form.handle_char(c);
                }
                return None;
            }
            _ => {}
        }

        None
    }

    /// Handle form submission
    fn handle_form_submit(&mut self) -> Option<ApiCommand> {
        // Extract data we need from form before borrowing self mutably
        let form = self.form_state.as_ref()?;

        // Check if cancel button is focused
        if form.current_field() == FormField::CancelButton {
            self.close_form();
            return None;
        }

        // On text input and date picker fields, Enter moves to next field instead of submitting
        if form.current_field().is_text_input() || form.current_field().is_date_picker() {
            if let Some(form) = &mut self.form_state {
                form.next_field();
            }
            return None;
        }

        // Only submit if on submit button
        if form.current_field() != FormField::SubmitButton {
            return None;
        }

        // Clone the form type to avoid borrow issues
        let form_type = form.form_type.clone();

        match form_type {
            FormType::CreateClient => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_create_client();
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Creating client..."));
                Some(ApiCommand::CreateClient(dto))
            }
            FormType::EditClient(id) => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_update_client();
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Updating client..."));
                Some(ApiCommand::UpdateClient(id, dto))
            }
            FormType::CreateProject => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_create_project(&self.clients, &self.users);
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Creating project..."));
                Some(ApiCommand::CreateProject(dto))
            }
            FormType::EditProject(id) => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_update_project(&self.clients, &self.users);
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Updating project..."));
                Some(ApiCommand::UpdateProject(id, dto))
            }
            FormType::CreateUser => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_create_user();
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Creating user..."));
                Some(ApiCommand::CreateUser(dto))
            }
            FormType::EditUser(id) => {
                let form = self.form_state.as_ref()?;
                let dto = form.build_update_user();
                if let Err(e) = dto.validate() {
                    if let Some(f) = &mut self.form_state {
                        f.error = Some(e.to_string());
                    }
                    return None;
                }
                self.log(LogEntry::info("Updating user..."));
                Some(ApiCommand::UpdateUser(id, dto))
            }
        }
    }

    /// Handle keys in confirming mode (delete dialog)
    fn handle_confirming_key(&mut self, key: KeyEvent) -> Option<ApiCommand> {
        if self.confirm_dialog.is_none() {
            self.input_mode = InputMode::Normal;
            return None;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.close_confirm();
                return None;
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                if let Some(dialog) = &mut self.confirm_dialog {
                    dialog.yes_focused = !dialog.yes_focused;
                }
                return None;
            }
            KeyCode::Enter => {
                if let Some(dialog) = &self.confirm_dialog {
                    if dialog.yes_focused {
                        let cmd = match dialog.entity_type {
                            EntityType::Client => ApiCommand::DeleteClient(dialog.entity_id),
                            EntityType::Project => ApiCommand::DeleteProject(dialog.entity_id),
                            EntityType::User => ApiCommand::DeleteUser(dialog.entity_id),
                        };
                        self.log(LogEntry::info(format!("Deleting {}...", dialog.entity_type)));
                        return Some(cmd);
                    } else {
                        self.close_confirm();
                    }
                }
                return None;
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(dialog) = &self.confirm_dialog {
                    let cmd = match dialog.entity_type {
                        EntityType::Client => ApiCommand::DeleteClient(dialog.entity_id),
                        EntityType::Project => ApiCommand::DeleteProject(dialog.entity_id),
                        EntityType::User => ApiCommand::DeleteUser(dialog.entity_id),
                    };
                    self.log(LogEntry::info(format!("Deleting {}...", dialog.entity_type)));
                    return Some(cmd);
                }
                return None;
            }
            _ => {}
        }

        None
    }

    /// Handle timeline-specific key events
    fn handle_timeline_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => {
                let amount = if key.modifiers.contains(KeyModifiers::SHIFT) { 7 } else { 1 };
                self.timeline_state.scroll_left(amount);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let amount = if key.modifiers.contains(KeyModifiers::SHIFT) { 7 } else { 1 };
                self.timeline_state.scroll_right(amount);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.timeline_state.select_next(self.projects.len());
                // Auto-jump to selected project when navigating
                self.jump_to_selected_project();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.timeline_state.select_previous(self.projects.len());
                // Auto-jump to selected project when navigating
                self.jump_to_selected_project();
            }
            KeyCode::Enter => {
                // Jump to center the selected project in the viewport
                self.jump_to_selected_project();
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.timeline_state.zoom_in();
            }
            KeyCode::Char('-') => {
                self.timeline_state.zoom_out();
            }
            KeyCode::Char('t') => {
                self.timeline_state.center_on_today(80); // Approximate width
            }
            KeyCode::Home => {
                self.timeline_state.scroll_offset = 0;
            }
            _ => {}
        }
    }

    /// Jump timeline viewport to show the currently selected project
    fn jump_to_selected_project(&mut self) {
        if let Some(idx) = self.timeline_state.selected_project {
            if let Some(project) = self.projects.get(idx) {
                // Use approximate viewport width (adjust based on typical terminal width)
                let viewport_width = 100u16;
                self.timeline_state.jump_to_project(project, &self.projects, viewport_width);
            }
        }
    }

    /// Auto-center the timeline on the selected project or first project
    fn auto_center_timeline(&mut self) {
        if self.projects.is_empty() {
            return;
        }

        // Reset scroll to beginning so projects are immediately visible
        // The timeline calculates start from the earliest project, so scroll 0 = first project visible
        self.timeline_state.scroll_offset = 0;
    }

    /// Handle list view key events
    fn handle_list_key(&mut self, key: KeyEvent, total: usize) {
        if total == 0 {
            return;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.list_selected = (self.list_selected + 1) % total;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.list_selected = self.list_selected.checked_sub(1).unwrap_or(total - 1);
            }
            KeyCode::Char('g') => {
                self.list_selected = 0;
            }
            KeyCode::Char('G') => {
                self.list_selected = total.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Update animations (called every frame)
    pub fn tick(&mut self, width: u16, height: u16) {
        self.frame_count = self.frame_count.wrapping_add(1);

        // Update particles
        self.particle_system.update(width, height);

        // Update timeline animations (goyslop effects!)
        self.timeline_state.tick();

        // Auto-dismiss error popup
        if let Some(ref popup) = self.error_popup {
            if popup.should_dismiss() {
                self.error_popup = None;
            }
        }
    }

    /// Get the status bar text
    pub fn status_text(&self) -> String {
        let connection = if self.api_connected {
            "Connected"
        } else {
            "Disconnected"
        };

        let loading = if self.is_loading { " [Loading...]" } else { "" };

        let last_refresh = self
            .last_refresh
            .map(|t| {
                let secs = t.elapsed().as_secs();
                if secs < 60 {
                    format!(" ({}s ago)", secs)
                } else {
                    format!(" ({}m ago)", secs / 60)
                }
            })
            .unwrap_or_default();

        format!(
            "{}{}{} | {} | ?: Help | c: Create | e: Edit | d: Delete | q: Quit",
            connection,
            loading,
            last_refresh,
            self.active_tab.name()
        )
    }
}
