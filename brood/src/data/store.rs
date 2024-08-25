use std::io::{self, Read, Write};

use super::{
    adjacency_list::{AdjacencyList, Link, LinkIdx, Page, PageIdx},
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
    write_u32(page.start.0, to)?;
    write_u32(page.data.id, to)?;
    write_u32(page.data.length, to)?;
    write_u8(if page.data.redirect { 1 } else { 0 }, to)?;
    write_str(&page.data.title, to)?;

    Ok(())
}

pub fn read_page<R: Read>(from: &mut R) -> io::Result<Page<PageInfo>> {
    let start = LinkIdx(read_u32(from)?);
    let id = read_u32(from)?;
    let length = read_u32(from)?;
    let redirect = read_u8(from)? != 0;
    let title = read_str(from)?;

    Ok(Page {
        start,
        data: PageInfo {
            id,
            length,
            redirect,
            title,
        },
    })
}

fn write_link<W: Write>(link: &Link<LinkInfo>, to: &mut W) -> io::Result<()> {
    write_u32(link.to.0, to)?;
    write_u32(link.data.start, to)?;
    write_u32(link.data.len, to)?;
    write_u8(link.data.flags, to)?;

    Ok(())
}

fn read_link<R: Read>(from: &mut R) -> io::Result<Link<LinkInfo>> {
    let to = PageIdx(read_u32(from)?);
    let start = read_u32(from)?;
    let len = read_u32(from)?;
    let flags = read_u8(from)?;

    Ok(Link {
        to,
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
