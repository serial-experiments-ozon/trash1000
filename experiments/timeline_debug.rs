/// Debug script to demonstrate the timeline unit mismatch bug fix
///
/// ## The Bug (Fixed)
///
/// In the original `jump_to_project` function, there was a unit mismatch:
///
/// ```rust
/// // BUGGY CODE:
/// let target_scroll = (project_start_days as f64 / self.days_per_column) as i64 - offset_from_left;
/// self.scroll_offset = target_scroll.max(0);
/// ```
///
/// The problem:
/// - `project_start_days` is in DAYS
/// - `/ self.days_per_column` converts to COLUMNS
/// - `offset_from_left` is in COLUMNS
/// - So `target_scroll` is in COLUMNS
///
/// But `scroll_offset` is used in `date_to_column_raw` as DAYS:
/// ```rust
/// let days_with_offset = days_from_start - self.state.scroll_offset;  // scroll_offset should be DAYS
/// (days_with_offset as f64 / self.state.days_per_column) as i64
/// ```
///
/// ## The Fix
///
/// Convert the column-based offset back to days:
/// ```rust
/// // FIXED CODE:
/// let offset_from_left_days = (effective_width / 4) as f64 * self.days_per_column;
/// let target_scroll = project_start_days - offset_from_left_days as i64;
/// self.scroll_offset = target_scroll.max(0);
/// ```
///
/// This ensures scroll_offset is always in DAYS, consistent with how it's used
/// in scrolling (scroll_left/scroll_right) and in date_to_column_raw.
///
/// ## Example calculation
///
/// - days_per_column = 1.0
/// - viewport_width = 100
/// - effective_width = 100 - 26 = 74 columns
/// - offset_from_left = 74 / 4 = 18 columns
/// - offset_from_left_days = 18 * 1.0 = 18 days
/// - If project starts on timeline start (day 0), scroll = 0 - 18 = 0 (clamped from -18)
/// - This puts the project start at column 0, making it visible!
fn main() {
    println!("This file documents the timeline unit mismatch bug fix.");
    println!("See the file comments for details.");
}
