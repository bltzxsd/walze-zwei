pub mod macros;

// Helper functions

/// removes all asterisks and backticks from the string
pub fn normalize_dice_expr(s: &str) -> String {
    s.replace(['*', '`'], "")
}

/// splits dice string with commas: ,
pub fn split_dice(expr: &str) -> Vec<&str> {
    expr.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}
