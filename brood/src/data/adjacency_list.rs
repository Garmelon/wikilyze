use super::info::{LinkInfo, PageInfo};

pub const SENTINEL_PAGE_MARKER: &str = "Q2AKO3OYzyitmCJURghJ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageIdx(pub u32);

impl PageIdx {
    pub const MAX: PageIdx = PageIdx(u32::MAX);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkIdx(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Page<P> {
    pub start: LinkIdx,
    pub data: P,
}

impl<P> Page<P> {
    pub fn change_data<P2>(self, f: impl Fn(P) -> P2) -> Page<P2> {
        Page {
            start: self.start,
            data: f(self.data),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Link<L> {
    pub to: PageIdx,
    pub data: L,
}

impl<L> Link<L> {
    pub fn change_data<L2>(self, f: impl Fn(L) -> L2) -> Link<L2> {
        Link {
            to: self.to,
            data: f(self.data),
        }
    }
}

pub struct AdjacencyList<P, L> {
    pub pages: Vec<Page<P>>,
    pub links: Vec<Link<L>>,
}

impl<P, L> Default for AdjacencyList<P, L> {
    fn default() -> Self {
        Self {
            pages: Default::default(),
            links: Default::default(),
        }
    }
}

impl<P, L> AdjacencyList<P, L> {
    pub fn page(&self, idx: PageIdx) -> &Page<P> {
        &self.pages[idx.0 as usize]
    }

    pub fn page_mut(&mut self, idx: PageIdx) -> &mut Page<P> {
        &mut self.pages[idx.0 as usize]
    }

    pub fn pages_range(&self) -> impl DoubleEndedIterator<Item = PageIdx> {
        (0..self.pages.len() as u32 - 1).map(PageIdx)
    }

    pub fn link_range(&self, idx: PageIdx) -> impl DoubleEndedIterator<Item = LinkIdx> {
        let start_idx = self.page(idx).start;
        let end_idx = self.page(PageIdx(idx.0 + 1)).start;
        (start_idx.0..end_idx.0).map(LinkIdx)
    }

    pub fn link_redirect(&self, idx: PageIdx) -> Option<LinkIdx> {
        let start_idx = self.page(idx).start;
        let end_idx = self.page(PageIdx(idx.0 + 1)).start;
        if start_idx == end_idx {
            None
        } else {
            Some(start_idx)
        }
    }

    pub fn link(&self, idx: LinkIdx) -> &Link<L> {
        &self.links[idx.0 as usize]
    }

    pub fn link_mut(&mut self, idx: LinkIdx) -> &mut Link<L> {
        &mut self.links[idx.0 as usize]
    }

    pub fn change_page_data<P2>(self, page_f: impl Fn(P) -> P2 + Copy) -> AdjacencyList<P2, L> {
        let pages = self
            .pages
            .into_iter()
            .map(|p| p.change_data(page_f))
            .collect::<Vec<_>>();

        AdjacencyList {
            pages,
            links: self.links,
        }
    }

    pub fn change_link_data<L2>(self, link_f: impl Fn(L) -> L2 + Copy) -> AdjacencyList<P, L2> {
        let links = self
            .links
            .into_iter()
            .map(|l| l.change_data(link_f))
            .collect::<Vec<_>>();

        AdjacencyList {
            pages: self.pages,
            links,
        }
    }
}

impl AdjacencyList<PageInfo, LinkInfo> {
    pub fn check_consistency(&self) {
        // Check that we have a sentinel page
        let sentinel = self.pages.last().expect("no sentinel page");
        assert!(sentinel.data.id == u32::MAX, "unmarked sentinel page");
        assert!(
            sentinel.data.title.contains(SENTINEL_PAGE_MARKER),
            "unmarked sentinel page"
        );

        // Check that all types are large enough
        assert!(self.pages.len() < u32::MAX as usize, "too many pages");
        assert!(self.links.len() < u32::MAX as usize, "too many links");
        for page in &self.pages {
            assert!(
                page.data.title.len() <= u8::MAX as usize,
                "page title too long"
            );
        }

        // Check that all links contain valid indices. Links must not link to
        // the sentinel page.
        let range = 0..self.pages.len() as u32 - 1;
        for link in &self.links {
            assert!(range.contains(&link.to.0), "invalid link");
        }

        // Check that all redirect pages have at most one link
        for page_idx in (0..self.pages.len() as u32 - 1).map(PageIdx) {
            let page = self.page(page_idx);
            if page.data.redirect {
                let mut range = self.link_range(page_idx);
                range.next(); // 0 or 1 links allowed
                assert!(range.next().is_none(), "too many redirect links");
            }
        }
    }
}
