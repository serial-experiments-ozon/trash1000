//! UI rendering module.
//!
//! This module handles all the TUI rendering using ratatui,
//! implementing the Kanagawa Dragon aesthetic with CRUD forms.

use chrono::{Datelike, NaiveDate};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::{App, FormField, FormState, FormType, LogLevel, Tab};
use crate::models::Role;
use crate::particles::ParticleWidget;
use crate::theme::{colors, styles};
use crate::radar::RadarWidget;

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Fill background with theme color
    let bg_block = Block::default().style(Style::default().bg(colors::BG_DARK));
    frame.render_widget(bg_block, area);

    // Render background particles
    frame.render_widget(ParticleWidget::new(&app.particle_system), area);

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Status bar / tabs
            Constraint::Min(10),    // Main content
            Constraint::Length(5),  // Log area
        ])
        .split(area);

    // Render components
    render_tabs(frame, app, chunks[0]);
    render_main_content(frame, app, chunks[1]);
    render_logs(frame, app, chunks[2]);

    // Render overlays (modals, dialogs)
    if app.form_state.is_some() {
        render_form_modal(frame, app, area);
    }

    if app.confirm_dialog.is_some() {
        render_confirm_dialog(frame, app, area);
    }

    if app.error_popup.is_some() {
        render_error_popup(frame, app, area);
    }

    if app.show_help {
        render_help_overlay(frame, area);
    }
}

/// Render the tab bar
fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = [Tab::Clients, Tab::Timeline, Tab::Users]
        .iter()
        .map(|tab| {
            let style = if *tab == app.active_tab {
                styles::tab_active()
            } else {
                styles::tab_inactive()
            };
            Line::from(Span::styled(format!(" {} ", tab.name()), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" SWEeM Management Console ")
                .title_style(styles::title())
                .borders(Borders::ALL)
                .border_style(styles::border())
                .style(Style::default().bg(colors::BG_MEDIUM)),
        )
        .select(match app.active_tab {
            Tab::Clients => 0,
            Tab::Timeline => 1,
            Tab::Users => 2,
        })
        .style(styles::text())
        .highlight_style(styles::tab_active())
        .divider(Span::styled(" | ", styles::border_dim()));

    frame.render_widget(tabs, area);
}

/// Render the main content area based on active tab
fn render_main_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.active_tab {
        Tab::Clients => render_clients_view(frame, app, area),
        Tab::Timeline => render_timeline_view(frame, app, area),
        Tab::Users => render_users_view(frame, app, area),
    }
}

fn render_timeline_view(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ])
        .split(area);

    // FIX: Pass clients to radar for labels
    let radar = RadarWidget::new(&app.projects, &app.clients, &app.radar_state);
    frame.render_widget(radar, chunks[0]);

    render_project_details(frame, app, chunks[1]);
}

