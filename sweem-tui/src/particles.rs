//! Particle system for background animations.
//!
//! This module implements a lightweight particle system that creates
//! ambient effects: floating ash, dust particles, or subtle embers.
//! Designed to match the Kanagawa Dragon aesthetic.

use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::theme::colors;

/// Types of background animations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParticleMode {
    /// Floating ash particles (default for Kanagawa Dragon theme)
    #[default]
    FloatingAsh,
    /// Subtle dust particles
    Dust,
    /// Warm ember effect
    Embers,
    /// No particles (static background)
    None,
}

impl ParticleMode {
    /// Cycle to the next mode
    pub fn next(&self) -> Self {
        match self {
            ParticleMode::FloatingAsh => ParticleMode::Dust,
            ParticleMode::Dust => ParticleMode::Embers,
            ParticleMode::Embers => ParticleMode::None,
            ParticleMode::None => ParticleMode::FloatingAsh,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ParticleMode::FloatingAsh => "Floating Ash",
            ParticleMode::Dust => "Dust",
            ParticleMode::Embers => "Embers",
            ParticleMode::None => "None",
        }
    }
}

/// A single particle in the system
#[derive(Debug, Clone)]
pub struct Particle {
    /// X position (column)
    pub x: f32,
    /// Y position (row)
    pub y: f32,
    /// Velocity in Y direction
    pub vy: f32,
    /// Velocity in X direction
    pub vx: f32,
    /// Character to display
    pub char: char,
    /// Brightness (0.0 - 1.0)
    pub brightness: f32,
    /// Fade rate
    pub fade_rate: f32,
    /// Particle age (for animation)
    pub age: u32,
    /// Sway phase (for gentle horizontal movement)
    pub sway_phase: f32,
}

impl Particle {
    /// Create a new floating ash particle
    pub fn new_ash(width: u16, height: u16) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0.0..width as f32),
            y: rng.gen_range(0.0..height as f32),
            vy: rng.gen_range(-0.1..-0.02), // Slowly rise like ash
            vx: 0.0,
            char: Self::random_ash_char(),
            brightness: rng.gen_range(0.2..0.6),
            fade_rate: rng.gen_range(0.002..0.008),
            age: 0,
            sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }

    /// Create a new dust particle
    pub fn new_dust(width: u16, height: u16) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0.0..width as f32),
            y: rng.gen_range(0.0..height as f32),
            vy: rng.gen_range(-0.05..0.05), // Gentle drift
            vx: rng.gen_range(-0.02..0.02),
            char: Self::random_dust_char(),
            brightness: rng.gen_range(0.1..0.4),
            fade_rate: rng.gen_range(0.001..0.005),
            age: 0,
            sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }

    /// Create a new ember particle
    pub fn new_ember(width: u16, _height: u16) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(0.0..width as f32),
            y: rng.gen_range(0.7.._height as f32), // Start from bottom
            vy: rng.gen_range(-0.2..-0.05), // Rise like embers
            vx: rng.gen_range(-0.03..0.03),
            char: Self::random_ember_char(),
            brightness: rng.gen_range(0.4..0.8),
            fade_rate: rng.gen_range(0.005..0.015),
            age: 0,
            sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }

    /// Get a random character for ash
    fn random_ash_char() -> char {
        let mut rng = rand::thread_rng();
        let chars = ['·', '∙', '˙', '°', '·', '∙'];
        chars[rng.gen_range(0..chars.len())]
    }

    /// Get a random character for dust
    fn random_dust_char() -> char {
        let mut rng = rand::thread_rng();
        let chars = ['·', '∙', '⁺', '˚', '∘'];
        chars[rng.gen_range(0..chars.len())]
    }

    /// Get a random character for embers
    fn random_ember_char() -> char {
        let mut rng = rand::thread_rng();
        let chars = ['◦', '°', '∘', '•', '·'];
        chars[rng.gen_range(0..chars.len())]
    }

    /// Update particle position and state
    pub fn update(&mut self, mode: ParticleMode) {
        self.age = self.age.wrapping_add(1);

        // Apply sway for natural movement
        let sway = (self.sway_phase + self.age as f32 * 0.05).sin() * 0.02;

        match mode {
            ParticleMode::FloatingAsh => {
                self.y += self.vy;
                self.x += sway;
            }
            ParticleMode::Dust => {
                self.y += self.vy;
                self.x += self.vx + sway * 0.5;
            }
            ParticleMode::Embers => {
                self.y += self.vy;
                self.x += self.vx + sway;
                // Embers flicker
                if rand::thread_rng().gen_ratio(1, 10) {
                    self.brightness = (self.brightness + rand::thread_rng().gen_range(-0.1..0.1))
                        .clamp(0.2, 0.9);
                }
            }
            ParticleMode::None => {}
        }

        self.brightness -= self.fade_rate;
    }

    /// Check if particle is still visible
    pub fn is_alive(&self, max_y: u16, max_x: u16) -> bool {
        self.brightness > 0.05
            && self.y >= 0.0
            && self.y < max_y as f32
            && self.x >= 0.0
            && self.x < max_x as f32
    }

    /// Get the color based on brightness and mode
    pub fn get_color(&self, mode: ParticleMode) -> Color {
        match mode {
            ParticleMode::FloatingAsh => {
                // Warm gray ash color
                let base = colors::PARTICLE_ASH;
                if let Color::Rgb(r, g, b) = base {
                    let factor = self.brightness;
                    Color::Rgb(
                        (r as f32 * factor) as u8,
                        (g as f32 * factor) as u8,
                        (b as f32 * factor) as u8,
                    )
                } else {
                    base
                }
            }
            ParticleMode::Dust => {
                // Slightly cooler dust
                let base = colors::PARTICLE_DUST;
                if let Color::Rgb(r, g, b) = base {
                    let factor = self.brightness;
                    Color::Rgb(
                        (r as f32 * factor) as u8,
                        (g as f32 * factor) as u8,
                        (b as f32 * factor) as u8,
                    )
                } else {
                    base
                }
            }
            ParticleMode::Embers => {
                // Warm ember glow - more orange/red
                let factor = self.brightness;
                Color::Rgb(
                    (180.0 * factor) as u8,
                    (80.0 * factor) as u8,
                    (40.0 * factor) as u8,
                )
            }
            ParticleMode::None => Color::Reset,
        }
    }
}

