//! Kanagawa Dragon theme module.
//!
//! This module implements the "Kanagawa Dragon" / "Ef-Autumn" color palette.
//! A low-contrast, warm, dark theme inspired by traditional Japanese ink wash painting.

#![allow(dead_code)]

use ratatui::style::Color;

/// Kanagawa Dragon color palette
/// Low-contrast, warm, dark theme inspired by traditional Japanese ink wash painting
pub mod colors {
    use super::Color;

    // === Background Colors ===
    /// Dragon Black - Primary background
    pub const BG_DARK: Color = Color::Rgb(0x18, 0x16, 0x16);
    /// Slightly lighter background for medium contrast areas
    pub const BG_MEDIUM: Color = Color::Rgb(0x1D, 0x1C, 0x19);
    /// Background for highlighted/selected areas
    pub const BG_HIGHLIGHT: Color = Color::Rgb(0x28, 0x27, 0x27);
    /// Background for dimmed/overlay areas
    pub const BG_DIM: Color = Color::Rgb(0x12, 0x12, 0x12);

    // === Foreground Colors ===
    /// Old White - Primary text color
    pub const FG_PRIMARY: Color = Color::Rgb(0xC5, 0xC9, 0xC5);
    /// Dimmed text for secondary information
    pub const FG_DIM: Color = Color::Rgb(0x72, 0x71, 0x69);
    /// Very dim text for hints and placeholders
    pub const FG_HINT: Color = Color::Rgb(0x54, 0x54, 0x54);

    // === Accent Colors ===
    /// Dragon Red - For errors, delete actions, and warnings
    pub const RED: Color = Color::Rgb(0xC4, 0x74, 0x6E);
    /// Light Red - For hover/lighter red accents
    pub const RED_LIGHT: Color = Color::Rgb(0xE4, 0x6B, 0x6B);

    /// Dragon Green - For success, completed items
    pub const GREEN: Color = Color::Rgb(0x8A, 0x9A, 0x7B);
    /// Light Green - For hover/lighter green accents
    pub const GREEN_LIGHT: Color = Color::Rgb(0x87, 0xA9, 0x87);

    /// Carp Yellow - For warnings, in-progress items
    pub const YELLOW: Color = Color::Rgb(0xC4, 0xB2, 0x8A);
    /// Orange - For attention-grabbing elements
    pub const ORANGE: Color = Color::Rgb(0xB6, 0x92, 0x7B);

    /// Dragon Blue - For info, selected items
    pub const BLUE: Color = Color::Rgb(0x8B, 0xA4, 0xB0);
    /// Light Blue - For hover/lighter blue accents
    pub const BLUE_LIGHT: Color = Color::Rgb(0x7F, 0xB4, 0xCA);

    /// Purple - For special accents
    pub const PURPLE: Color = Color::Rgb(0x95, 0x7F, 0xB8);
    /// Magenta - For highlights
    pub const MAGENTA: Color = Color::Rgb(0xD2, 0x7E, 0x99);

    // === UI Element Colors ===
    /// Wall Gray - For borders and separators
    pub const BORDER: Color = Color::Rgb(0x72, 0x71, 0x69);
    /// Dim border for less important separators
    pub const BORDER_DIM: Color = Color::Rgb(0x3A, 0x3A, 0x3A);
    /// Accent border for focused elements
    pub const BORDER_ACCENT: Color = Color::Rgb(0x8B, 0xA4, 0xB0);

    // === Status Colors ===
    /// Connected status
    pub const STATUS_CONNECTED: Color = GREEN;
    /// Disconnected/Error status
    pub const STATUS_DISCONNECTED: Color = RED;
    /// Loading/Pending status
    pub const STATUS_PENDING: Color = YELLOW;

    // === Timeline Colors ===
    /// Today marker line
    pub const TODAY_MARKER: Color = YELLOW;
    /// Overdue project bar
    pub const PROJECT_OVERDUE: Color = RED;
    /// Completed project bar
    pub const PROJECT_COMPLETED: Color = GREEN;
    /// In-progress project bar
    pub const PROJECT_ACTIVE: Color = BLUE;

}

