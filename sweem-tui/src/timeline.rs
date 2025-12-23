//! Timeline widget for project visualization.
//!
//! A completely rewritten, modern Gantt chart widget with:
//! - Rainbow-colored project bars with gradient effects
//! - Visual indicators for project status (completed, overdue, active)
//! - Smooth block character rendering with sparkle effects
//! - Animated elements and modern aesthetic
//! - "Goyslop" killer feature: flashy visual effects

#![allow(dead_code)]

use chrono::{Datelike, Duration, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::models::ProjectDto;
use crate::theme::{colors, styles, get_project_color};

/// Unicode block characters for smooth rendering
const BLOCK_FULL: char = '█';
const BLOCK_LEFT: char = '▌';
const BLOCK_RIGHT: char = '▐';
const BLOCK_TOP: char = '▀';
const BLOCK_BOTTOM: char = '▄';
const BLOCK_LIGHT: char = '░';
const BLOCK_MEDIUM: char = '▒';
const BLOCK_DARK: char = '▓';

/// Status indicators for projects
const STATUS_COMPLETED: char = '✓';
const STATUS_OVERDUE: char = '!';
const STATUS_ACTIVE: char = '●';
const SPARKLE_CHARS: [char; 4] = ['✦', '✧', '⋆', '★'];

/// Modern border characters
const BORDER_TL: char = '╭';
const BORDER_TR: char = '╮';
const BORDER_BL: char = '╰';
const BORDER_BR: char = '╯';
const BORDER_H: char = '─';
const BORDER_V: char = '│';

/// Timeline widget state
#[derive(Debug, Clone)]
pub struct TimelineState {
    /// Current scroll offset in days from the start of the timeline
    pub scroll_offset: i64,
    /// Selected project index
    pub selected_project: Option<usize>,
    /// Zoom level (days per column)
    pub days_per_column: f64,
    /// Animation frame counter for effects
    pub animation_frame: u64,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            selected_project: None,
            days_per_column: 1.0,
            animation_frame: 0,
        }
    }
}

impl TimelineState {
    /// Scroll left (earlier in time)
    pub fn scroll_left(&mut self, amount: i64) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Scroll right (later in time)
    pub fn scroll_right(&mut self, amount: i64) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    /// Move selection up
    pub fn select_previous(&mut self, total: usize) {
        if total == 0 {
            self.selected_project = None;
            return;
        }
        self.selected_project = Some(match self.selected_project {
            Some(i) if i > 0 => i - 1,
            Some(_) => total - 1,
            None => 0,
        });
    }

    /// Move selection down
    pub fn select_next(&mut self, total: usize) {
        if total == 0 {
            self.selected_project = None;
            return;
        }
        self.selected_project = Some(match self.selected_project {
            Some(i) if i < total - 1 => i + 1,
            Some(_) => 0,
            None => 0,
        });
    }

    /// Zoom in (fewer days per column)
    pub fn zoom_in(&mut self) {
        if self.days_per_column > 0.25 {
            self.days_per_column /= 2.0;
        }
    }

    /// Zoom out (more days per column)
    pub fn zoom_out(&mut self) {
        if self.days_per_column < 14.0 {
            self.days_per_column *= 2.0;
        }
    }

    /// Center the timeline on today
    pub fn center_on_today(&mut self, width: u16) {
        let today = chrono::Local::now().date_naive();
        let start = self.calculate_timeline_start(&[]);
        let days_from_start = (today - start).num_days();
        let center_offset = (width as i64 / 2) * self.days_per_column as i64;
        self.scroll_offset = (days_from_start - center_offset).max(0);
    }

    /// Jump to and center on a specific project
    pub fn jump_to_project(&mut self, project: &ProjectDto, projects: &[ProjectDto], viewport_width: u16) {
        let timeline_start = self.calculate_timeline_start(projects);

        // Calculate the middle of the project's duration
        let project_end = project.actual_end_date.unwrap_or(project.planned_end_date);
        let project_mid = project.start_date + Duration::days(
            (project_end - project.start_date).num_days() / 2
        );

        // Calculate how many days from timeline start to project middle
        let days_from_start = (project_mid - timeline_start).num_days();

        // Center the viewport on this point
        // Account for the name column (about 26 chars) in the viewport
        let effective_width = viewport_width.saturating_sub(26) as i64;
        let center_offset = (effective_width / 2) * self.days_per_column as i64;

        self.scroll_offset = (days_from_start - center_offset).max(0);
    }

    /// Calculate the start date of the timeline
    fn calculate_timeline_start(&self, projects: &[ProjectDto]) -> NaiveDate {
        projects
            .iter()
            .map(|p| p.start_date)
            .min()
            .unwrap_or_else(|| chrono::Local::now().date_naive() - Duration::days(30))
    }

    /// Advance animation frame
    pub fn tick(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
    }
}

