use super::{
    parse::TrueTypeParser,
    table::{
        CmapTable, CvtTable, FontDirectory, GlyfTable, Head, LocaTable, MaxpTable, NameTable,
        SimpleGlyph, TableName, TrueTypeGlyph,
    },
    FWord,
};

#[derive(Debug)]
pub struct ParsedTrueTypeFontFile<'a> {
    pub font_directory: FontDirectory,
    pub head: Head,
    pub maxp: MaxpTable,
    pub loca: LocaTable,
    pub cvt: Option<CvtTable>,
    pub cmap: Option<CmapTable>,
    parser: TrueTypeParser<'a>,
}

impl<'a> ParsedTrueTypeFontFile<'a> {
    pub fn new(buffer: &'a [u8]) -> anyhow::Result<Self> {
        let mut parser = TrueTypeParser::new(buffer);

        let font_directory = parser.read_font_directory()?;

        let head = Self::get_head(&mut parser, &font_directory)?;
        let maxp = Self::get_maxp(&mut parser, &font_directory)?;
        let loca = Self::get_loca(&mut parser, &font_directory, head.index_to_loc_format)?;
        let cvt = Self::get_cvt(&mut parser, &font_directory)?;
        let cmap = Self::get_cmap(&mut parser, &font_directory)?;

        Ok(Self {
            font_directory,
            head,
            maxp,
            loca,
            cvt,
            cmap,
            parser,
        })
    }

    fn get_head(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
    ) -> anyhow::Result<Head> {
        let offset = font_directory
            .find_table_offset(TableName::Head.as_tag())
            .unwrap();
        parser.read_head_table(offset as usize)
    }

    fn get_maxp(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
    ) -> anyhow::Result<MaxpTable> {
        let offset = font_directory
            .find_table_offset(TableName::Maxp.as_tag())
            .unwrap();
        parser.read_maxp_table(offset as usize)
    }

    fn get_cvt(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
    ) -> anyhow::Result<Option<CvtTable>> {
        let entry = match font_directory.find_table_entry(TableName::Cvt.as_tag()) {
            Some(e) => e,
            None => return Ok(None),
        };
        parser.read_cvt_table(entry).map(Some)
    }

    fn get_cmap(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
    ) -> anyhow::Result<Option<CmapTable>> {
        let offset = match font_directory.find_table_offset(TableName::Cmap.as_tag()) {
            Some(o) => o,
            None => return Ok(None),
        };

        parser.read_cmap_table(offset as usize).map(Some)
    }

    fn get_loca(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
        loca_format: i16,
    ) -> anyhow::Result<LocaTable> {
        let entry = font_directory
            .find_table_entry(TableName::Loca.as_tag())
            .unwrap();
        parser.read_loca_table(entry.offset as usize, entry.length as usize, loca_format)
    }

    pub fn glyph(&mut self, char_code: u32) -> anyhow::Result<TrueTypeGlyph> {
        let glyph_index = if let Some(cmap) = &self.cmap {
            // todo: be smarter about subtable selection
            cmap.subtables[0].lookup_char_code(char_code)
        } else {
            char_code
        };

        let glyf_entry = self.loca.get_glyf_entry(glyph_index).unwrap();

        let glyf_table_offset = self
            .font_directory
            .find_table_offset(GlyfTable::TAG)
            .unwrap();

        let glyf_offset = glyf_table_offset + glyf_entry.offset;

        if glyf_entry.len == 0 {
            return Ok(TrueTypeGlyph::Simple(SimpleGlyph::empty()));
        }

        self.parser.cursor = glyf_offset as usize;
        let glyf = self.parser.parse_glyph().unwrap();

        // todo: this should be true
        // assert_eq!(
        //     glyf_entry.len as usize,
        //     self.parser.cursor - glyf_offset as usize
        // );

        Ok(glyf)
    }

    pub fn glyphs(&mut self) -> Vec<TrueTypeGlyph> {
        let offset = self
            .font_directory
            .find_table_offset(GlyfTable::TAG)
            .unwrap();
        let num_glyphs = self.maxp.num_glyphs as usize;
        let glyf_table = self
            .parser
            .read_glyf_table(offset as usize, num_glyphs)
            .unwrap();
        glyf_table.glyphs
    }

    /// Number of Storage Area locations
    pub fn max_storage(&self) -> u16 {
        self.maxp.max_storage
    }

    pub fn max_twilight_points(&self) -> u16 {
        self.maxp.max_twilight_points
    }

    pub fn max_points(&self) -> u16 {
        self.maxp.max_points
    }

    pub fn cvt_entry(&self, n: usize) -> Option<FWord> {
        self.cvt
            .as_ref()
            .and_then(|cvt| cvt.entries.get(n).copied())
    }

    pub fn name_table(&mut self) -> anyhow::Result<NameTable> {
        let offset = self
            .font_directory
            .find_table_offset(TableName::Name.as_tag())
            .unwrap();
        self.parser.read_name_table(offset as usize)
    }
}
