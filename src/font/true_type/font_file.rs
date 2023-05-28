use super::{
    parse::TrueTypeParser,
    table::{
        CvtTable, FontDirectory, GlyfTable, Head, LocaTable, MaxpTable, NameTable, TableName,
        TrueTypeGlyph,
    },
    FWord,
};

#[derive(Debug)]
pub struct ParsedTrueTypeFontFile<'a> {
    font_directory: FontDirectory,
    head: Head,
    maxp: MaxpTable,
    loca: LocaTable,
    cvt: CvtTable,
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

        Ok(Self {
            font_directory,
            head,
            maxp,
            loca,
            cvt,
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
    ) -> anyhow::Result<CvtTable> {
        let entry = font_directory
            .find_table_entry(TableName::Cvt.as_tag())
            .unwrap();
        parser.read_cvt_table(entry)
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
        let glyf_entry = self.loca.get_glyf_entry(char_code).unwrap();

        let glyf_table_offset = self
            .font_directory
            .find_table_offset(GlyfTable::TAG)
            .unwrap();

        let glyf_offset = glyf_table_offset + glyf_entry.offset;

        self.parser.cursor = glyf_offset as usize;
        let glyf = self.parser.parse_glyph().unwrap();

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
        self.cvt.entries.get(n).copied()
    }

    pub fn name_table(&mut self) -> anyhow::Result<NameTable> {
        let offset = self
            .font_directory
            .find_table_offset(TableName::Name.as_tag())
            .unwrap();
        self.parser.read_name_table(offset as usize)
    }
}
