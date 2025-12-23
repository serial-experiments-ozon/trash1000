//! Project Radar Widget.
//!
//! Visualizes projects in polar coordinates.
//! Improvements: Client Labels, Distance Rings, Distinct Markers.

use std::f64::consts::PI;
use chrono::{Local, NaiveDate, Datelike};
use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Modifier, Style}, symbols::Marker, text::Span, widgets::{Widget, canvas::{Canvas, Circle, Context, Line}}
};
use uuid::Uuid;

use crate::{models::{ClientDto, ProjectDto}, theme::styles}; // Добавили ClientDto
use crate::theme::{colors, get_project_color};

/// Radar State
#[derive(Debug, Clone)]
pub struct RadarState {
    pub scan_angle: f64,
    pub selected_index: Option<usize>,
    pub range_days: f64,
}

impl Default for RadarState {
    fn default() -> Self {
        Self {
            scan_angle: 0.0,
            selected_index: None,
            range_days: 90.0, 
        }
    }
}

impl RadarState {
    pub fn tick(&mut self) {
        self.scan_angle += 0.05;
        if self.scan_angle > 2.0 * PI {
            self.scan_angle -= 2.0 * PI;
        }
    }

    pub fn select_next(&mut self, total: usize) {
        if total == 0 { return; }
        self.selected_index = Some(match self.selected_index {
            Some(i) => (i + 1) % total,
            None => 0,
        });
    }

    pub fn select_prev(&mut self, total: usize) {
        if total == 0 { return; }
        self.selected_index = Some(match self.selected_index {
            Some(i) => (i + total - 1) % total,
            None => 0,
        });
    }
    
    pub fn zoom_in(&mut self) {
        if self.range_days > 14.0 { self.range_days -= 7.0; }
    }
    
    pub fn zoom_out(&mut self) {
        if self.range_days < 365.0 { self.range_days += 7.0; }
    }

    pub fn center_on_today(&mut self, _projects: &[ProjectDto], _width: u16) {
        self.range_days = 90.0;
        self.scan_angle = 0.0;
    }

    pub fn jump_to_project(&mut self, project: &ProjectDto, projects: &[ProjectDto], _width: u16) {
        if let Some(idx) = projects.iter().position(|p| p.id == project.id) {
            self.selected_index = Some(idx);
        }
    }
}

pub struct RadarWidget<'a> {
    projects: &'a [ProjectDto],
    clients: &'a [ClientDto], // Добавили ссылку на клиентов для отображения имен
    state: &'a RadarState,
}

impl<'a> RadarWidget<'a> {
    pub fn new(projects: &'a [ProjectDto], clients: &'a [ClientDto], state: &'a RadarState) -> Self {
        Self { projects, clients, state }
    }

    fn get_project_coords(&self, project: &ProjectDto) -> (f64, f64) {
        let today = Local::now().date_naive();
        // Для радара используем planned_end_date, чтобы видеть дедлайн
        let target_date = project.planned_end_date;
        
        // Fix for "Year 1" bug
        if target_date.year() < 2000 {
             // Если дата сломана, кидаем в центр как "ошибку" или "просрочку"
             return (5.0, self.client_hash_to_angle(project.client_id));
        }

        let days_left = (target_date - today).num_days() as f64;
        
        // Map radius:
        // < 0 (Overdue) -> 0..15
        // 0..Range -> 15..90
        let r = if days_left < 0.0 {
             // Overdue: Closer to 0 means MORE overdue, but let's keep them in the "danger zone" (0-15)
             // Let's clamp to 5.0-15.0 range randomly or fixed
             10.0
        } else {
             // Future: 
             let pct = (days_left / self.state.range_days).clamp(0.0, 1.0);
             20.0 + (pct * 75.0)
        };

        let angle = self.client_hash_to_angle(project.client_id);
        (r, angle)
    }

    fn client_hash_to_angle(&self, id: Uuid) -> f64 {
        let bytes = id.as_bytes();
        let mut sum: u32 = 0;
        // Simple hash to spread clients around the circle
        for (i, b) in bytes.iter().enumerate() { 
            sum = sum.wrapping_add((*b as u32).wrapping_mul(i as u32 + 1)); 
        }
        (sum as f64 % 360.0).to_radians()
    }

