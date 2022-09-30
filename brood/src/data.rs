pub struct Page {
    pub link_idx: u32,
    pub ns: u16,
    pub id: u32,
    pub title: String,
    pub redirect: bool,
}

pub struct Link {
    pub to: u32,
    pub start: u32,
    pub end: u32,
}

pub struct AdjacencyList {
    pub pages: Vec<Page>,
    pub links: Vec<Link>,
}
