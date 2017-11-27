use std::cmp::Ordering;
use std::path::Path;

use vfs::{File, MetaData, VFS};
use catalog::proxy::Duplicates;

/// Interface for choosing between files
pub trait Selector<'a, V: VFS> {
    // indicate that you want the max instead of the min or vice versa
    fn reverse(self) -> Self;
    // ctor
    fn new(v: &'a V) -> Self;
    // choose which of the Paths in Duplicates is the "true" (unchanged) one
    fn select<'b>(&'a self, dups: &'b Duplicates) -> &'b Path;
    // helpers to be called by select
    fn min<'b>(&'a self, dups: &'b Duplicates) -> &'b Path;
    fn max<'b>(&'a self, dups: &'b Duplicates) -> &'b Path;
}

/// Choose between files based on their path
pub struct PathSelect<'a, V: VFS + 'a> {
    reverse: bool,
    vfs: &'a V,
}

/// Chose between files based on their creation date
pub struct DateSelect<'a, V: VFS + 'a> {
    reverse: bool,
    vfs: &'a V,
}

impl<'a, V: VFS> Selector<'a, V> for PathSelect<'a, V> {
    fn new(v: &V) -> Self {
        PathSelect { 
            reverse: false,
            vfs: v,
        }
    }
    fn reverse(self) -> Self {
        PathSelect { 
            reverse: true,
            vfs: self.vfs,
        }
    }
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        // select the shallowest element (the path is the shortest)
        if self.reverse {
            self.max(dups)
        } else {
            self.min(dups)
        }
    }
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .min_by(|&a_path, &b_path| {
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
            })
            .unwrap()
    }
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .max_by(|&a_path, &b_path| {
                let a_score = a_path.components().count();
                let b_score = b_path.components().count();
                a_score.cmp(&b_score)
            })
            .unwrap()
    }
}

/*
fn cmp<'a, T: File>(a: &'a T, b: &'a T) -> Ordering {
    let md_a = a.get_metadata().unwrap();
    let md_b = b.get_metadata().unwrap();
    let date_a = md_a.get_creation_time().unwrap();
    let date_b = md_b.get_creation_time().unwrap();
    date_a.cmp(&date_b)
}

impl<V: VFS> Selector<V> for DateSelect<V> {
    fn new(v: V) -> Self {
        DateSelect { 
            reverse: false,
            vfs: v,
        }
    }
    fn reverse(self) -> Self {
        DateSelect { 
            reverse: true,
            vfs: self.vfs,
        }
    }
    fn min<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .map(|path| (path, self.vfs.get_file(path).unwrap()))
            .min_by(|&(_, ref a), &(_, ref b)| cmp(a, b))
            .unwrap()
            .0
    }
    fn max<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        dups.0
            .iter()
            .map(|path| (path, self.vfs.get_file(path).unwrap()))
            .max_by(|&(_, ref a), &(_, ref b)| cmp(a, b))
            .unwrap()
            .0
    }
    fn select<'b>(&self, dups: &'b Duplicates) -> &'b Path {
        // select the newest element (the SystemTime is the largest)
        if self.reverse {
            self.min(dups)
        } else {
            self.max(dups)
        }
    }
}
*/