fn render_project_details(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Target Analysis ")
        .title_style(styles::title_accent())
        .borders(Borders::ALL)
        .border_style(styles::border())
        .style(Style::default().bg(colors::BG_MEDIUM));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let project = match app.radar_state.selected_index {
        Some(i) => app.projects.get(i),
        None => None,
    };

    if let Some(p) = project {
        let details_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Header
                Constraint::Length(8), // Stats Grid
                Constraint::Min(0),    // Relations
            ])
            .margin(1)
            .split(inner_area);

        // -- Header --
        let mut text = vec![
            Line::from(Span::styled(
                p.display_name(), 
                Style::default().fg(colors::FG_PRIMARY).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            )),
            Line::from(Span::styled(
                format!("UUID: {}", p.id), 
                styles::text_dim()
            )),
        ];
        frame.render_widget(Paragraph::new(text), details_chunks[0]);

        // -- Metrics Calculation (FIXED) --
        let today = chrono::Local::now().date_naive();
        
        // 1. Deadline Math: Always use planned_end_date for deadline countdown
        let deadline_date = p.planned_end_date;
        let days_until_deadline = (deadline_date - today).num_days();

        // 2. Formatting Deadline with Sanity Check
        let (deadline_str, deadline_style) = if deadline_date.year() < 2000 {
            ("Not Set".to_string(), styles::text_dim())
        } else if p.is_completed() {
            ("Completed".to_string(), styles::success())
        } else if days_until_deadline < 0 {
            (format!("{} days OVERDUE", days_until_deadline.abs()), styles::error())
        } else {
            (format!("{} days left", days_until_deadline), styles::info())
        };

        // 3. Progress Math
        // If completed, 100%. Else calculate time elapsed vs planned duration.
        let total_duration = (p.planned_end_date - p.start_date).num_days().max(1);
        let elapsed = (today - p.start_date).num_days().max(0);
        let raw_pct = (elapsed as f64 / total_duration as f64).clamp(0.0, 1.0);
        
        let progress_pct = if p.is_completed() { 1.0 } else { raw_pct };
        let progress_bar_width = 20 as usize;
        let filled = (progress_pct * progress_bar_width as f64) as usize;
        let empty = progress_bar_width.saturating_sub(filled);
        let bar_str = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        let status_text = if p.is_completed() { "DONE" } 
                          else if p.is_overdue() { "LATE" } 
                          else { "ACTIVE" };
        
        let status_color = if p.is_completed() { colors::GREEN }
                           else if p.is_overdue() { colors::RED }
                           else { colors::BLUE };

        let total_duration = (p.planned_end_date - p.start_date).num_days().max(1);
        let elapsed = (today - p.start_date).num_days().max(0);
        
        // Если проект будущий, прогресс 0%
        let raw_pct = if p.is_pending() {
            0.0
        } else {
            (elapsed as f64 / total_duration as f64).clamp(0.0, 1.0)
        };
        
        let progress_pct = if p.is_completed() { 1.0 } else { raw_pct };
        let progress_bar_width = 20usize;
        let filled = (progress_pct * progress_bar_width as f64) as usize;
        let empty = progress_bar_width.saturating_sub(filled);
        let bar_str = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        // Logic fix: Handle Pending state
        let (status_text, status_color) = if p.is_completed() { 
            ("DONE", colors::GREEN)
        } else if p.is_overdue() { 
            ("LATE", colors::RED)
        } else if p.is_pending() {
            ("PLANNED", colors::FG_DIM) // Новый статус
        } else { 
            ("ACTIVE", colors::BLUE)
        };

        let stats = vec![
            Line::from(vec![
                Span::raw("Status:   "),
                Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("Deadline: "),
                Span::styled(deadline_str, deadline_style),
            ]),
            Line::from(vec![
                 Span::raw("Progress: "),
                 Span::styled(format!("{:.0}% ", progress_pct * 100.0), styles::text()),
                 Span::styled(bar_str, Style::default().fg(status_color)),
            ]),
            Line::from(vec![
                Span::raw("Start:    "),
                Span::styled(p.start_date.format("%Y-%m-%d").to_string(), styles::text_hint()),
            ]),
            Line::from(vec![
                Span::raw("Plan End: "),
                Span::styled(p.planned_end_date.format("%Y-%m-%d").to_string(), styles::text_hint()),
            ]),
        ];
        frame.render_widget(Paragraph::new(stats), details_chunks[1]);

        // -- Relations --
        let client_name = app.clients.iter().find(|c| c.id == p.client_id)
            .map(|c| c.display_name()).unwrap_or("Unknown ID");
        let manager_name = app.users.iter().find(|u| u.id == p.manager_id)
            .map(|u| u.display_name()).unwrap_or("Unknown ID");

        let relations = vec![
            Line::from(Span::styled("Personnel & Client:", styles::title())),
            Line::from(vec![Span::raw("  Client:  "), Span::styled(client_name, styles::info())]),
            Line::from(vec![Span::raw("  Manager: "), Span::styled(manager_name, styles::info())]),
        ];
        frame.render_widget(Paragraph::new(relations), details_chunks[2]);

    } else {
        let msg = vec![
            Line::from("Awaiting Selection..."),
            Line::from(""),
            Line::from(Span::styled("Use arrow keys to acquire target", styles::text_dim())),
        ];
        frame.render_widget(
            Paragraph::new(msg).alignment(Alignment::Center), 
            Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage(50), Constraint::Min(1)]).split(inner_area)[1]
        );
    }
}