    fn draw_radar(&self, ctx: &mut Context) {
        // --- 1. Grid & HUD ---
        // Outer rim
        ctx.draw(&Circle { x: 0.0, y: 0.0, radius: 95.0, color: colors::BORDER_DIM }); 
        
        // Mid range rings with labels
        ctx.draw(&Circle { x: 0.0, y: 0.0, radius: 57.5, color: colors::BG_HIGHLIGHT }); 
        ctx.print(60.0, 2.0, Span::styled(format!("{:.0}d", self.state.range_days / 2.0), Style::default().fg(colors::FG_HINT)));

        // Danger zone (Now)
        ctx.draw(&Circle { x: 0.0, y: 0.0, radius: 20.0, color: colors::RED_LIGHT }); 
        ctx.print(22.0, 2.0, Span::styled("NOW", Style::default().fg(colors::RED)));

        // Axis
        ctx.draw(&Line { x1: -100.0, y1: 0.0, x2: 100.0, y2: 0.0, color: colors::BG_HIGHLIGHT });
        ctx.draw(&Line { x1: 0.0, y1: -100.0, x2: 0.0, y2: 100.0, color: colors::BG_HIGHLIGHT });

        // --- 2. Client Sectors Labels ---
        // Draw client names at the edge based on their angle
        for client in self.clients {
            let angle = self.client_hash_to_angle(client.id);
            let label_r = 85.0; // Place inside outer rim
            let x = label_r * angle.cos();
            let y = label_r * angle.sin();
            
            // Shorten name
            let name = client.display_name();
            let short = if name.len() > 8 { &name[0..8] } else { name };
            
            ctx.print(x, y, Span::styled(short.to_string(), Style::default().fg(colors::BLUE_LIGHT).add_modifier(Modifier::DIM)));
            
            // Draw faint spoke line
            ctx.draw(&Line { 
                x1: 20.0 * angle.cos(), 
                y1: 20.0 * angle.sin(), 
                x2: 90.0 * angle.cos(), 
                y2: 90.0 * angle.sin(), 
                color: colors::BG_HIGHLIGHT 
            });
        }

        // --- 3. Scanline ---
        let scan_x = self.state.scan_angle.cos() * 95.0;
        let scan_y = self.state.scan_angle.sin() * 95.0;
        ctx.draw(&Line { x1: 0.0, y1: 0.0, x2: scan_x, y2: scan_y, color: colors::GREEN_LIGHT });

        // --- 4. Projects ---
        for (i, project) in self.projects.iter().enumerate() {
            let (r, theta) = self.get_project_coords(project);
            let x = r * theta.cos();
            let y = r * theta.sin();

            if r > 100.0 { continue; }

            let is_selected = self.state.selected_index == Some(i);
            
            let mut color = get_project_color(i);
            if project.is_completed() { 
                color = colors::GREEN; 
            } else if project.is_overdue() { 
                color = colors::RED; 
            } else if project.is_pending() {
                color = colors::FG_DIM; 
            }
            if is_selected { color = colors::FG_PRIMARY; }

            // Marker Shape Logic
            if project.is_completed() {
                // Square-ish (4 lines)
                let sz = 2.0;
                ctx.draw(&Line { x1: x-sz, y1: y-sz, x2: x+sz, y2: y-sz, color });
                ctx.draw(&Line { x1: x+sz, y1: y-sz, x2: x+sz, y2: y+sz, color });
                ctx.draw(&Line { x1: x+sz, y1: y+sz, x2: x-sz, y2: y+sz, color });
                ctx.draw(&Line { x1: x-sz, y1: y+sz, x2: x-sz, y2: y-sz, color });
            } else if project.is_overdue() {
                // Cross
                let sz = 2.0;
                ctx.draw(&Line { x1: x-sz, y1: y-sz, x2: x+sz, y2: y+sz, color });
                ctx.draw(&Line { x1: x-sz, y1: y+sz, x2: x+sz, y2: y-sz, color });
            } else {
                // Dot/Circle
                ctx.draw(&Circle { x, y, radius: 1.5, color });
            }

            // Selection Highlight
            if is_selected {
                // Line to center
                ctx.draw(&Line { x1: 0.0, y1: 0.0, x2: x, y2: y, color: colors::FG_DIM });
                
                // Brackets
                let b_sz = 4.0;
                let c = colors::YELLOW;
                // [ ] style brackets
                ctx.draw(&Line { x1: x-b_sz, y1: y-b_sz, x2: x-b_sz, y2: y+b_sz, color: c }); // Left
                ctx.draw(&Line { x1: x+b_sz, y1: y-b_sz, x2: x+b_sz, y2: y+b_sz, color: c }); // Right
                ctx.draw(&Line { x1: x-b_sz, y1: y-b_sz, x2: x-b_sz+2.0, y2: y-b_sz, color: c }); // Corners
                ctx.draw(&Line { x1: x+b_sz, y1: y+b_sz, x2: x+b_sz-2.0, y2: y+b_sz, color: c });

                if let Some(name) = project.name.clone() {
                    ctx.print(x + 5.0, y, Span::styled(name, Style::default().fg(colors::YELLOW).add_modifier(Modifier::BOLD)));
                }
            }
        }
    }
}

impl Widget for RadarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .block(ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(styles::border())
                .title(" Orbital Command ")
                .title_style(styles::title_accent())
                .style(Style::default().bg(colors::BG_DARK))
            )
            .x_bounds([-100.0, 100.0])
            .y_bounds([-100.0, 100.0])
            .marker(Marker::Braille)
            .paint(|ctx| self.draw_radar(ctx))
            .render(area, buf);
            
        // Stats in corners
        let count_txt = format!("TRACKING: {}", self.projects.len());
        buf.set_string(area.x + 2, area.y + area.height - 2, count_txt, Style::default().fg(colors::FG_HINT));

        let zoom_txt = format!("SENSOR RANGE: {}d", self.state.range_days);
        buf.set_string(area.x + area.width - zoom_txt.len() as u16 - 2, area.y + area.height - 2, zoom_txt, Style::default().fg(colors::FG_HINT));
    }
}