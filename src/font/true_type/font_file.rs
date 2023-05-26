
use super::{
    table::{
        FontDirectory, GlyfTable, Head,
        LocaTable, MaxpTable,
        TableName, TrueTypeGlyph,
    }, parse::TrueTypeParser,
};


#[derive(Debug)]
pub struct TrueTypeFontFile<'a> {
    font_directory: FontDirectory,
    head: Head,
    maxp: MaxpTable,
    loca: LocaTable,
    parser: TrueTypeParser<'a>,
}

impl<'a> TrueTypeFontFile<'a> {
    pub fn new(buffer: &'a [u8]) -> Option<Self> {
        let mut parser = TrueTypeParser::new(buffer);

        let font_directory = parser.read_font_directory()?;

        let head = Self::get_head(&mut parser, &font_directory)?;
        let maxp = Self::get_maxp(&mut parser, &font_directory)?;
        let loca = Self::get_loca(&mut parser, &font_directory, head.index_to_loc_format)?;

        Some(Self {
            font_directory,
            head,
            maxp,
            loca,
            parser,
        })
    }

    fn get_head(parser: &mut TrueTypeParser, font_directory: &FontDirectory) -> Option<Head> {
        let offset = font_directory
            .find_table_offset(TableName::Head.as_tag())
            .unwrap();
        parser.read_head_table(offset as usize)
    }

    fn get_maxp(parser: &mut TrueTypeParser, font_directory: &FontDirectory) -> Option<MaxpTable> {
        let offset = font_directory
            .find_table_offset(TableName::Maxp.as_tag())
            .unwrap();
        parser.read_maxp_table(offset as usize)
    }

    fn get_loca(
        parser: &mut TrueTypeParser,
        font_directory: &FontDirectory,
        loca_format: i16,
    ) -> Option<LocaTable> {
        let entry = font_directory
            .find_table_entry(TableName::Loca.as_tag())
            .unwrap();
        parser.read_loca_table(entry.offset as usize, entry.length as usize, loca_format)
    }

    pub fn glyph(&mut self, char_code: u32) -> Option<TrueTypeGlyph> {
        let glyf_entry = self.loca.get_glyf_entry(char_code)?;

        let glyf_table_offset = self
            .font_directory
            .find_table_offset(GlyfTable::TAG)
            .unwrap();

        let glyf_offset = glyf_table_offset + glyf_entry.offset;

        self.parser.cursor = glyf_offset as usize;
        let glyf = self.parser.parse_glyph()?;

        dbg!(&glyf);
        todo!()
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

    pub fn max_storage(&self) -> u16 {
        self.maxp.max_storage
    }
}