/// Render the clients list view
fn render_clients_view(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .clients
        .iter()
        .enumerate()
        .map(|(i, client)| {
            let is_selected = i == app.list_selected;
            let style = if is_selected {
                Style::default()
                    .fg(colors::BG_DARK)
                    .bg(colors::BLUE)
                    .add_modifier(Modifier::BOLD)
            } else {
                styles::text()
            };

            // Calculate project counts from actual projects data
            let (completed, total) = calculate_client_project_counts(&app.projects, client.id);

            // Create a visual progress bar for projects
            let progress_bar = if total > 0 {
                let filled = (completed * 5 / total).min(5) as usize;
                let empty = 5 - filled;
                format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
            } else {
                "[░░░░░]".to_string()
            };

            // Choose color based on completion
            let progress_style = if is_selected {
                style
            } else if total == 0 {
                styles::text_dim()
            } else if completed == total {
                styles::success()
            } else if completed as f32 / total as f32 >= 0.5 {
                Style::default().fg(colors::YELLOW)
            } else {
                Style::default().fg(colors::ORANGE)
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:20}", client.display_name()),
                    style,
                ),
                Span::styled(" │ ", styles::border_dim()),
                Span::styled(
                    format!("{:30}", client.address.as_deref().unwrap_or("-")),
                    if is_selected { style } else { styles::text_dim() },
                ),
                Span::styled(" │ ", styles::border_dim()),
                Span::styled(progress_bar, progress_style),
                Span::styled(" ", Style::default()),
                Span::styled(
                    format!("{}/{}", completed, total),
                    progress_style,
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Clients ")
                .title_style(styles::title_accent())
                .borders(Borders::ALL)
                .border_style(styles::border())
                .style(Style::default().bg(colors::BG_DARK)),
        )
        .style(styles::text());

    frame.render_widget(list, area);

    // Render empty state
    if app.clients.is_empty() {
        render_empty_state(frame, area, "No clients found", app.is_loading);
    }
}

/// Render the users list view
fn render_users_view(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .users
        .iter()
        .enumerate()
        .map(|(i, user)| {
            let is_selected = i == app.list_selected;
            let style = if is_selected {
                Style::default()
                    .fg(colors::BG_DARK)
                    .bg(colors::PURPLE)
                    .add_modifier(Modifier::BOLD)
            } else {
                styles::text()
            };

            let role_color = match user.role {
                Role::Admin => colors::YELLOW,
                Role::Manager => colors::GREEN,
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{:20}", user.display_name()),
                    style,
                ),
                Span::styled(" | ", styles::border_dim()),
                Span::styled(
                    format!("{:20}", user.login.as_deref().unwrap_or("-")),
                    if is_selected { style } else { styles::text_dim() },
                ),
                Span::styled(" | ", styles::border_dim()),
                Span::styled(
                    format!("{:10}", user.role),
                    if is_selected { style } else { Style::default().fg(role_color) },
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Users ")
                .title_style(styles::title_accent())
                .borders(Borders::ALL)
                .border_style(styles::border())
                .style(Style::default().bg(colors::BG_DARK)),
        )
        .style(styles::text());

    frame.render_widget(list, area);

    // Render empty state
    if app.users.is_empty() {
        render_empty_state(frame, area, "No users found", app.is_loading);
    }
}

/// Render the log area
fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .map(|entry| {
            let (prefix, color) = match entry.level {
                LogLevel::Info => ("i", colors::BLUE),
                LogLevel::Success => ("+", colors::GREEN),
                LogLevel::Warning => ("!", colors::YELLOW),
                LogLevel::Error => ("x", colors::RED),
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", prefix), Style::default().fg(color)),
                Span::styled(&entry.message, styles::text_dim()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" System Log ")
                .title_style(Style::default().fg(colors::FG_DIM))
                .borders(Borders::ALL)
                .border_style(styles::border_dim())
                .style(Style::default().bg(colors::BG_DARK)),
        );

    frame.render_widget(list, area);
}

/// Render empty state message
fn render_empty_state(frame: &mut Frame, area: Rect, message: &str, is_loading: bool) {
    let text = if is_loading {
        "Loading..."
    } else {
        message
    };

    let paragraph = Paragraph::new(text)
        .style(styles::text_dim())
        .alignment(Alignment::Center);

    // Center the message
    let inner = Block::default().borders(Borders::ALL).inner(area);
    let y = inner.y + inner.height / 2;
    let centered = Rect::new(inner.x, y, inner.width, 1);

    frame.render_widget(paragraph, centered);
}

/// Render the form modal
fn render_form_modal(frame: &mut Frame, app: &App, area: Rect) {
    let form = match &app.form_state {
        Some(f) => f,
        None => return,
    };

    // Determine form size based on type
    // Heights calculated as: fields * 3 + spacer(1) + buttons(1) + margin(2) + borders(2)
    let (popup_width, popup_height) = match form.form_type {
        FormType::CreateClient | FormType::EditClient(_) => (50, 12),
        FormType::CreateProject | FormType::EditProject(_) => (55, 22), // 5 fields
        FormType::CreateUser | FormType::EditUser(_) => (50, 18), // 4 fields
    };

    let popup_area = centered_rect(popup_width, popup_height, area);

    // Dim background
    frame.render_widget(Clear, popup_area);

    // Form title
    let title = match &form.form_type {
        FormType::CreateClient => " New Client ",
        FormType::EditClient(_) => " Edit Client ",
        FormType::CreateProject => " New Project ",
        FormType::EditProject(_) => " Edit Project ",
        FormType::CreateUser => " New User ",
        FormType::EditUser(_) => " Edit User ",
    };

    let block = Block::default()
        .title(title)
        .title_style(styles::title())
        .borders(Borders::ALL)
        .border_style(styles::border_focused())
        .style(Style::default().bg(colors::BG_MEDIUM));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Render form fields
    match &form.form_type {
        FormType::CreateClient | FormType::EditClient(_) => {
            render_client_form(frame, form, inner);
        }
        FormType::CreateProject | FormType::EditProject(_) => {
            render_project_form(frame, form, app, inner);
        }
        FormType::CreateUser | FormType::EditUser(_) => {
            render_user_form(frame, form, inner);
        }
    }

    // Render error message if any
    if let Some(ref error) = form.error {
        let error_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
        let error_text = Paragraph::new(error.as_str())
            .style(styles::error())
            .alignment(Alignment::Center);
        frame.render_widget(error_text, error_area);
    }

    // Render mini calendar popup if a date field is focused
    if form.current_field().is_date_picker() {
        let date_str = match form.current_field() {
            FormField::ProjectStartDate => &form.project_start_date,
            FormField::ProjectEndDate => &form.project_end_date,
            _ => return,
        };
        render_mini_calendar(frame, date_str, area, popup_area);
    }
}

/// Render client form fields
fn render_client_form(frame: &mut Frame, form: &FormState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Address
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
        ])
        .margin(1)
        .split(area);

    // Name field
    render_text_field(
        frame,
        "Name:",
        &form.client_name,
        form.current_field() == FormField::ClientName,
        false,
        chunks[0],
    );

    // Address field
    render_text_field(
        frame,
        "Address:",
        &form.client_address,
        form.current_field() == FormField::ClientAddress,
        false,
        chunks[1],
    );

    // Buttons
    render_form_buttons(
        frame,
        form.current_field() == FormField::SubmitButton,
        form.current_field() == FormField::CancelButton,
        chunks[3],
    );
}

/// Render project form fields
fn render_project_form(frame: &mut Frame, form: &FormState, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Client
            Constraint::Length(3), // Manager
            Constraint::Length(3), // Start Date
            Constraint::Length(3), // End Date
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
        ])
        .margin(1)
        .split(area);

    // Name field
    render_text_field(
        frame,
        "Name:",
        &form.project_name,
        form.current_field() == FormField::ProjectName,
        false,
        chunks[0],
    );

    // Client selector
    let client_name = app.clients
        .get(form.project_client_idx)
        .map(|c| c.display_name().to_string())
        .unwrap_or_else(|| "(Select client)".to_string());
    render_selector_field(
        frame,
        "Client:",
        &client_name,
        form.current_field() == FormField::ProjectClient,
        chunks[1],
    );

    // Manager selector
    let manager_name = app.users
        .get(form.project_manager_idx)
        .map(|u| u.display_name().to_string())
        .unwrap_or_else(|| "(Select manager)".to_string());
    render_selector_field(
        frame,
        "Manager:",
        &manager_name,
        form.current_field() == FormField::ProjectManager,
        chunks[2],
    );

    // Start Date field (date picker)
    render_date_picker_field(
        frame,
        "Start Date:",
        &form.project_start_date,
        form.current_field() == FormField::ProjectStartDate,
        chunks[3],
    );

    // End Date field (date picker)
    render_date_picker_field(
        frame,
        "End Date:",
        &form.project_end_date,
        form.current_field() == FormField::ProjectEndDate,
        chunks[4],
    );

    // Buttons
    render_form_buttons(
        frame,
        form.current_field() == FormField::SubmitButton,
        form.current_field() == FormField::CancelButton,
        chunks[6],
    );
}

