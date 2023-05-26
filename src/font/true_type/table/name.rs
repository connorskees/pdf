use super::tag::TableTag;

#[derive(Debug)]
pub enum TableName {
    Acnt,
    Ankr,
    Avar,
    Bdat,
    Bhed,
    Bloc,
    Bsln,
    Cmap,
    Cvar,
    Cvt,
    Ebsc,
    Fdsc,
    Feat,
    Fmtx,
    Fond,
    Fpgm,
    Fvar,
    Gasp,
    Gcid,
    Glyf,
    Gvar,
    Hdmx,
    Head,
    Hhea,
    Hmtx,
    Just,
    Kern,
    Kerx,
    Lcar,
    Loca,
    Ltag,
    Maxp,
    Meta,
    Mort,
    Morx,
    Name,
    Opbd,
    Os2,
    Post,
    Prep,
    Prop,
    Sbix,
    Trak,
    Vhea,
    Vmtx,
    Xref,
    Zapf,
}

impl TableName {
    pub const fn as_tag(&self) -> TableTag {
        match self {
            Self::Acnt => TableTag::new(*b"acnt"),
            Self::Ankr => TableTag::new(*b"ankr"),
            Self::Avar => TableTag::new(*b"avar"),
            Self::Bdat => TableTag::new(*b"bdat"),
            Self::Bhed => TableTag::new(*b"bhed"),
            Self::Bloc => TableTag::new(*b"bloc"),
            Self::Bsln => TableTag::new(*b"bsln"),
            Self::Cmap => TableTag::new(*b"cmap"),
            Self::Cvar => TableTag::new(*b"cvar"),
            Self::Cvt => TableTag::new(*b"cvt "),
            Self::Ebsc => TableTag::new(*b"EBSC"),
            Self::Fdsc => TableTag::new(*b"fdsc"),
            Self::Feat => TableTag::new(*b"feat"),
            Self::Fmtx => TableTag::new(*b"fmtx"),
            Self::Fond => TableTag::new(*b"fond"),
            Self::Fpgm => TableTag::new(*b"fpgm"),
            Self::Fvar => TableTag::new(*b"fvar"),
            Self::Gasp => TableTag::new(*b"gasp"),
            Self::Gcid => TableTag::new(*b"gcid"),
            Self::Glyf => TableTag::new(*b"glyf"),
            Self::Gvar => TableTag::new(*b"gvar"),
            Self::Hdmx => TableTag::new(*b"hdmx"),
            Self::Head => TableTag::new(*b"head"),
            Self::Hhea => TableTag::new(*b"hhea"),
            Self::Hmtx => TableTag::new(*b"hmtx"),
            Self::Just => TableTag::new(*b"just"),
            Self::Kern => TableTag::new(*b"kern"),
            Self::Kerx => TableTag::new(*b"kerx"),
            Self::Lcar => TableTag::new(*b"lcar"),
            Self::Loca => TableTag::new(*b"loca"),
            Self::Ltag => TableTag::new(*b"ltag"),
            Self::Maxp => TableTag::new(*b"maxp"),
            Self::Meta => TableTag::new(*b"meta"),
            Self::Mort => TableTag::new(*b"mort"),
            Self::Morx => TableTag::new(*b"morx"),
            Self::Name => TableTag::new(*b"name"),
            Self::Opbd => TableTag::new(*b"opbd"),
            Self::Os2 => TableTag::new(*b"OS/2"),
            Self::Post => TableTag::new(*b"post"),
            Self::Prep => TableTag::new(*b"prep"),
            Self::Prop => TableTag::new(*b"prop"),
            Self::Sbix => TableTag::new(*b"sbix"),
            Self::Trak => TableTag::new(*b"trak"),
            Self::Vhea => TableTag::new(*b"vhea"),
            Self::Vmtx => TableTag::new(*b"vmtx"),
            Self::Xref => TableTag::new(*b"xref"),
            Self::Zapf => TableTag::new(*b"Zapf"),
        }
    }
}
