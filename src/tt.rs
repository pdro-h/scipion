use cozy_chess::Move;

pub const EXACT: u8 = 0;
pub const LOWER: u8 = 1;
pub const UPPER: u8 = 2;

#[derive(Clone, Copy)]
pub struct Entry {
    pub key: u64,
    pub depth: i8,
    pub score: i32,
    pub flag: u8,
    pub mv: Option<Move>,
}

pub struct Tt {
    table: Vec<Entry>,
}

impl Tt {
    pub fn new(size: usize) -> Self {
        let empty = Entry { key: 0, depth: -1, score: 0, flag: EXACT, mv: None };
        Self { table: vec![empty; size] }
    }

    fn index(&self, key: u64) -> usize {
        (key as usize) % self.table.len()
    }

    pub fn probe(&self, key: u64) -> Option<Entry> {
        let e = self.table[self.index(key)];
        if e.key == key { Some(e) } else { None }
    }

    pub fn store(&mut self, key: u64, depth: i32, score: i32, flag: u8, mv: Option<Move>) {
        let i = self.index(key);
        self.table[i] = Entry { key, depth: depth as i8, score, flag, mv };
    }
}
