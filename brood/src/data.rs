use std::io::{self, Read, Write};
use std::ops::Range;

mod ioutil {
    use std::io::{self, Read, Write};

    pub fn write_u8<W: Write>(n: u8, to: &mut W) -> io::Result<()> {
        to.write_all(&n.to_le_bytes())
    }

    pub fn read_u8<R: Read>(from: &mut R) -> io::Result<u8> {
        let mut buf = [0_u8; 1];
        from.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf))
    }

    pub fn write_u16<W: Write>(n: u16, to: &mut W) -> io::Result<()> {
        to.write_all(&n.to_le_bytes())
    }

    pub fn read_u16<R: Read>(from: &mut R) -> io::Result<u16> {
        let mut buf = [0_u8; 2];
        from.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn write_u32<W: Write>(n: u32, to: &mut W) -> io::Result<()> {
        to.write_all(&n.to_le_bytes())
    }

    pub fn read_u32<R: Read>(from: &mut R) -> io::Result<u32> {
        let mut buf = [0_u8; 4];
        from.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn write_str<W: Write>(s: &str, to: &mut W) -> io::Result<()> {
        assert!(s.len() <= u16::MAX as usize);
        write_u16(s.len() as u16, to)?;
        to.write_all(s.as_bytes())?;
        Ok(())
    }

    pub fn read_str<R: Read>(from: &mut R) -> io::Result<String> {
        let len = read_u16(from)? as usize;
        let mut buf = vec![0_u8; len];
        from.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub id: u32,
    pub length: u32,
    pub redirect: bool,
    pub title: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Page<P> {
    pub link_idx: u32,
    pub data: P,
}

impl Page<PageInfo> {
    pub fn write<W: Write>(&self, to: &mut W) -> io::Result<()> {
        ioutil::write_u32(self.link_idx, to)?;
        ioutil::write_u32(self.data.id, to)?;
        ioutil::write_u32(self.data.length, to)?;
        ioutil::write_u8(if self.data.redirect { 1 } else { 0 }, to)?;
        ioutil::write_str(&self.data.title, to)?;

        Ok(())
    }

    pub fn read<R: Read>(from: &mut R) -> io::Result<Self> {
        let link_idx = ioutil::read_u32(from)?;
        let id = ioutil::read_u32(from)?;
        let length = ioutil::read_u32(from)?;
        let redirect = ioutil::read_u8(from)? != 0;
        let title = ioutil::read_str(from)?;

        Ok(Self {
            link_idx,
            data: PageInfo {
                id,
                length,
                redirect,
                title,
            },
        })
    }
}

impl<P> Page<P> {
    pub fn change_data<P2>(self, f: impl Fn(P) -> P2) -> Page<P2> {
        Page {
            link_idx: self.link_idx,
            data: f(self.data),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LinkInfo {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Link<L> {
    pub to: u32,
    pub data: L,
}

impl Link<LinkInfo> {
    pub fn write<W: Write>(&self, to: &mut W) -> io::Result<()> {
        ioutil::write_u32(self.to, to)?;
        ioutil::write_u32(self.data.start, to)?;
        ioutil::write_u32(self.data.end, to)?;

        Ok(())
    }

    pub fn read<R: Read>(from: &mut R) -> io::Result<Self> {
        let to = ioutil::read_u32(from)?;
        let start = ioutil::read_u32(from)?;
        let end = ioutil::read_u32(from)?;

        Ok(Self {
            to,
            data: LinkInfo { start, end },
        })
    }
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

impl AdjacencyList<PageInfo, LinkInfo> {
    pub fn write<W: Write>(&self, to: &mut W) -> io::Result<()> {
        ioutil::write_u32(self.pages.len() as u32, to)?;
        ioutil::write_u32(self.links.len() as u32, to)?;

        for page in &self.pages {
            page.write(to)?;
        }

        for link in &self.links {
            link.write(to)?;
        }

        Ok(())
    }

    pub fn read<R: Read>(from: &mut R) -> io::Result<Self> {
        let n_pages = ioutil::read_u32(from)?;
        let n_links = ioutil::read_u32(from)?;

        let mut pages = vec![];
        for _ in 0..n_pages {
            pages.push(Page::read(from)?);
        }

        let mut links = vec![];
        for _ in 0..n_links {
            links.push(Link::read(from)?);
        }

        Ok(Self { pages, links })
    }

    pub fn check_consistency(&self) {
        // Check that all types are large enough
        assert!(self.pages.len() <= u32::MAX as usize, "pages len");
        assert!(self.links.len() <= u32::MAX as usize, "links len");
        for page in &self.pages {
            assert!(page.link_idx <= u32::MAX as u32, "page link_idx");
            assert!(page.data.id <= u32::MAX as u32, "page id");
            assert!(page.data.length <= u32::MAX as u32, "page length");
            assert!(page.data.title.len() <= u8::MAX as usize, "page title len");
        }
        for link in &self.links {
            assert!(link.to <= u32::MAX as u32, "link to");
            assert!(link.data.start <= u32::MAX as u32, "link start");
            assert!(link.data.end <= u32::MAX as u32, "link end");
        }

        // Check that all links contain valid indices
        let range = 0..self.pages.len() as u32;
        for link in &self.links {
            if !range.contains(&link.to) {
                panic!("Invalid link detected!");
            }
        }

        // Check that all redirect pages have at most one link
        for page_idx in 0..self.pages.len() as u32 - 1 {
            let page = self.page(page_idx);
            if page.data.redirect {
                let start_idx = page.link_idx;
                let end_idx = self.page(page_idx + 1).link_idx;
                let n_links = end_idx - start_idx;
                if n_links > 1 {
                    panic!(
                        "Redirect {:?} has too many ({n_links}) links",
                        page.data.title
                    );
                }
            }
        }
    }
}

impl<P, L> AdjacencyList<P, L> {
    pub fn page(&self, idx: u32) -> &Page<P> {
        &self.pages[idx as usize]
    }

    pub fn page_mut(&mut self, idx: u32) -> &mut Page<P> {
        &mut self.pages[idx as usize]
    }

    pub fn link_range(&self, page_idx: u32) -> Range<u32> {
        let start_idx = self.page(page_idx).link_idx;
        let end_idx = self.page(page_idx + 1).link_idx;
        start_idx..end_idx
    }

    pub fn link_redirect(&self, page_idx: u32) -> Option<u32> {
        let start_idx = self.page(page_idx).link_idx;
        let end_idx = self.page(page_idx + 1).link_idx;
        if start_idx == end_idx {
            None
        } else {
            Some(start_idx)
        }
    }

    pub fn link(&self, idx: u32) -> &Link<L> {
        &self.links[idx as usize]
    }

    pub fn link_mut(&mut self, idx: u32) -> &mut Link<L> {
        &mut self.links[idx as usize]
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
