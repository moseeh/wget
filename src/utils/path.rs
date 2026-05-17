use std::path::Path;

/// Format a path the way wget displays "saving file to:".
/// Bare filenames get a "./" prefix; absolute paths and paths with directory
/// components are shown as-is.
pub fn display_save_path(p: &Path) -> String {
    if p.is_absolute() {
        return p.display().to_string();
    }
    // If the path has more than one component (i.e. it has a parent
    // directory other than the implicit cwd), show it verbatim.
    if p.parent().map(|par| !par.as_os_str().is_empty()).unwrap_or(false) {
        return p.display().to_string();
    }
    format!("./{}", p.display())
}
