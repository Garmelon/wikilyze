use std::io::{self, Read, Write};

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

impl AdjacencyList {
    pub fn check_consistency(&self) {
        let range = 0..self.pages.len() as u32;
        for link in &self.links {
            if !range.contains(&link.to) {
                panic!("Invalid link detected!");
            }
        }
    }

    pub fn write<W: Write>(&self, mut to: W) -> io::Result<()> {
        let n_pages: u32 = self.pages.len() as u32;
        to.write_all(&n_pages.to_le_bytes())?;

        let n_links: u32 = self.links.len() as u32;
        to.write_all(&n_links.to_le_bytes())?;

        for page in &self.pages {
            to.write_all(&page.link_idx.to_le_bytes())?;
            to.write_all(&page.id.to_le_bytes())?;
            to.write_all(&[if page.redirect { 1 } else { 0 }])?;

            let title_len: u16 = page.title.len() as u16;
            to.write_all(&title_len.to_le_bytes())?;
            to.write_all(page.title.as_bytes())?;
        }

        for link in &self.links {
            to.write_all(&link.to.to_le_bytes())?;
            to.write_all(&link.start.to_le_bytes())?;
            to.write_all(&link.end.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read<R: Read>(mut from: R) -> io::Result<Self> {
        let mut result = Self {
            pages: vec![],
            links: vec![],
        };

        let mut u8_buf = [0_u8; 1];
        let mut u16_buf = [0_u8; 2];
        let mut u32_buf = [0_u8; 4];

        from.read_exact(&mut u32_buf)?;
        let n_pages = u32::from_le_bytes(u32_buf);

        from.read_exact(&mut u32_buf)?;
        let n_links = u32::from_le_bytes(u32_buf);

        for _ in 0..n_pages {
            from.read_exact(&mut u32_buf)?;
            let link_idx = u32::from_le_bytes(u32_buf);

            from.read_exact(&mut u32_buf)?;
            let id = u32::from_le_bytes(u32_buf);

            from.read_exact(&mut u8_buf)?;
            let redirect = u8_buf[0] != 0;

            from.read_exact(&mut u16_buf)?;
            let title_len = u16::from_le_bytes(u16_buf);
            let mut title_bytes = vec![0_u8; title_len as usize];
            from.read_exact(&mut title_bytes)?;
            let title = String::from_utf8(title_bytes).unwrap();

            let page = Page {
                link_idx,
                id,
                title,
                redirect,
            };
            result.pages.push(page);
        }

        for _ in 0..n_links {
            from.read_exact(&mut u32_buf)?;
            let to = u32::from_le_bytes(u32_buf);

            from.read_exact(&mut u32_buf)?;
            let start = u32::from_le_bytes(u32_buf);

            from.read_exact(&mut u32_buf)?;
            let end = u32::from_le_bytes(u32_buf);

            let link = Link { to, start, end };
            result.links.push(link);
        }

        Ok(result)
    }
}
