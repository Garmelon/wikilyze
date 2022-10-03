use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub link_idx: u32,
    pub id: u32,
    pub title: String,
    pub redirect: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
