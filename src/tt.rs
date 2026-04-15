use crate::board::Move;
#[derive(Clone, Copy, PartialEq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}
#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,
    pub depth: u32,
    pub score: f32,
    pub flag: TTFlag,
    pub best_move: Option<Move>,
}
pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024) / entry_size;
        Self {
            entries: vec![None; num_entries],
        }
    }
    pub fn probe(&self, hash: u64) -> Option<TTEntry> {
        let index = (hash as usize) % self.entries.len();
        if let Some(entry) = self.entries[index] {
            if entry.hash == hash {
                return Some(entry);
            }
        }
        None
    }
    pub fn store(
        &mut self,
        hash: u64,
        depth: u32,
        score: f32,
        flag: TTFlag,
        best_move: Option<Move>,
    ) {
        let index = (hash as usize) % self.entries.len();
        self.entries[index] = Some(TTEntry {
            hash,
            depth,
            score,
            flag,
            best_move,
        });
    }
}