/// The particle system managing all particles
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    /// All active particles
    particles: Vec<Particle>,
    /// Current animation mode
    mode: ParticleMode,
    /// Maximum number of particles
    max_particles: usize,
    /// Frame counter for spawn timing
    frame_count: u64,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(ParticleMode::FloatingAsh, 60)
    }
}

impl ParticleSystem {
    /// Create a new particle system
    pub fn new(mode: ParticleMode, max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            mode,
            max_particles,
            frame_count: 0,
        }
    }

    /// Set the animation mode
    pub fn set_mode(&mut self, mode: ParticleMode) {
        if self.mode != mode {
            self.mode = mode;
            self.particles.clear();
        }
    }

    /// Get current mode
    pub fn mode(&self) -> ParticleMode {
        self.mode
    }

    /// Toggle to the next animation mode
    pub fn toggle_mode(&mut self) {
        self.set_mode(self.mode.next());
    }

    /// Update all particles and spawn new ones
    pub fn update(&mut self, width: u16, height: u16) {
        self.frame_count = self.frame_count.wrapping_add(1);

        if self.mode == ParticleMode::None {
            return;
        }

        // Update existing particles
        for particle in &mut self.particles {
            particle.update(self.mode);
        }

        // Remove dead particles
        self.particles
            .retain(|p| p.is_alive(height, width));

        // Spawn new particles
        self.spawn_particles(width, height);
    }

    /// Spawn new particles based on mode
    fn spawn_particles(&mut self, width: u16, height: u16) {
        let mut rng = rand::thread_rng();

        match self.mode {
            ParticleMode::FloatingAsh => {
                // Spawn ash particles slowly
                if self.frame_count % 8 == 0 && self.particles.len() < self.max_particles {
                    let num_new = rng.gen_range(1..=2).min(self.max_particles - self.particles.len());
                    for _ in 0..num_new {
                        self.particles.push(Particle::new_ash(width, height));
                    }
                }
            }
            ParticleMode::Dust => {
                // Maintain a steady number of dust particles
                if self.frame_count % 10 == 0 && self.particles.len() < self.max_particles / 2 {
                    self.particles.push(Particle::new_dust(width, height));
                }
            }
            ParticleMode::Embers => {
                // Spawn embers from bottom occasionally
                if self.frame_count % 15 == 0 && self.particles.len() < self.max_particles / 3 {
                    self.particles.push(Particle::new_ember(width, height));
                }
            }
            ParticleMode::None => {}
        }
    }

    /// Render the particle system
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if self.mode == ParticleMode::None {
            return;
        }

        for particle in &self.particles {
            let x = particle.x as u16;
            let y = particle.y as u16;

            if x < area.width && y < area.height {
                let pos = (area.x + x, area.y + y);
                let color = particle.get_color(self.mode);
                buf[pos].set_char(particle.char);
                buf[pos].set_style(Style::default().fg(color));
            }
        }
    }
}

/// Widget wrapper for the particle system
pub struct ParticleWidget<'a> {
    system: &'a ParticleSystem,
}

impl<'a> ParticleWidget<'a> {
    pub fn new(system: &'a ParticleSystem) -> Self {
        Self { system }
    }
}

impl Widget for ParticleWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.system.render(area, buf);
    }
}