/// Modern timeline widget for rendering the Gantt chart
pub struct TimelineWidget<'a> {
    projects: &'a [ProjectDto],
    state: &'a TimelineState,
    title: &'a str,
}

impl<'a> TimelineWidget<'a> {
    pub fn new(projects: &'a [ProjectDto], state: &'a TimelineState) -> Self {
        Self {
            projects,
            state,
            title: " ✨ Project Timeline ",
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Calculate the timeline start date
    fn calculate_timeline_start(&self) -> NaiveDate {
        self.projects
            .iter()
            .map(|p| p.start_date)
            .min()
            .unwrap_or_else(|| chrono::Local::now().date_naive() - Duration::days(30))
    }

    /// Convert a date to a column position (returns i64 for full range)
    fn date_to_column_raw(&self, date: NaiveDate, start: NaiveDate) -> i64 {
        let days_from_start = (date - start).num_days();
        let days_with_offset = days_from_start - self.state.scroll_offset;
        (days_with_offset as f64 / self.state.days_per_column) as i64
    }

    /// Convert a date to a visible column position (clamped to viewport)
    fn date_to_column(&self, date: NaiveDate, start: NaiveDate, width: u16) -> Option<u16> {
        let column = self.date_to_column_raw(date, start);

        if column >= 0 && column < width as i64 {
            Some(column as u16)
        } else {
            None
        }
    }

    /// Render the modern time axis (header)
    fn render_time_axis(&self, area: Rect, buf: &mut Buffer, start: NaiveDate) {
        let today = chrono::Local::now().date_naive();

        // Draw month markers and day numbers
        for col in 0..area.width {
            let days_offset = self.state.scroll_offset + (col as f64 * self.state.days_per_column) as i64;
            let date = start + Duration::days(days_offset);

            if date.day() == 1 {
                // Month start - show month name with modern styling
                let month_name = date.format("%b").to_string();
                let month_style = Style::default()
                    .fg(colors::PURPLE)
                    .add_modifier(Modifier::BOLD);
                if col + month_name.len() as u16 <= area.width {
                    buf.set_string(area.x + col, area.y, &month_name, month_style);
                }
            } else if date.day() % 7 == 0 && col > 0 {
                // Weekly marker - subtle
                let day_str = date.format("%d").to_string();
                if col + 2 <= area.width {
                    buf.set_string(area.x + col, area.y, &day_str, styles::text_dim());
                }
            }
        }

        // Draw modern axis line with gradient effect
        for col in 0..area.width {
            let pos = (area.x + col, area.y + 1);
            let days_offset = self.state.scroll_offset + (col as f64 * self.state.days_per_column) as i64;
            let date = start + Duration::days(days_offset);

            // Special styling for today's column
            let char_and_style = if date == today {
                ('▼', Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD))
            } else if date.weekday().num_days_from_monday() >= 5 {
                // Weekend - dimmer
                ('┄', Style::default().fg(colors::BORDER_DIM))
            } else {
                ('─', Style::default().fg(colors::BORDER))
            };

            buf[pos].set_char(char_and_style.0);
            buf[pos].set_style(char_and_style.1);
        }
    }

    /// Render a single project bar with modern styling and goyslop effects
    fn render_project_bar(
        &self,
        area: Rect,
        buf: &mut Buffer,
        project: &ProjectDto,
        index: usize,
        start: NaiveDate,
        row: u16,
        is_selected: bool,
    ) {
        let color = get_project_color(index);
        let name_width = 24.min(area.width.saturating_sub(1) as usize);

        // Status indicator with animation for selected items
        let status_char = if project.is_completed() {
            STATUS_COMPLETED
        } else if project.is_overdue() {
            STATUS_OVERDUE
        } else {
            STATUS_ACTIVE
        };

        let status_color = if project.is_completed() {
            colors::GREEN
        } else if project.is_overdue() {
            colors::RED
        } else {
            color
        };

        // Animated sparkle effect for selected row (goyslop!)
        let sparkle_idx = (self.state.animation_frame / 4) as usize % SPARKLE_CHARS.len();
        let prefix_char = if is_selected {
            SPARKLE_CHARS[sparkle_idx]
        } else {
            '│'
        };

        // Render row prefix with color indicator
        let prefix_style = Style::default().fg(if is_selected { colors::YELLOW } else { color });
        buf.set_string(area.x, area.y + row, &prefix_char.to_string(), prefix_style);

        // Status indicator
        let status_style = Style::default()
            .fg(status_color)
            .add_modifier(if is_selected { Modifier::BOLD | Modifier::SLOW_BLINK } else { Modifier::BOLD });
        buf.set_string(area.x + 1, area.y + row, &status_char.to_string(), status_style);
        buf.set_string(area.x + 2, area.y + row, " ", Style::default());

        // Render project name (left column) with modern styling
        let name = project.display_name();
        let display_name: String = if name.len() > name_width - 4 {
            format!("{}…", &name[..name_width - 5])
        } else {
            format!("{:width$}", name, width = name_width - 4)
        };

        let name_style = if is_selected {
            Style::default()
                .fg(colors::BG_DARK)
                .bg(color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors::FG_PRIMARY)
        };

        buf.set_string(area.x + 3, area.y + row, &display_name, name_style);

        // Calculate bar positions
        let bar_area_start = area.x + name_width as u16 + 2;
        let bar_area_width = area.width.saturating_sub(name_width as u16 + 3);

        if bar_area_width == 0 {
            return;
        }

        // Draw the project bar with gradient effect
        let project_end_date = project.actual_end_date.unwrap_or(project.planned_end_date);

        // Get raw column positions (can be negative or beyond width)
        let start_col_raw = self.date_to_column_raw(project.start_date, start);
        let end_col_raw = self.date_to_column_raw(project_end_date, start);

        // Check if project is visible at all
        if end_col_raw < 0 || start_col_raw >= bar_area_width as i64 {
            return;
        }

        // Calculate visible portion of the bar
        let visible_start = start_col_raw.max(0).min(bar_area_width as i64 - 1) as u16;
        let visible_end = end_col_raw.max(0).min(bar_area_width as i64 - 1) as u16;

        if visible_end < visible_start {
            return;
        }

        // Draw the bar with gradient-like effect (goyslop!)
        let bar_length = (visible_end - visible_start + 1).max(1);

        for col in visible_start..=visible_end {
            let pos = (bar_area_start + col, area.y + row);
            let is_start = col as i64 == start_col_raw;
            let is_end = col as i64 == end_col_raw;

            // Calculate relative position for gradient effect
            let relative_pos = if bar_length > 1 {
                (col - visible_start) as f32 / (bar_length - 1) as f32
            } else {
                0.5
            };

            // Create gradient character based on position
            let bar_char = if is_start && !is_end {
                BLOCK_LEFT
            } else if is_end && !is_start {
                BLOCK_RIGHT
            } else if is_selected {
                // Animated pattern for selected (goyslop!)
                let anim_offset = (self.state.animation_frame / 2) as u16 % 4;
                if (col + anim_offset) % 2 == 0 { BLOCK_FULL } else { BLOCK_DARK }
            } else {
                BLOCK_FULL
            };

            // Color based on status with gradient
            let bar_color = if project.is_completed() {
                Self::blend_colors(color, colors::GREEN, 0.4 + relative_pos * 0.2)
            } else if project.is_overdue() {
                // Pulsing red effect for overdue (goyslop!)
                let pulse = ((self.state.animation_frame % 20) as f32 / 20.0 * std::f32::consts::PI).sin() * 0.3;
                Self::blend_colors(color, colors::RED, 0.5 + pulse)
            } else {
                // Active projects: subtle gradient from left to right
                if relative_pos < 0.15 || relative_pos > 0.85 {
                    Self::dim_color(color, 0.7)
                } else {
                    color
                }
            };

            let bar_style = if is_selected {
                Style::default()
                    .fg(bar_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(bar_color)
            };

            buf[pos].set_char(bar_char);
            buf[pos].set_style(bar_style);
        }

        // Draw today marker on top if it falls within this project
        let today = chrono::Local::now().date_naive();
        if let Some(today_col) = self.date_to_column(today, start, bar_area_width) {
            if today_col >= visible_start && today_col <= visible_end {
                let pos = (bar_area_start + today_col, area.y + row);
                buf[pos].set_char('│');
                buf[pos].set_style(Style::default()
                    .fg(colors::YELLOW)
                    .add_modifier(Modifier::BOLD));
            }
        }
    }

    /// Blend two colors together
    fn blend_colors(c1: ratatui::style::Color, c2: ratatui::style::Color, ratio: f32) -> ratatui::style::Color {
        use ratatui::style::Color;
        match (c1, c2) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let ratio = ratio.clamp(0.0, 1.0);
                let r = (r1 as f32 * (1.0 - ratio) + r2 as f32 * ratio) as u8;
                let g = (g1 as f32 * (1.0 - ratio) + g2 as f32 * ratio) as u8;
                let b = (b1 as f32 * (1.0 - ratio) + b2 as f32 * ratio) as u8;
                Color::Rgb(r, g, b)
            }
            _ => c1,
        }
    }

