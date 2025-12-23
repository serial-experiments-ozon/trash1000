//! Timeline widget for project visualization.
//!
//! This module implements a custom horizontal Gantt chart widget
//! that renders projects as colored bars on a time axis.
//! Uses Kanagawa Dragon theme colors.

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

/// Timeline widget state
#[derive(Debug, Clone)]
pub struct TimelineState {
    /// Current scroll offset in days from the start of the timeline
    pub scroll_offset: i64,
    /// Selected project index
    pub selected_project: Option<usize>,
    /// Zoom level (days per column)
    pub days_per_column: f64,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            selected_project: None,
            days_per_column: 1.0,
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

    /// Calculate the start date of the timeline
    fn calculate_timeline_start(&self, projects: &[ProjectDto]) -> NaiveDate {
        projects
            .iter()
            .map(|p| p.start_date)
            .min()
            .unwrap_or_else(|| chrono::Local::now().date_naive() - Duration::days(30))
    }
}

/// Timeline widget for rendering the Gantt chart
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
            title: " Project Timeline ",
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

    /// Convert a date to a column position
    fn date_to_column(&self, date: NaiveDate, start: NaiveDate, width: u16) -> Option<u16> {
        let days_from_start = (date - start).num_days();
        let days_with_offset = days_from_start - self.state.scroll_offset;
        let column = (days_with_offset as f64 / self.state.days_per_column) as i64;

        if column >= 0 && column < width as i64 {
            Some(column as u16)
        } else {
            None
        }
    }

    /// Render the time axis (header)
    fn render_time_axis(&self, area: Rect, buf: &mut Buffer, start: NaiveDate) {
        let style = styles::text_dim();
        let month_style = Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD);

        // Draw month markers
        for col in 0..area.width {
            let days_offset = self.state.scroll_offset + (col as f64 * self.state.days_per_column) as i64;
            let date = start + Duration::days(days_offset);

            if date.day() == 1 {
                // Month start - show month name
                let month_name = date.format("%b").to_string();
                if col + month_name.len() as u16 <= area.width {
                    buf.set_string(area.x + col, area.y, &month_name, month_style);
                }
            } else if date.day() % 7 == 0 && col > 0 {
                // Weekly marker
                let day_str = date.format("%d").to_string();
                if col + 2 <= area.width {
                    buf.set_string(area.x + col, area.y, &day_str, style);
                }
            }
        }

        // Draw axis line
        for col in 0..area.width {
            let pos = (area.x + col, area.y + 1);
            buf[pos].set_char('─');
            buf[pos].set_style(Style::default().fg(colors::BORDER_DIM));
        }
    }

    /// Render a single project bar
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
        let name_width = 20.min(area.width.saturating_sub(1) as usize);

        // Render project name (left column)
        let name = project.display_name();
        let display_name: String = if name.len() > name_width {
            format!("{}…", &name[..name_width - 1])
        } else {
            format!("{:width$}", name, width = name_width)
        };

        let name_style = if is_selected {
            Style::default()
                .fg(colors::BG_DARK)
                .bg(color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color)
        };

        buf.set_string(area.x, area.y + row, &display_name, name_style);

        // Calculate bar positions
        let bar_area_start = area.x + name_width as u16 + 2;
        let bar_area_width = area.width.saturating_sub(name_width as u16 + 3);

        if bar_area_width == 0 {
            return;
        }

        // Draw the project bar
        let project_start_col = self.date_to_column(project.start_date, start, bar_area_width);
        let project_end_date = project.actual_end_date.unwrap_or(project.planned_end_date);
        let project_end_col = self.date_to_column(project_end_date, start, bar_area_width);

        // Determine bar style based on project status
        let bar_style = if project.is_completed() {
            Style::default().fg(colors::PROJECT_COMPLETED)
        } else if project.is_overdue() {
            Style::default().fg(colors::PROJECT_OVERDUE)
        } else {
            Style::default().fg(color)
        };

        // Draw the bar
        if let (Some(start_col), Some(end_col)) = (project_start_col, project_end_col) {
            for col in start_col..=end_col.min(bar_area_width - 1) {
                let pos = (bar_area_start + col, area.y + row);
                if col == start_col {
                    buf[pos].set_char(BLOCK_LEFT);
                } else if col == end_col {
                    buf[pos].set_char(BLOCK_RIGHT);
                } else {
                    buf[pos].set_char(BLOCK_FULL);
                }
                buf[pos].set_style(bar_style);
            }
        } else if let Some(start_col) = project_start_col {
            // Project extends beyond visible area
            for col in start_col..bar_area_width {
                let pos = (bar_area_start + col, area.y + row);
                if col == start_col {
                    buf[pos].set_char(BLOCK_LEFT);
                } else {
                    buf[pos].set_char(BLOCK_FULL);
                }
                buf[pos].set_style(bar_style);
            }
        } else if let Some(end_col) = project_end_col {
            // Project started before visible area
            for col in 0..=end_col.min(bar_area_width - 1) {
                let pos = (bar_area_start + col, area.y + row);
                if col == end_col {
                    buf[pos].set_char(BLOCK_RIGHT);
                } else {
                    buf[pos].set_char(BLOCK_FULL);
                }
                buf[pos].set_style(bar_style);
            }
        }

        // Draw today marker
        let today = chrono::Local::now().date_naive();
        if let Some(today_col) = self.date_to_column(today, start, bar_area_width) {
            let pos = (bar_area_start + today_col, area.y + row);
            if buf[pos].symbol() == " " {
                buf[pos].set_char('│');
                buf[pos].set_style(Style::default().fg(colors::TODAY_MARKER));
            }
        }
    }

    /// Render the "today" vertical line
    fn render_today_line(&self, area: Rect, buf: &mut Buffer, start: NaiveDate, name_width: u16) {
        let today = chrono::Local::now().date_naive();
        let bar_area_start = area.x + name_width + 2;
        let bar_area_width = area.width.saturating_sub(name_width + 3);

        if let Some(today_col) = self.date_to_column(today, start, bar_area_width) {
            for row in 0..area.height {
                let pos = (bar_area_start + today_col, area.y + row);
                if buf[pos].symbol() == " " {
                    buf[pos].set_char('│');
                    buf[pos].set_style(Style::default().fg(colors::TODAY_MARKER).add_modifier(Modifier::DIM));
                }
            }
        }
    }
}

impl Widget for TimelineWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw the block border
        let block = Block::default()
            .title(self.title)
            .title_style(styles::title_accent())
            .borders(Borders::ALL)
            .border_style(styles::border())
            .style(Style::default().bg(colors::BG_DARK));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 25 || inner.height < 3 {
            return; // Too small to render
        }

        let start = self.calculate_timeline_start();

        // Render time axis (top 2 rows)
        if inner.height >= 3 {
            self.render_time_axis(
                Rect::new(inner.x + 22, inner.y, inner.width.saturating_sub(23), 2),
                buf,
                start,
            );
        }

        // Render today vertical line
        self.render_today_line(inner, buf, start, 20);

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

        // Render scroll hints
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

/// Status information widget for the timeline
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
        let zoom_level = format!("Zoom: {:.1}d/col", self.state.days_per_column);
        let project_info = format!("Projects: {}", self.project_count);
        let selected_info = self
            .state
            .selected_project
            .map(|i| format!("Selected: {}", i + 1))
            .unwrap_or_else(|| "None selected".to_string());

        let status = format!("{} | {} | {}", project_info, selected_info, zoom_level);

        buf.set_string(
            area.x,
            area.y,
            &status,
            styles::text_dim(),
        );
    }
}
