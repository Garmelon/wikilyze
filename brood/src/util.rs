pub fn normalize_link(link: &str) -> String {
    let link = link.trim().replace(' ', "_");

    // Make only first char lowercase
    link.chars()
        .next()
        .iter()
        .flat_map(|c| c.to_lowercase())
        .chain(link.chars().skip(1))
        .collect::<String>()
}
