use anyhow::Result;

pub fn check_dependencies(require_fzf: bool, require_renderer: bool) -> Result<()> {
    check_dependencies_with(require_fzf, require_renderer, |name| which::which(name).is_ok())
}

fn check_dependencies_with<F>(
    require_fzf: bool,
    require_renderer: bool,
    mut finder: F,
) -> Result<()>
where
    F: FnMut(&str) -> bool,
{
    let deps = ["git", "curl"];
    for dep in deps {
        if !finder(dep) {
            anyhow::bail!("Required dependency '{}' is not installed or not in PATH.", dep);
        }
    }

    if require_fzf && !finder("fzf") {
        anyhow::bail!("Required dependency 'fzf' is not installed or not in PATH.");
    }

    if require_renderer {
        let has_renderer = finder("man-db") || finder("mandoc") || finder("man");
        if !has_renderer {
            anyhow::bail!(
                "A man page renderer is required. Please install 'man-db', 'mandoc', or just regular 'man'."
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_dependencies_with;
    use std::collections::HashSet;

    fn make_finder(present: &[&str]) -> impl FnMut(&str) -> bool {
        let set: HashSet<String> = present.iter().map(|name| name.to_string()).collect();
        move |name| set.contains(name)
    }

    #[test]
    fn deps_missing_required_binary_errors() {
        let finder = make_finder(&["curl", "fzf", "man"]);
        let err = check_dependencies_with(true, true, finder).unwrap_err();
        assert!(err.to_string().contains("git"));
    }

    #[test]
    fn deps_missing_renderer_errors() {
        let finder = make_finder(&["git", "curl", "fzf"]);
        let err = check_dependencies_with(true, true, finder).unwrap_err();
        assert!(err.to_string().contains("man page renderer"));
    }

    #[test]
    fn deps_accepts_man_db() {
        let finder = make_finder(&["git", "curl", "fzf", "man-db"]);
        check_dependencies_with(true, true, finder).unwrap();
    }

    #[test]
    fn deps_accepts_mandoc() {
        let finder = make_finder(&["git", "curl", "fzf", "mandoc"]);
        check_dependencies_with(true, true, finder).unwrap();
    }

    #[test]
    fn deps_accepts_man() {
        let finder = make_finder(&["git", "curl", "fzf", "man"]);
        check_dependencies_with(true, true, finder).unwrap();
    }

    #[test]
    fn deps_skip_fzf_when_not_required() {
        let finder = make_finder(&["git", "curl", "man"]);
        check_dependencies_with(false, true, finder).unwrap();
    }

    #[test]
    fn deps_skip_renderer_when_not_required() {
        let finder = make_finder(&["git", "curl", "fzf"]);
        check_dependencies_with(true, false, finder).unwrap();
    }
}
