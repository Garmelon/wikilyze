#[derive(Debug, Clone)]
pub struct PageInfo {
    pub id: u32,
    pub title: String,
    pub length: u32,
    pub redirect: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LinkInfo {
    pub start: u32,
    pub len: u32,
    pub flags: u8,
}

impl LinkInfo {
    pub fn in_parens(self) -> bool {
        self.flags & 0b1 != 0
    }

    pub fn in_structure(self) -> bool {
        self.flags & 0b10 != 0
    }
}
