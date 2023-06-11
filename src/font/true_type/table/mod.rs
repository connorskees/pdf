#![allow(unused_imports)]

mod cmap;
mod cvt;
mod font_directory;
mod glyf;
mod head;
mod loca;
mod maxp;
mod name;
mod table_name;
mod tag;

pub(super) use cmap::{Cmap8Group, CmapSubtable, CmapTable};
pub(super) use cvt::CvtTable;
pub(super) use font_directory::{
    DirectoryTableEntry, FontDirectory, OffsetSubtable, TableDirectory,
};
pub(super) use glyf::{
    CompoundGlyphPartDescription, GlyfTable, GlyphDescription, OutlineFlag, SimpleGlyph,
    TrueTypeGlyph,
};
pub(super) use head::{Head, HeadFlags, MacStyle};
pub(super) use loca::LocaTable;
pub(super) use maxp::{MaxpPostscriptTable, MaxpTable};
pub(super) use name::{NameRecord, NameTable};
pub(super) use table_name::TableName;
pub(super) use tag::TableTag;