/// Render user form fields
fn render_user_form(frame: &mut Frame, form: &FormState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Login
            Constraint::Length(3), // Password
            Constraint::Length(3), // Role
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
        ])
        .margin(1)
        .split(area);

    // Name field
    render_text_field(
        frame,
        "Name:",
        &form.user_name,
        form.current_field() == FormField::UserName,
        false,
        chunks[0],
    );

    // Login field
    render_text_field(
        frame,
        "Login:",
        &form.user_login,
        form.current_field() == FormField::UserLogin,
        false,
        chunks[1],
    );

    // Password field (masked)
    render_text_field(
        frame,
        "Password:",
        &form.user_password,
        form.current_field() == FormField::UserPassword,
        true,
        chunks[2],
    );

    // Role selector
    render_selector_field(
        frame,
        "Role:",
        &form.user_role.to_string(),
        form.current_field() == FormField::UserRole,
        chunks[3],
    );

    // Buttons
    render_form_buttons(
        frame,
        form.current_field() == FormField::SubmitButton,
        form.current_field() == FormField::CancelButton,
        chunks[5],
    );
}

/// Render a text input field
fn render_text_field(
    frame: &mut Frame,
    label: &str,
    value: &str,
    is_focused: bool,
    is_password: bool,
    area: Rect,
) {
    // Use 14 characters for label column to accommodate "Start Date:" and "End Date:" with padding
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(10)])
        .split(area);

    // Label
    let label_text = Paragraph::new(label)
        .style(styles::form_label())
        .alignment(Alignment::Right);
    frame.render_widget(label_text, chunks[0]);

    // Input field
    let display_value = if is_password {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    let input_style = if is_focused {
        styles::form_input_focused()
    } else {
        styles::form_input()
    };

    let cursor = if is_focused { "█" } else { "" };
    let input = Paragraph::new(format!(" {}{}", display_value, cursor))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    styles::border_focused()
                } else {
                    styles::border_dim()
                }),
        );
    frame.render_widget(input, chunks[1]);
}

