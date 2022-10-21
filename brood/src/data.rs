use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub link_idx: u32,
    pub id: u32,
    pub length: u32,
    pub redirect: bool,
    pub title: String,
}

impl Page {
    pub fn write<W: Write>(&self, to: &mut W) -> io::Result<()> {
        ioutil::write_u32(self.link_idx, to)?;
        ioutil::write_u32(self.id, to)?;
        ioutil::write_u32(self.length, to)?;
        ioutil::write_u8(if self.redirect { 1 } else { 0 }, to)?;
        ioutil::write_str(&self.title, to)?;

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
            id,
            length,
            redirect,
            title,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Link {
    pub to: u32,
    pub start: u32,
    pub end: u32,
}

impl Link {
    pub fn write<W: Write>(&self, to: &mut W) -> io::Result<()> {
        ioutil::write_u32(self.to, to)?;
        ioutil::write_u32(self.start, to)?;
        ioutil::write_u32(self.end, to)?;

        Ok(())
    }

    pub fn read<R: Read>(from: &mut R) -> io::Result<Self> {
        let to = ioutil::read_u32(from)?;
        let start = ioutil::read_u32(from)?;
        let end = ioutil::read_u32(from)?;

        Ok(Self { to, start, end })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjacencyList {
    pub pages: Vec<Page>,
    pub links: Vec<Link>,
}

impl AdjacencyList {
    pub fn check_consistency(&self) {
        // Check that all types are large enough
        assert!(self.pages.len() <= u32::MAX as usize, "pages len");
        assert!(self.links.len() <= u32::MAX as usize, "links len");
        for page in &self.pages {
            assert!(page.link_idx <= u32::MAX as u32, "page link_idx");
            assert!(page.id <= u32::MAX as u32, "page id");
            assert!(page.length <= u32::MAX as u32, "page length");
            assert!(page.title.len() <= u8::MAX as usize, "page title len");
        }
        for link in &self.links {
            assert!(link.to <= u32::MAX as u32, "link to");
            assert!(link.start <= u32::MAX as u32, "link start");
            assert!(link.end <= u32::MAX as u32, "link end");
        }

        // Check that all links contain valid indices
        let range = 0..self.pages.len() as u32;
        for link in &self.links {
            if !range.contains(&link.to) {
                panic!("Invalid link detected!");
            }
        }
    }

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
}