/// Color palette for project bars in the timeline
/// Vibrant, distinct colors for easy project differentiation
/// Uses a rainbow-like progression for maximum visual clarity
pub const PROJECT_COLORS: &[Color] = &[
    Color::Rgb(0x7A, 0xA2, 0xF7), // Bright blue - Project 1
    Color::Rgb(0x9E, 0xCE, 0x6A), // Bright green - Project 2
    Color::Rgb(0xE0, 0xAF, 0x68), // Golden yellow - Project 3
    Color::Rgb(0xBB, 0x9A, 0xF7), // Bright purple - Project 4
    Color::Rgb(0xFF, 0x9E, 0x64), // Bright orange - Project 5
    Color::Rgb(0xF7, 0x76, 0x8E), // Pink/magenta - Project 6
    Color::Rgb(0x73, 0xDA, 0xCA), // Cyan/teal - Project 7
    Color::Rgb(0xFF, 0x75, 0x7F), // Coral red - Project 8
    Color::Rgb(0xC0, 0xCA, 0xF5), // Lavender - Project 9
    Color::Rgb(0xA9, 0xDC, 0x76), // Lime green - Project 10
    Color::Rgb(0xF2, 0xCD, 0xCD), // Light pink - Project 11
    Color::Rgb(0x89, 0xDD, 0xFF), // Sky blue - Project 12
];

/// Get a dimmed version of a project color (for secondary elements)
pub fn get_project_color_dim(index: usize) -> Color {
    let base = PROJECT_COLORS[index % PROJECT_COLORS.len()];
    if let Color::Rgb(r, g, b) = base {
        Color::Rgb(r / 2, g / 2, b / 2)
    } else {
        base
    }
}

/// Semantic styling helpers
pub mod styles {
    use ratatui::style::{Modifier, Style};
    use super::colors;

    /// Style for primary text
    pub fn text() -> Style {
        Style::default().fg(colors::FG_PRIMARY)
    }

    /// Style for dimmed/secondary text
    pub fn text_dim() -> Style {
        Style::default().fg(colors::FG_DIM)
    }

    /// Style for hint text
    pub fn text_hint() -> Style {
        Style::default().fg(colors::FG_HINT)
    }

    /// Style for success messages
    pub fn success() -> Style {
        Style::default().fg(colors::GREEN)
    }

    /// Style for error messages
    pub fn error() -> Style {
        Style::default().fg(colors::RED)
    }

    /// Style for warning messages
    pub fn warning() -> Style {
        Style::default().fg(colors::YELLOW)
    }

    /// Style for info messages
    pub fn info() -> Style {
        Style::default().fg(colors::BLUE)
    }

    /// Style for selected/highlighted items
    pub fn selected() -> Style {
        Style::default()
            .fg(colors::BG_DARK)
            .bg(colors::BLUE)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for focused borders
    pub fn border_focused() -> Style {
        Style::default().fg(colors::BORDER_ACCENT)
    }

    /// Style for unfocused borders
    pub fn border() -> Style {
        Style::default().fg(colors::BORDER)
    }

    /// Style for dim borders
    pub fn border_dim() -> Style {
        Style::default().fg(colors::BORDER_DIM)
    }

    /// Style for block titles
    pub fn title() -> Style {
        Style::default()
            .fg(colors::FG_PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for accent titles (tabs, headers)
    pub fn title_accent() -> Style {
        Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for tab titles (active)
    pub fn tab_active() -> Style {
        Style::default()
            .fg(colors::BLUE)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for tab titles (inactive)
    pub fn tab_inactive() -> Style {
        Style::default().fg(colors::FG_DIM)
    }

    /// Style for form labels
    pub fn form_label() -> Style {
        Style::default().fg(colors::FG_DIM)
    }

    /// Style for form input (focused)
    pub fn form_input_focused() -> Style {
        Style::default()
            .fg(colors::FG_PRIMARY)
            .bg(colors::BG_HIGHLIGHT)
    }

    /// Style for form input (unfocused)
    pub fn form_input() -> Style {
        Style::default()
            .fg(colors::FG_PRIMARY)
            .bg(colors::BG_MEDIUM)
    }

    /// Style for buttons
    pub fn button() -> Style {
        Style::default()
            .fg(colors::FG_PRIMARY)
            .bg(colors::BG_MEDIUM)
    }

    /// Style for focused buttons
    pub fn button_focused() -> Style {
        Style::default()
            .fg(colors::BG_DARK)
            .bg(colors::BLUE)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for danger buttons (delete, cancel)
    pub fn button_danger() -> Style {
        Style::default()
            .fg(colors::BG_DARK)
            .bg(colors::RED)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for modal overlay background
    pub fn modal_bg() -> Style {
        Style::default().bg(colors::BG_DIM)
    }

    /// Style for modal content background
    pub fn modal_content_bg() -> Style {
        Style::default().bg(colors::BG_MEDIUM)
    }
}

/// Get a project color by index (cycles through available colors)
pub fn get_project_color(index: usize) -> Color {
    PROJECT_COLORS[index % PROJECT_COLORS.len()]
}
