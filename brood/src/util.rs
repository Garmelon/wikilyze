pub fn normalize_link(link: &str) -> String {
    link.trim().to_lowercase().replace(' ', "_")
}