    /// Dim a color by a factor
    fn dim_color(c: ratatui::style::Color, factor: f32) -> ratatui::style::Color {
        use ratatui::style::Color;
        match c {
            Color::Rgb(r, g, b) => {
                Color::Rgb(
                    (r as f32 * factor) as u8,
                    (g as f32 * factor) as u8,
                    (b as f32 * factor) as u8,
                )
            }
            _ => c,
        }
    }

    /// Render the "today" vertical line with glow effect
    fn render_today_line(&self, area: Rect, buf: &mut Buffer, start: NaiveDate, name_width: u16) {
        let today = chrono::Local::now().date_naive();
        let bar_area_start = area.x + name_width + 2;
        let bar_area_width = area.width.saturating_sub(name_width + 3);

        if let Some(today_col) = self.date_to_column(today, start, bar_area_width) {
            for row in 0..area.height {
                let pos = (bar_area_start + today_col, area.y + row);
                if buf[pos].symbol() == " " {
                    // Animated glow effect for today line (goyslop!)
                    let glow_char = if (row + self.state.animation_frame as u16 / 3) % 3 == 0 {
                        '┃'
                    } else {
                        '│'
                    };
                    buf[pos].set_char(glow_char);
                    buf[pos].set_style(Style::default()
                        .fg(colors::TODAY_MARKER)
                        .add_modifier(Modifier::DIM));
                }
            }
        }
    }