/// Render a date picker field with mini calendar
fn render_date_picker_field(
    frame: &mut Frame,
    label: &str,
    value: &str,
    is_focused: bool,
    area: Rect,
) {
    // Use 14 characters for label column to match text fields
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(10)])
        .split(area);

    // Label
    let label_text = Paragraph::new(label)
        .style(styles::form_label())
        .alignment(Alignment::Right);
    frame.render_widget(label_text, chunks[0]);

    // Date picker display with navigation hints
    let input_style = if is_focused {
        styles::form_input_focused()
    } else {
        styles::form_input()
    };

    // Show navigation hints when focused, plus calendar icon
    let hint = if is_focused { " ◀-7 ▲+1 ▼-1 +7▶" } else { "" };
    let calendar_icon = "📅";
    let display = format!(" {} {}{}", calendar_icon, value, hint);

    let input = Paragraph::new(display)
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    styles::border_focused()
                } else {
                    styles::border_dim()
                }),
        );
    frame.render_widget(input, chunks[1]);
}

/// Render a selector/dropdown field
fn render_selector_field(
    frame: &mut Frame,
    label: &str,
    value: &str,
    is_focused: bool,
    area: Rect,
) {
    // Use 14 characters for label column to match text fields
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(10)])
        .split(area);

    // Label
    let label_text = Paragraph::new(label)
        .style(styles::form_label())
        .alignment(Alignment::Right);
    frame.render_widget(label_text, chunks[0]);

    // Selector display with arrows indicators
    let input_style = if is_focused {
        styles::form_input_focused()
    } else {
        styles::form_input()
    };

    let arrows = if is_focused { " ▲▼" } else { " ▼" };
    let input = Paragraph::new(format!(" {}{}", value, arrows))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if is_focused {
                    styles::border_focused()
                } else {
                    styles::border_dim()
                }),
        );
    frame.render_widget(input, chunks[1]);
}

