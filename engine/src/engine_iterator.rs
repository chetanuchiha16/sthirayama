use crate::{skiplist::SkipListKV, traits::SkipListIterator};

pub struct EngineIterator<'a, T>
where
    T: Iterator<Item = SkipListKV<Vec<u8>, Vec<u8>>>,
{
    pub merged_sstable_iteator: std::iter::Peekable<T>,
    pub mem_table_iterator: std::iter::Peekable<SkipListIterator<'a, Vec<u8>, Vec<u8>>>,
}

impl<'a, T> EngineIterator<'a, T>
where
    T: Iterator<Item = SkipListKV<Vec<u8>, Vec<u8>>>,
{
    pub fn new(si: T, mi: SkipListIterator<'a, Vec<u8>, Vec<u8>>) -> Self {
        return Self {
            mem_table_iterator: mi.peekable(),
            merged_sstable_iteator: si.peekable(),
        };
    }
}

impl<'a, T> Iterator for EngineIterator<'a, T>
where
    T: Iterator<Item = SkipListKV<Vec<u8>, Vec<u8>>>,
{
    type Item = SkipListKV<Vec<u8>, Vec<u8>>;
    fn next(&mut self) -> Option<Self::Item> {
        // let mem_kv = self.mem_table_iterator.next()?;
        // let ss_kv = self.merged_sstable_iteator.next()?;
        // if ss_kv.key < mem_kv.key {
        //     return Some(ss_kv);
        // } else {
        //     return Some(mem_kv.clone());
        // }
        // if let Some(mk) = self.mem_table_iterator.next()
        //     && let Some(sk) = self.merged_sstable_iteator.next()
        // {
        //     if sk.key < mk.key {
        //         return Some(sk);
        //     } else {
        //         return Some(mk.clone());
        //     }
        // } else {
        //     return None;
        match (
            self.mem_table_iterator.peek(),
            self.merged_sstable_iteator.peek(),
        ) {
            (Some(x), Some(y)) => {
                let ak = &x.key;
                let bk = &y.key;
                if bk < ak {
                    return self.merged_sstable_iteator.next();
                } else {
                    self.mem_table_iterator.next().cloned()
                }
            }
            (Some(_), None) => self.mem_table_iterator.next().cloned(),
            (None, Some(_)) => self.merged_sstable_iteator.next(),
            (None, None) => return None,
        }
    }
}
