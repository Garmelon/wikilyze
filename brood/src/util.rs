use crate::data::{AdjacencyList, LinkInfo, Page, PageInfo};

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

pub fn find_index_of_title(pages: &[Page<PageInfo>], title: &str) -> u32 {
    let title = normalize_link(title);
    pages
        .iter()
        .enumerate()
        .find(|(_, p)| normalize_link(&p.data.title) == title)
        .map(|(i, _)| i)
        .expect("invalid title") as u32
}

pub fn resolve_redirects(data: &AdjacencyList<PageInfo, LinkInfo>, mut page_idx: u32) -> u32 {
    loop {
        if data.page(page_idx).data.redirect {
            if let Some(link_idx) = data.link_redirect(page_idx) {
                page_idx = data.link(link_idx).to;
                continue;
            }
        }

        return page_idx;
    }
}