/// Render form buttons
fn render_form_buttons(
    frame: &mut Frame,
    save_focused: bool,
    cancel_focused: bool,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(12),
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Save button
    let save_style = if save_focused {
        styles::button_focused()
    } else {
        styles::button()
    };
    let save_btn = Paragraph::new("  [ Save ]  ")
        .style(save_style)
        .alignment(Alignment::Center);
    frame.render_widget(save_btn, chunks[1]);

    // Cancel button
    let cancel_style = if cancel_focused {
        styles::button_danger()
    } else {
        styles::button()
    };
    let cancel_btn = Paragraph::new(" [ Cancel ] ")
        .style(cancel_style)
        .alignment(Alignment::Center);
    frame.render_widget(cancel_btn, chunks[3]);
}

/// Render confirmation dialog
fn render_confirm_dialog(frame: &mut Frame, app: &App, area: Rect) {
    let dialog = match &app.confirm_dialog {
        Some(d) => d,
        None => return,
    };

    let popup_area = centered_rect(45, 10, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {} ", dialog.title))
        .title_style(Style::default().fg(colors::RED).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::RED))
        .style(Style::default().bg(colors::BG_MEDIUM));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
        ])
        .margin(1)
        .split(inner);

    // Message
    let message = Paragraph::new(dialog.message.as_str())
        .style(styles::text())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(message, chunks[0]);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Percentage(10),
            Constraint::Length(10),
            Constraint::Percentage(25),
        ])
        .split(chunks[2]);

    let no_style = if !dialog.yes_focused {
        styles::button_focused()
    } else {
        styles::button()
    };
    let no_btn = Paragraph::new("  [ No ]  ")
        .style(no_style)
        .alignment(Alignment::Center);
    frame.render_widget(no_btn, button_chunks[1]);

    let yes_style = if dialog.yes_focused {
        styles::button_danger()
    } else {
        styles::button()
    };
    let yes_btn = Paragraph::new(" [ Yes ]  ")
        .style(yes_style)
        .alignment(Alignment::Center);
    frame.render_widget(yes_btn, button_chunks[3]);
}

