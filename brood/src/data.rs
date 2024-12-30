use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

use crate::graph::{EdgeIdx, Graph, NodeIdx};

#[derive(Debug, Clone)]
pub struct Page {
    pub id: u32,
    pub title: String,
    pub length: u32,
    pub redirect: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Link {
    pub start: u32,
    pub len: u32,
    pub flags: u8,
}

impl Link {
    pub fn in_parens(self) -> bool {
        self.flags & 0b1 != 0
    }

    pub fn in_structure(self) -> bool {
        self.flags & 0b10 != 0
    }
}

struct Store<'a, W>(&'a mut W);

fn write_u8(w: &mut impl Write, n: u8) -> io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn read_u8(r: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0_u8; 1];
    r.read_exact(&mut buf)?;
    Ok(u8::from_le_bytes(buf))
}

fn write_u16(w: &mut impl Write, n: u16) -> io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn read_u16(r: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0_u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn write_u32(w: &mut impl Write, n: u32) -> io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0_u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_str(w: &mut impl Write, s: &str) -> io::Result<()> {
    assert!(s.len() <= u16::MAX as usize);
    write_u16(w, s.len() as u16)?;
    w.write_all(s.as_bytes())?;
    Ok(())
}

fn read_str(r: &mut impl Read) -> io::Result<String> {
    let len = read_u16(r)? as usize;
    let mut buf = vec![0_u8; len];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf).unwrap())
}

fn write_page(w: &mut impl Write, page: &Page) -> io::Result<()> {
    write_u32(w, page.id)?;
    write_u32(w, page.length)?;
    write_u8(w, if page.redirect { 1 } else { 0 })?;
    write_str(w, &page.title)?;
    Ok(())
}

pub fn read_page(r: &mut impl Read) -> io::Result<Page> {
    Ok(Page {
        id: read_u32(r)?,
        length: read_u32(r)?,
        redirect: read_u8(r)? != 0,
        title: read_str(r)?,
    })
}

fn write_link(w: &mut impl Write, link: &Link) -> io::Result<()> {
    write_u32(w, link.start)?;
    write_u32(w, link.len)?;
    write_u8(w, link.flags)?;
    Ok(())
}

fn read_link(r: &mut impl Read) -> io::Result<Link> {
    Ok(Link {
        start: read_u32(r)?,
        len: read_u32(r)?,
        flags: read_u8(r)?,
    })
}

fn write(w: &mut impl Write, pages: &[Page], links: &[Link], graph: &Graph) -> io::Result<()> {
    assert!(pages.len() < u32::MAX as usize);
    assert!(links.len() < u32::MAX as usize);
    assert_eq!(pages.len(), graph.nodes.len());
    assert_eq!(links.len(), graph.edges.len());
    write_u32(w, pages.len() as u32)?;
    write_u32(w, links.len() as u32)?;

    for page in pages {
        write_page(w, page)?;
    }

    for link in links {
        write_link(w, link)?;
    }

    for node in &graph.nodes {
        write_u32(w, node.0)?;
    }

    for edge in &graph.edges {
        write_u32(w, edge.0)?;
    }

    Ok(())
}

fn read(r: &mut impl Read) -> io::Result<(Vec<Page>, Vec<Link>, Graph)> {
    let n_pages = read_u32(r)?;
    let n_links = read_u32(r)?;

    let mut pages = Vec::with_capacity(n_pages as usize);
    let mut links = Vec::with_capacity(n_links as usize);
    let mut graph = Graph::with_capacity(n_pages as usize, n_links as usize);

    for _ in 0..n_pages {
        pages.push(read_page(r)?);
    }

    for _ in 0..n_links {
        links.push(read_link(r)?);
    }

    for _ in 0..n_pages {
        graph.nodes.push(EdgeIdx(read_u32(r)?));
    }

    for _ in 0..n_links {
        graph.edges.push(NodeIdx(read_u32(r)?));
    }

    assert_eq!(pages.len(), graph.nodes.len());
    assert_eq!(links.len(), graph.edges.len());
    graph.check_consistency();
    Ok((pages, links, graph))
}

pub fn write_to_file(path: &Path, pages: &[Page], links: &[Link], graph: &Graph) -> io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    write(&mut file, pages, links, graph)
}

pub fn read_from_file(path: &Path) -> io::Result<(Vec<Page>, Vec<Link>, Graph)> {
    let mut file = BufReader::new(File::open(path)?);
    read(&mut file)
}
