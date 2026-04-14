//! Shared helpers for paper artifact generation.

/// Stylized backend names for publication.
pub fn backend_display_name(name: &str) -> &'static str {
    match name {
        "native" => "Base",
        "yolo-no-perm" => "\\fs",
        "yolo-realistic" => "\\fs",
        "yolo" => "\\fs",
        "overlayfs" => "OverlayFS",
        "branchfs" => "BranchFS",
        _ => "Unknown",
    }
}

/// Escape special LaTeX characters.
pub fn latex_escape(s: &str) -> String {
    // Preserve known LaTeX commands.
    if s.starts_with('\\') && !s.contains(' ') {
        return s.to_string();
    }
    s.replace('\\', "\\textbackslash{}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('~', "\\textasciitilde{}")
        .replace('^', "\\textasciicircum{}")
}