/// Render error popup
fn render_error_popup(frame: &mut Frame, app: &App, area: Rect) {
    let popup = app.error_popup.as_ref().unwrap();

    let popup_width = (area.width * 60 / 100).min(60).max(30);
    let popup_height = 7;

    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Render the popup
    let block = Block::default()
        .title(format!(" {} ", popup.title))
        .title_style(
            Style::default()
                .fg(Color::White)
                .bg(colors::RED)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::RED))
        .style(Style::default().bg(Color::Rgb(0x2A, 0x18, 0x18)));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = Paragraph::new(popup.message.as_str())
        .style(styles::text())
        .wrap(Wrap { trim: true });

    frame.render_widget(text, inner);

    // Dismiss hint
    let hint = Paragraph::new("Press ESC or ENTER to dismiss")
        .style(styles::text_hint())
        .alignment(Alignment::Center);

    let hint_area = Rect::new(
        popup_area.x,
        popup_area.y + popup_area.height - 1,
        popup_area.width,
        1,
    );
    frame.render_widget(hint, hint_area);
}

/// Render help overlay
fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_width = 60;
    let popup_height = 29;
    let popup_area = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(colors::BLUE)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(colors::PURPLE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Tab/Shift+Tab ", Style::default().fg(colors::BLUE)),
            Span::raw("Switch tabs / form fields"),
        ]),
        Line::from(vec![
            Span::styled("  j/k or Up/Down", Style::default().fg(colors::BLUE)),
            Span::raw("Move up/down in lists"),
        ]),
        Line::from(vec![
            Span::styled("  h/l or Left/Right", Style::default().fg(colors::BLUE)),
            Span::raw("Scroll timeline"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("CRUD Operations", Style::default().fg(colors::PURPLE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  c             ", Style::default().fg(colors::BLUE)),
            Span::raw("Create new item"),
        ]),
        Line::from(vec![
            Span::styled("  e             ", Style::default().fg(colors::BLUE)),
            Span::raw("Edit selected item"),
        ]),
        Line::from(vec![
            Span::styled("  d / Delete    ", Style::default().fg(colors::BLUE)),
            Span::raw("Delete selected item"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Form Editing", Style::default().fg(colors::PURPLE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Tab           ", Style::default().fg(colors::BLUE)),
            Span::raw("Move to next field"),
        ]),
        Line::from(vec![
            Span::styled("  Up/Down       ", Style::default().fg(colors::BLUE)),
            Span::raw("Change dropdown/date (+/-1 day)"),
        ]),
        Line::from(vec![
            Span::styled("  Left/Right    ", Style::default().fg(colors::BLUE)),
            Span::raw("Date picker: +/-7 days"),
        ]),
        Line::from(vec![
            Span::styled("  Type text     ", Style::default().fg(colors::BLUE)),
            Span::raw("Edit text fields directly"),
        ]),
        Line::from(vec![
            Span::styled("  Enter         ", Style::default().fg(colors::BLUE)),
            Span::raw("Next field / Submit on button"),
        ]),
        Line::from(vec![
            Span::styled("  Esc           ", Style::default().fg(colors::BLUE)),
            Span::raw("Cancel / Close form"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("General", Style::default().fg(colors::PURPLE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  r             ", Style::default().fg(colors::BLUE)),
            Span::raw("Refresh data"),
        ]),
        Line::from(vec![
            Span::styled("  p             ", Style::default().fg(colors::BLUE)),
            Span::raw("Toggle particles"),
        ]),
        Line::from(vec![
            Span::styled("  q/Ctrl+C      ", Style::default().fg(colors::BLUE)),
            Span::raw("Quit"),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(styles::title())
                .borders(Borders::ALL)
                .border_style(styles::border())
                .style(Style::default().bg(colors::BG_MEDIUM)),
        )
        .style(styles::text());

    frame.render_widget(paragraph, popup_area);
}

/// Helper to create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Render a mini calendar popup next to the form
fn render_mini_calendar(frame: &mut Frame, date_str: &str, screen_area: Rect, form_area: Rect) {
    // Parse the date string
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    // Calendar dimensions
    let cal_width = 24;
    let cal_height = 10;

    // Position calendar to the right of the form if space, otherwise to the left
    let cal_x = if form_area.x + form_area.width + cal_width + 2 < screen_area.width {
        form_area.x + form_area.width + 1
    } else if form_area.x >= cal_width + 2 {
        form_area.x - cal_width - 1
    } else {
        // Center below
        (screen_area.width.saturating_sub(cal_width)) / 2
    };

    let cal_y = form_area.y + 2;
    let cal_area = Rect::new(
        cal_x,
        cal_y.min(screen_area.height.saturating_sub(cal_height)),
        cal_width,
        cal_height,
    );

    frame.render_widget(Clear, cal_area);

    // Build calendar lines
    let month_names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    let month_name = month_names[date.month0() as usize];
    let year = date.year();

    // Get first day of month and number of days
    let first_of_month = NaiveDate::from_ymd_opt(year, date.month(), 1).unwrap();
    let days_in_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, date.month() + 1, 1)
    }.unwrap().pred_opt().unwrap().day();

    // Day of week for first day (0 = Monday, 6 = Sunday)
    let first_weekday = first_of_month.weekday().num_days_from_monday() as usize;

    let mut lines = Vec::new();

    // Header with month/year
    let header = format!("{} {}", month_name, year);
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:^22}", header),
            Style::default().fg(colors::BLUE).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Day of week headers
    lines.push(Line::from(vec![
        Span::styled(" Mo Tu We Th Fr ", styles::text_dim()),
        Span::styled("Sa ", Style::default().fg(colors::BLUE)),
        Span::styled("Su", Style::default().fg(colors::RED)),
    ]));

    // Build week rows
    let mut day = 1u32;
    let selected_day = date.day();
    let today = chrono::Local::now().date_naive();
    let today_day = if today.year() == year && today.month() == date.month() {
        Some(today.day())
    } else {
        None
    };

    for week in 0..6 {
        let mut spans = Vec::new();
        spans.push(Span::raw(" "));

        for weekday in 0..7 {
            if (week == 0 && weekday < first_weekday) || day > days_in_month {
                spans.push(Span::raw("   "));
            } else {
                let is_selected = day == selected_day;
                let is_today = today_day == Some(day);
                let is_weekend = weekday >= 5;

                let style = if is_selected {
                    Style::default()
                        .fg(colors::BG_DARK)
                        .bg(colors::BLUE)
                        .add_modifier(Modifier::BOLD)
                } else if is_today {
                    Style::default()
                        .fg(colors::YELLOW)
                        .add_modifier(Modifier::BOLD)
                } else if is_weekend {
                    if weekday == 5 {
                        Style::default().fg(colors::BLUE)
                    } else {
                        Style::default().fg(colors::RED)
                    }
                } else {
                    styles::text()
                };

                spans.push(Span::styled(format!("{:2} ", day), style));
                day += 1;
            }
        }

        if day <= days_in_month || week == 0 {
            lines.push(Line::from(spans));
        }
    }

    // Instructions
    lines.push(Line::from(vec![
        Span::styled("▲▼±1d  ◀▶±7d", styles::text_hint()),
    ]));

    let calendar = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Calendar ")
                .title_style(styles::title())
                .borders(Borders::ALL)
                .border_style(styles::border_focused())
                .style(Style::default().bg(colors::BG_MEDIUM)),
        );

    frame.render_widget(calendar, cal_area);
}

/// Calculate the number of projects (completed/total) for a client
fn calculate_client_project_counts(projects: &[crate::models::ProjectDto], client_id: uuid::Uuid) -> (i32, i32) {
    let client_projects: Vec<_> = projects.iter().filter(|p| p.client_id == client_id).collect();
    let total = client_projects.len() as i32;
    let completed = client_projects.iter().filter(|p| p.is_completed()).count() as i32;
    (completed, total)
}
