use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub link_idx: u32,
    pub ns: u16,
    pub id: u32,
    pub title: String,
    pub redirect: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub to: u32,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjacencyList {
    pub pages: Vec<Page>,
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimAdjacencyList {
    pages: Vec<(u32, u32, u16, String, bool)>,
    links: Vec<(u32, u32, u32)>,
}

impl SlimAdjacencyList {
    pub fn from_alist(alist: AdjacencyList) -> Self {
        let pages = alist
            .pages
            .into_iter()
            .map(|p| (p.link_idx, p.id, p.ns, p.title, p.redirect))
            .collect();

        let links = alist
            .links
            .into_iter()
            .map(|l| (l.to, l.start, l.end))
            .collect();

        Self { pages, links }
    }

    pub fn to_alist(self) -> AdjacencyList {
        let pages = self
            .pages
            .into_iter()
            .map(|(link_idx, id, ns, title, redirect)| Page {
                link_idx,
                ns,
                id,
                title,
                redirect,
            })
            .collect();

        let links = self
            .links
            .into_iter()
            .map(|(to, start, end)| Link { to, start, end })
            .collect();

        AdjacencyList { pages, links }
    }
}