    /// Render the modern legend with icons
    fn render_legend(&self, area: Rect, buf: &mut Buffer) {
        let legend_y = area.y + area.height - 1;
        let mut x = area.x + 6; // After scroll hint

        // Status legend with modern icons
        let legend_items = [
            (STATUS_ACTIVE, "Active", colors::BLUE),
            (STATUS_COMPLETED, "Done", colors::GREEN),
            (STATUS_OVERDUE, "Overdue", colors::RED),
            ('│', "Today", colors::YELLOW),
        ];

        for (icon, label, color) in legend_items {
            if x + label.len() as u16 + 4 > area.x + area.width - 6 {
                break;
            }

            buf.set_string(x, legend_y, &icon.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD));
            x += 1;
            buf.set_string(x, legend_y, label, styles::text_hint());
            x += label.len() as u16 + 2;
        }
    }
}

impl Widget for TimelineWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the modern block border with rounded corners
        let block = Block::default()
            .title(self.title)
            .title_style(Style::default()
                .fg(colors::PURPLE)
                .add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(styles::border())
            .style(Style::default().bg(colors::BG_DARK));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 30 || inner.height < 3 {
            return; // Too small to render
        }

        let start = self.calculate_timeline_start();
        let name_col_width: u16 = 26; // Color indicator (3) + name (21) + spacing (2)

        // Render time axis (top 2 rows)
        if inner.height >= 3 {
            self.render_time_axis(
                Rect::new(inner.x + name_col_width, inner.y, inner.width.saturating_sub(name_col_width + 1), 2),
                buf,
                start,
            );
        }

        // Render today vertical line
        self.render_today_line(inner, buf, start, name_col_width - 2);

        // Render project bars
        let projects_area = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(2));
        for (index, project) in self.projects.iter().enumerate() {
            if index >= projects_area.height as usize {
                break;
            }

            let is_selected = self.state.selected_project == Some(index);
            self.render_project_bar(
                projects_area,
                buf,
                project,
                index,
                start,
                index as u16,
                is_selected,
            );
        }

        // Render legend in bottom border
        if !self.projects.is_empty() {
            self.render_legend(area, buf);
        }

        // Render navigation hints with modern styling
        if self.state.scroll_offset > 0 {
            buf.set_string(
                area.x + 1,
                area.y + area.height - 1,
                "◀ h",
                styles::text_hint(),
            );
        }
        buf.set_string(
            area.x + area.width - 4,
            area.y + area.height - 1,
            "l ▶",
            styles::text_hint(),
        );
    }
}

/// Modern status information widget for the timeline
pub struct TimelineStatusWidget<'a> {
    state: &'a TimelineState,
    project_count: usize,
}

impl<'a> TimelineStatusWidget<'a> {
    pub fn new(state: &'a TimelineState, project_count: usize) -> Self {
        Self {
            state,
            project_count,
        }
    }
}

impl Widget for TimelineStatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let zoom_level = format!("⚲ {:.1}d/col", self.state.days_per_column);
        let project_info = format!("📊 {} projects", self.project_count);
        let selected_info = self
            .state
            .selected_project
            .map(|i| format!("▸ #{}", i + 1))
            .unwrap_or_else(|| "▸ none".to_string());

        let status = format!("{}  {}  {}", project_info, selected_info, zoom_level);

        buf.set_string(
            area.x,
            area.y,
            &status,
            Style::default().fg(colors::FG_DIM),
        );

        // Add sparkle animation at the end (goyslop!)
        let sparkle_idx = (self.state.animation_frame / 8) as usize % SPARKLE_CHARS.len();
        buf.set_string(
            area.x + status.len() as u16 + 2,
            area.y,
            &SPARKLE_CHARS[sparkle_idx].to_string(),
            Style::default().fg(colors::PURPLE),
        );
    }
}
