use std::io::{self, Read, Write};

use petgraph::{graph::NodeIndex, Directed, Graph};

use super::{
    adjacency_list::{AdjacencyList, Link, Page},
    info::{LinkInfo, PageInfo},
};

fn write_u8<W: Write>(n: u8, to: &mut W) -> io::Result<()> {
    to.write_all(&n.to_le_bytes())
}

fn read_u8<R: Read>(from: &mut R) -> io::Result<u8> {
    let mut buf = [0_u8; 1];
    from.read_exact(&mut buf)?;
    Ok(u8::from_le_bytes(buf))
}

fn write_u16<W: Write>(n: u16, to: &mut W) -> io::Result<()> {
    to.write_all(&n.to_le_bytes())
}

fn read_u16<R: Read>(from: &mut R) -> io::Result<u16> {
    let mut buf = [0_u8; 2];
    from.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn write_u32<W: Write>(n: u32, to: &mut W) -> io::Result<()> {
    to.write_all(&n.to_le_bytes())
}

fn read_u32<R: Read>(from: &mut R) -> io::Result<u32> {
    let mut buf = [0_u8; 4];
    from.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_str<W: Write>(s: &str, to: &mut W) -> io::Result<()> {
    assert!(s.len() <= u16::MAX as usize);
    write_u16(s.len() as u16, to)?;
    to.write_all(s.as_bytes())?;
    Ok(())
}

fn read_str<R: Read>(from: &mut R) -> io::Result<String> {
    let len = read_u16(from)? as usize;
    let mut buf = vec![0_u8; len];
    from.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf).unwrap())
}

fn write_page<W: Write>(page: &Page<PageInfo>, to: &mut W) -> io::Result<()> {
    write_u32(page.start, to)?;
    write_u32(page.data.id, to)?;
    write_u32(page.data.length, to)?;
    write_u8(if page.data.redirect { 1 } else { 0 }, to)?;
    write_str(&page.data.title, to)?;

    Ok(())
}

pub fn read_page<R: Read>(from: &mut R) -> io::Result<Page<PageInfo>> {
    let start_link_idx = read_u32(from)?;
    let id = read_u32(from)?;
    let length = read_u32(from)?;
    let redirect = read_u8(from)? != 0;
    let title = read_str(from)?;

    Ok(Page {
        start: start_link_idx,
        data: PageInfo {
            id,
            length,
            redirect,
            title,
        },
    })
}

fn write_link<W: Write>(link: &Link<LinkInfo>, to: &mut W) -> io::Result<()> {
    write_u32(link.to, to)?;
    write_u32(link.data.start, to)?;
    write_u32(link.data.len, to)?;
    write_u8(link.data.flags, to)?;

    Ok(())
}

fn read_link<R: Read>(from: &mut R) -> io::Result<Link<LinkInfo>> {
    let to_page_idx = read_u32(from)?;
    let start = read_u32(from)?;
    let len = read_u32(from)?;
    let flags = read_u8(from)?;

    Ok(Link {
        to: to_page_idx,
        data: LinkInfo { start, len, flags },
    })
}

pub fn write_adjacency_list<W: Write>(
    al: &AdjacencyList<PageInfo, LinkInfo>,
    to: &mut W,
) -> io::Result<()> {
    write_u32(al.pages.len() as u32, to)?;
    write_u32(al.links.len() as u32, to)?;

    for page in &al.pages {
        write_page(page, to)?;
    }

    for link in &al.links {
        write_link(link, to)?;
    }

    Ok(())
}

pub fn read_adjacency_list<R: Read>(from: &mut R) -> io::Result<AdjacencyList<PageInfo, LinkInfo>> {
    let n_pages = read_u32(from)?;
    let n_links = read_u32(from)?;

    let mut pages = vec![];
    for _ in 0..n_pages {
        pages.push(read_page(from)?);
    }

    let mut links = vec![];
    for _ in 0..n_links {
        links.push(read_link(from)?);
    }

    Ok(AdjacencyList { pages, links })
}

pub fn read_petgraph<R: Read>(from: &mut R) -> io::Result<Graph<PageInfo, LinkInfo>> {
    let n_pages = read_u32(from)?;
    let n_links = read_u32(from)?;

    let mut graph = Graph::<_, _, Directed, _>::with_capacity(n_pages as usize, n_links as usize);
    let mut page_starts = Vec::with_capacity(n_pages as usize);

    for _ in 0..n_pages {
        let page = read_page(from)?;
        page_starts.push(page.start);
        graph.add_node(page.data);
    }

    let mut ni = 0;
    for ei in 0..n_links {
        while ei >= page_starts.get(ni).copied().unwrap_or(u32::MAX) {
            ni += 1;
        }
        ni -= 1;

        let link = read_link(from)?;
        graph.add_edge(
            NodeIndex::new(ni),
            NodeIndex::new(link.to as usize),
            link.data,
        );
    }

    Ok(graph)
}
