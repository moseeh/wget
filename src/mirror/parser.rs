use std::collections::{HashMap, HashSet};
use std::path::Path;
use url::Url;

/// Extracts links from HTML. Per the spec we follow:
///   <a href="...">, <link href="...">, <img src="...">
pub fn extract_links(html: &str, base_url: &Url) -> Vec<String> {
    let mut links = HashSet::new();

    // Match opening tags for the three allowed elements, then pull href/src
    // from the attribute block. Tolerates whitespace and either quote style.
    let tag_re = match regex::Regex::new(
        r#"(?is)<\s*(a|link|img)\b([^>]*)>"#,
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let attr_re = match regex::Regex::new(
        r#"(?is)\b(href|src)\s*=\s*["']([^"']+)["']"#,
    ) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    for tag_cap in tag_re.captures_iter(html) {
        let tag_name = tag_cap.get(1).map(|m| m.as_str().to_ascii_lowercase()).unwrap_or_default();
        let attrs = tag_cap.get(2).map(|m| m.as_str()).unwrap_or("");

        for attr_cap in attr_re.captures_iter(attrs) {
            let attr_name = attr_cap.get(1).map(|m| m.as_str().to_ascii_lowercase()).unwrap_or_default();
            let value = attr_cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Spec: a/href, link/href, img/src
            let want = matches!(
                (tag_name.as_str(), attr_name.as_str()),
                ("a", "href") | ("link", "href") | ("img", "src")
            );
            if !want {
                continue;
            }

            // Skip in-page anchors, mailto:, data:, javascript:, tel:
            let v = value.trim();
            if v.is_empty()
                || v.starts_with('#')
                || v.starts_with("mailto:")
                || v.starts_with("javascript:")
                || v.starts_with("data:")
                || v.starts_with("tel:")
            {
                continue;
            }

            if let Ok(absolute_url) = base_url.join(v) {
                let url_str = absolute_url.to_string();
                if is_valid_mirror_url(&url_str, base_url) {
                    links.insert(url_str);
                }
            }
        }
    }

    links.into_iter().collect()
}

fn is_valid_mirror_url(url: &str, base_url: &Url) -> bool {
    if let Ok(parsed) = Url::parse(url) {
        parsed.domain() == base_url.domain() && parsed.scheme() == base_url.scheme()
    } else {
        false
    }
}

/// Returns true if the URL's path extension matches any of the rejected suffixes.
/// Strips query/fragment first so `/img.jpg?v=2` is correctly classified.
pub fn should_reject_file(url: &str, reject_suffixes: &Option<String>) -> bool {
    let Some(suffixes) = reject_suffixes else { return false };
    let path = match Url::parse(url) {
        Ok(u) => u.path().to_string(),
        Err(_) => url.split('?').next().unwrap_or(url).to_string(),
    };
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    let Some(ext) = ext else { return false };

    suffixes
        .split(',')
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .any(|s| s == ext)
}

pub fn should_exclude_directory(url: &str, exclude_dirs: &Option<String>) -> bool {
    let Some(dirs) = exclude_dirs else { return false };
    let path = match Url::parse(url) {
        Ok(u) => u.path().to_string(),
        Err(_) => url.to_string(),
    };
    for dir in dirs.split(',') {
        let d = dir.trim().trim_start_matches('/').trim_end_matches('/');
        if d.is_empty() {
            continue;
        }
        // Match the directory anywhere in the path as a complete segment
        let needle_mid = format!("/{}/", d);
        let needle_start = format!("/{}", d);
        if path.contains(&needle_mid) || path.starts_with(&needle_start) && (path == needle_start || path[needle_start.len()..].starts_with('/')) {
            return true;
        }
    }
    false
}

/// Rewrite href/src URLs in HTML to point at the locally-saved files.
/// `from_file` is the on-disk path of the document being rewritten — we
/// produce relative paths from its directory.
pub fn rewrite_html_links(
    html: &str,
    base_url: &Url,
    from_file: &Path,
    url_to_path: &HashMap<&str, &Path>,
) -> String {
    // Replace href="..." and src="..." with relative local paths when we have them.
    let re = match regex::Regex::new(r#"(?is)(<\s*(?:a|link|img)\b[^>]*?\b(?:href|src)\s*=\s*)(["'])([^"']+)(["'])"#) {
        Ok(r) => r,
        Err(_) => return html.to_string(),
    };

    re.replace_all(html, |caps: &regex::Captures| {
        let prefix = &caps[1];
        let q1 = &caps[2];
        let value = &caps[3];
        let q2 = &caps[4];

        let v = value.trim();
        if v.is_empty()
            || v.starts_with('#')
            || v.starts_with("mailto:")
            || v.starts_with("javascript:")
            || v.starts_with("data:")
            || v.starts_with("tel:")
        {
            return format!("{}{}{}{}", prefix, q1, value, q2);
        }

        if let Ok(absolute) = base_url.join(v) {
            let abs_str = absolute.to_string();
            if let Some(target) = url_to_path.get(abs_str.as_str()) {
                if let Some(rel) = relative_path(from_file, target) {
                    return format!("{}{}{}{}", prefix, q1, rel, q2);
                }
            }
        }
        format!("{}{}{}{}", prefix, q1, value, q2)
    })
    .into_owned()
}

/// Rewrite url(...) references in CSS to relative local paths.
pub fn rewrite_css_links(
    css: &str,
    base_url: &Url,
    from_file: &Path,
    url_to_path: &HashMap<&str, &Path>,
) -> String {
    let re = match regex::Regex::new(r#"(?is)url\(\s*(["']?)([^)"']+)\1\s*\)"#) {
        Ok(r) => r,
        Err(_) => return css.to_string(),
    };

    re.replace_all(css, |caps: &regex::Captures| {
        let q = &caps[1];
        let value = &caps[2];
        let v = value.trim();
        if v.is_empty() || v.starts_with("data:") {
            return format!("url({}{}{})", q, value, q);
        }
        if let Ok(absolute) = base_url.join(v) {
            let abs_str = absolute.to_string();
            if let Some(target) = url_to_path.get(abs_str.as_str()) {
                if let Some(rel) = relative_path(from_file, target) {
                    return format!("url({}{}{})", q, rel, q);
                }
            }
        }
        format!("url({}{}{})", q, value, q)
    })
    .into_owned()
}

/// Compute a relative path from `from_file`'s directory to `target`.
fn relative_path(from_file: &Path, target: &Path) -> Option<String> {
    let from_dir = from_file.parent()?;
    let from_components: Vec<_> = from_dir.components().collect();
    let target_components: Vec<_> = target.components().collect();

    let common = from_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let mut rel = std::path::PathBuf::new();
    for _ in common..from_components.len() {
        rel.push("..");
    }
    for c in &target_components[common..] {
        rel.push(c.as_os_str());
    }
    Some(rel.to_string_lossy().into_owned())
}
