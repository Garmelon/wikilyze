use std::ops::Range;

use super::info::{LinkInfo, PageInfo};

#[derive(Debug, Clone, Copy)]
pub struct Page<P> {
    /// Index of the first link belonging to this page.
    pub start: u32,
    pub data: P,
}

impl<P> Page<P> {
    pub fn change_data<P2>(self, f: &impl Fn(P) -> P2) -> Page<P2> {
        Page {
            start: self.start,
            data: f(self.data),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Link<L> {
    /// Index of the page this link points to.
    pub to: u32,
    pub data: L,
}

impl<L> Link<L> {
    pub fn change_data<L2>(self, f: &impl Fn(L) -> L2) -> Link<L2> {
        Link {
            to: self.to,
            data: f(self.data),
        }
    }

    pub fn change_data_with_page<P, L2>(self, page: &P, f: &impl Fn(&P, L) -> L2) -> Link<L2> {
        Link {
            to: self.to,
            data: f(page, self.data),
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
            pages: vec![],
            links: vec![],
        }
    }
}

impl<P, L> AdjacencyList<P, L> {
    pub fn push_page(&mut self, data: P) {
        self.pages.push(Page {
            start: self.links.len() as u32,
            data,
        });
    }

    pub fn push_link(&mut self, to: u32, data: L) {
        self.links.push(Link { to, data })
    }

    pub fn page(&self, page_idx: u32) -> &Page<P> {
        &self.pages[page_idx as usize]
    }

    pub fn page_mut(&mut self, page_idx: u32) -> &mut Page<P> {
        &mut self.pages[page_idx as usize]
    }

    pub fn pages(&self) -> impl Iterator<Item = (u32, &Page<P>)> {
        self.pages.iter().enumerate().map(|(i, p)| (i as u32, p))
    }

    pub fn link(&self, link_idx: u32) -> &Link<L> {
        &self.links[link_idx as usize]
    }

    pub fn link_mut(&mut self, link_idx: u32) -> &mut Link<L> {
        &mut self.links[link_idx as usize]
    }

    pub fn link_range(&self, page_idx: u32) -> Range<u32> {
        let start_idx = self.pages[page_idx as usize].start;
        let end_idx = match self.pages.get(page_idx as usize + 1) {
            Some(page) => page.start,
            None => self.links.len() as u32,
        };
        start_idx..end_idx
    }

    pub fn link_redirect(&self, page_idx: u32) -> Option<u32> {
        let range = self.link_range(page_idx);
        if range.is_empty() {
            None
        } else {
            Some(range.start)
        }
    }

    pub fn links(&self, page_idx: u32) -> impl Iterator<Item = (u32, &Link<L>)> {
        self.link_range(page_idx).map(|i| (i, self.link(i)))
    }

    pub fn change_page_data<P2>(self, page_f: impl Fn(P) -> P2) -> AdjacencyList<P2, L> {
        let pages = self
            .pages
            .into_iter()
            .map(|p| p.change_data(&page_f))
            .collect::<Vec<_>>();

        AdjacencyList {
            pages,
            links: self.links,
        }
    }

    pub fn change_link_data<L2>(self, link_f: impl Fn(L) -> L2) -> AdjacencyList<P, L2> {
        let links = self
            .links
            .into_iter()
            .map(|l| l.change_data(&link_f))
            .collect::<Vec<_>>();

        AdjacencyList {
            pages: self.pages,
            links,
        }
    }

    pub fn change_link_data_with_page<L2>(
        self,
        link_f: impl Fn(&P, L) -> L2,
    ) -> AdjacencyList<P, L2> {
        let mut pages = self.pages.iter().peekable();
        let Some(mut cur_page) = pages.next() else {
            // The list is empty, nothing to do
            return AdjacencyList::default();
        };

        let mut links = vec![];

        for (i, link) in self.links.into_iter().enumerate() {
            if let Some(page) = pages.peek() {
                if i >= page.start as usize {
                    cur_page = page;
                    pages.next();
                }
            }

            links.push(link.change_data_with_page(&cur_page.data, &link_f));
        }

        AdjacencyList {
            pages: self.pages,
            links,
        }
    }
}

impl AdjacencyList<PageInfo, LinkInfo> {
    pub fn check_consistency(&self) {
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
        let range = 0..self.pages.len() as u32;
        for link in &self.links {
            assert!(range.contains(&link.to), "invalid link");
        }

        // Check that all redirect pages have at most one link
        for (page_idx, page) in self.pages.iter().enumerate() {
            if page.data.redirect {
                let range = self.link_range(page_idx as u32);
                let amount = range.end - range.start;
                assert!(amount <= 1, "too many redirect links");
            }
        }
    }
}
