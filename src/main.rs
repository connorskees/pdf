#![allow(dead_code)]

#[macro_use]
extern crate pdf_macro;

mod actions;
mod annotation;
mod catalog;
mod color;
mod content;
mod data_structures;
mod date;
mod destination;
mod error;
mod file_specification;
mod filter;
mod font;
mod function;
mod geometry;
mod halftones;
mod lex;
mod object_stream;
mod objects;
mod optional_content;
mod page;
mod postscript;
mod render;
mod resolve;
mod resources;
mod shading;
mod stream;
mod structure;
mod trailer;
mod viewer_preferences;
mod xobject;
mod xref;

use std::{borrow::Cow, cell::RefCell, collections::HashMap, io, rc::Rc};

pub(crate) use crate::{objects::FromObj, resolve::Resolve};

use crate::{
    annotation::Annotation,
    catalog::{DocumentCatalog, InformationDictionary},
    content::ContentLexer,
    error::{ParseError, PdfResult},
    filter::decode_stream,
    lex::{LexBase, LexObject},
    object_stream::{ObjectStream, ObjectStreamDict, ObjectStreamParser},
    objects::{Dictionary, Object, ObjectType, Reference},
    page::{InheritablePageFields, PageNode, PageObject, PageTree, PageTreeNode},
    stream::StreamDict,
    trailer::Trailer,
    xref::{ByteOffset, TrailerOrOffset, Xref, XrefParser},
};

pub(crate) const NUMBERS: &[u8] = b"0123456789";

#[track_caller]
pub(crate) fn assert_empty(dict: Dictionary) {
    if !dict.is_empty() {
        todo!("dict not empty: {:#?}", dict);
    }
}

pub fn assert_reference(obj: Object) -> PdfResult<Reference> {
    match obj {
        Object::Reference(r) => Ok(r),
        _ => anyhow::bail!(ParseError::MismatchedObjectType {
            expected: ObjectType::Reference,
        }),
    }
}

impl<'a> LexBase<'a> for Lexer<'_> {
    fn buffer(&self) -> &[u8] {
        &self.file
    }

    fn cursor(&self) -> usize {
        self.pos
    }

    fn cursor_mut(&mut self) -> &mut usize {
        &mut self.pos
    }
}

impl<'a> LexObject<'a> for Lexer<'a> {
    // TODO: move to Lex trait proper and restrain to where Self: Sized + Resolve
    fn lex_dict(&mut self) -> PdfResult<Object<'a>> {
        let dict = self.lex_dict_ignore_stream()?;

        if self.next_matches(b"stream") {
            let stream_dict = StreamDict::from_dict(dict, self)?;
            return Ok(Object::Stream(self.lex_stream(stream_dict)?));
        }

        Ok(Object::Dictionary(dict))
    }
}

pub struct Lexer<'a> {
    file: Vec<u8>,
    pos: usize,
    xref: Rc<Xref>,
    cached_object_streams: HashMap<usize, ObjectStreamParser<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(file: Vec<u8>, xref: Rc<Xref>) -> io::Result<Self> {
        Ok(Self {
            file,
            xref,
            pos: 0,
            cached_object_streams: HashMap::new(),
        })
    }

    fn lex_object_stream(&mut self, byte_offset: usize) -> PdfResult<ObjectStream<'a>> {
        self.pos = byte_offset;
        self.read_obj_prelude()?;

        let object_stream_dict = ObjectStreamDict::from_dict(self.lex_dict_ignore_stream()?, self)?;
        let stream = self
            .lex_stream(object_stream_dict.stream_dict.clone())?
            .stream;

        self.read_obj_trailer()?;

        Ok(ObjectStream {
            stream,
            dict: object_stream_dict,
        })
    }

    fn lex_trailer(&mut self, offset: usize, is_previous: bool) -> PdfResult<Trailer> {
        self.pos = offset;
        self.expect_bytes(b"trailer")?;
        self.skip_whitespace();

        let trailer_dict = self.lex_dict()?;
        let trailer = Trailer::from_dict(self.assert_dict(trailer_dict)?, is_previous, self)?;

        Ok(trailer)
    }

    fn lex_object_from_object_stream(
        &mut self,
        byte_offset: usize,
        reference: Reference,
    ) -> PdfResult<Object<'a>> {
        let parser = match self.cached_object_streams.get_mut(&byte_offset) {
            Some(v) => v,
            None => {
                let ObjectStream { stream, dict } = self.lex_object_stream(byte_offset)?;

                let decoded_stream = decode_stream(&*stream, &dict.stream_dict, self)?;

                let parser = ObjectStreamParser::new(decoded_stream.into_owned(), dict)?;

                self.cached_object_streams
                    .entry(byte_offset)
                    .or_insert(parser)
            }
        };

        parser.parse_object(reference)
    }

    fn lex_page_tree(&mut self, xref: &Xref, root_reference: Reference) -> PdfResult<PageNode<'a>> {
        if xref.get_offset(root_reference, self)?.is_none() {
            return Ok(PageNode::Root(Rc::new(RefCell::new(PageTree {
                kids: Vec::new(),
                pages: HashMap::new(),
                count: 0,
                inheritable_page_fields: InheritablePageFields::new(),
            }))));
        };

        let mut root_dict = self.assert_dict(Object::Reference(root_reference))?;
        let count = root_dict.expect_integer("Count", self)? as usize;
        let raw_kids = root_dict.expect_arr("Kids", self)?;
        let inheritable_page_fields = InheritablePageFields::from_dict(&mut root_dict, self)?;

        root_dict.expect_type("Pages", self, true)?;

        assert_empty(root_dict);

        let root = PageNode::Root(Rc::new(RefCell::new(PageTree {
            count,
            inheritable_page_fields,
            pages: HashMap::new(),
            kids: Vec::new(),
        })));

        let mut pages = HashMap::new();

        pages.insert(root_reference, root.clone());

        let mut page_queue = raw_kids
            .into_iter()
            .map(assert_reference)
            .collect::<PdfResult<Vec<Reference>>>()?;

        while let Some(kid_ref) = page_queue.pop() {
            let mut kid_dict = self.assert_dict(Object::Reference(kid_ref))?;

            match kid_dict.expect_name("Type", self)?.as_ref() {
                "Pages" => {
                    self.lex_page_tree_node(kid_dict, kid_ref, &mut page_queue, &mut pages)?
                }
                "Page" => self.lex_page_object(kid_dict, kid_ref, &mut pages)?,
                found => {
                    anyhow::bail!(ParseError::MismatchedTypeKey {
                        expected: "Page",
                        found: found.to_owned(),
                    })
                }
            };
        }

        match root.clone() {
            PageNode::Root(root) => {
                root.borrow_mut().pages = pages;
            }
            _ => unreachable!(),
        }

        Ok(root)
    }

    fn lex_page_object(
        &mut self,
        mut dict: Dictionary<'a>,
        kid_ref: Reference,
        pages: &mut HashMap<Reference, PageNode<'a>>,
    ) -> PdfResult<()> {
        let parent = dict.expect_reference("Parent")?;
        let last_modified = dict.get("LastModified", self)?;
        let resources = dict.get("Resources", self)?;
        let media_box = dict.get("MediaBox", self)?;
        let crop_box = dict.get("CropBox", self)?;
        let bleed_box = dict.get("BleedBox", self)?;
        let trim_box = dict.get("TrimBox", self)?;
        let art_box = dict.get("ArtBox", self)?;
        let box_color_info = dict.get("BoxColorInfo", self)?;
        let contents = dict.get("Contents", self)?;
        let rotate = dict.get("Rotate", self)?;
        let group = dict.get("Group", self)?;
        let thumb = dict.get("Thumb", self)?;
        let b = dict.get("B", self)?;
        let dur = dict.get("Dur", self)?;
        let trans = dict.get("Trans", self)?;
        let annots = dict.get("Annots", self)?;
        let aa = dict.get("AA", self)?;
        let metadata = None;
        let piece_info = dict.get("PieceInfo", self)?;
        let struct_parents = dict.get("StructParents", self)?;
        let id = dict.get("ID", self)?;
        let pz = dict.get("PZ", self)?;
        let separation_info = dict.get("SeparationInfo", self)?;
        let tabs = dict.get("Tabs", self)?;
        let template_instantiated = None;
        let pres_steps = None;
        let user_unit = None;
        let vp = None;

        assert_empty(dict);

        let parent = pages.get(&parent).unwrap().clone();

        let this_node = PageNode::Leaf(Rc::new(PageObject {
            parent: parent.clone(),
            last_modified,
            resources,
            media_box,
            crop_box,
            bleed_box,
            trim_box,
            art_box,
            box_color_info,
            contents,
            rotate,
            group,
            thumb,
            b,
            dur,
            trans,
            annots,
            aa,
            metadata,
            piece_info,
            struct_parents,
            id,
            pz,
            separation_info,
            tabs,
            template_instantiated,
            pres_steps,
            user_unit,
            vp,
        }));

        pages.insert(kid_ref, this_node.clone());

        match parent {
            PageNode::Node(node) => node.borrow_mut().kids.push(this_node),
            PageNode::Root(node) => node.borrow_mut().kids.push(this_node),
            PageNode::Leaf(..) => todo!("unreachable"),
        }

        Ok(())
    }

    fn lex_page_tree_node(
        &mut self,
        mut dict: Dictionary<'a>,
        kid_ref: Reference,
        page_queue: &mut Vec<Reference>,
        pages: &mut HashMap<Reference, PageNode<'a>>,
    ) -> PdfResult<()> {
        let kids = dict.expect_arr("Kids", self)?;
        let parent = dict.expect_reference("Parent")?;
        let count = dict.expect_integer("Count", self)? as usize;
        let inheritable_page_fields = InheritablePageFields::from_dict(&mut dict, self)?;

        let parent = pages.get(&parent).unwrap().clone();

        let this_node = PageNode::Node(Rc::new(RefCell::new(PageTreeNode {
            count,
            inheritable_page_fields,
            kids: Vec::new(),
            parent: parent.clone(),
        })));

        match parent {
            PageNode::Node(node) => node.borrow_mut().kids.push(this_node.clone()),
            PageNode::Root(node) => node.borrow_mut().kids.push(this_node.clone()),
            PageNode::Leaf(..) => todo!("unreachable"),
        }

        pages.insert(kid_ref, this_node);

        page_queue.append(
            &mut kids
                .into_iter()
                .map(assert_reference)
                .collect::<PdfResult<Vec<Reference>>>()?,
        );

        Ok(())
    }
}

impl<'a> Resolve<'a> for Lexer<'a> {
    fn lex_object_from_reference(&mut self, reference: Reference) -> PdfResult<Object<'a>> {
        let init_pos = self.pos;

        self.pos = match Rc::clone(&self.xref).get_offset(reference, self)? {
            Some(ByteOffset::MainFile(p)) => p,
            Some(ByteOffset::ObjectStream { byte_offset, .. }) => {
                return self.lex_object_from_object_stream(byte_offset, reference);
            }
            None => return Ok(Object::Null),
        };

        self.read_obj_prelude()?;

        let obj = self.lex_object()?;

        self.read_obj_trailer()?;

        self.pos = init_pos;

        Ok(obj)
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    xref: Rc<Xref>,
    trailer: Trailer,
    catalog: DocumentCatalog<'a>,
    page_tree: PageNode<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(p: &'static str) -> PdfResult<Self> {
        let file = std::fs::read(p)?;

        let mut xref_parser = XrefParser::new(&file);
        let xref_and_trailer = xref_parser.read_xref()?;
        let mut xref = Rc::new(xref_and_trailer.xref);
        let mut lexer = Lexer::new(file.clone(), xref.clone())?;

        let trailer = match xref_and_trailer.trailer_or_offset {
            TrailerOrOffset::Offset(offset) => {
                let trailer = lexer.lex_trailer(offset, false)?;
                let mut xref = (*xref).clone();

                let mut prev = trailer.prev;
                while let Some(prev_offset) = prev {
                    let xref_and_trailer = xref_parser.parse_xref_at_offset(prev_offset)?;

                    xref.merge_with_previous(xref_and_trailer.xref);

                    // todo: superfluous clone(?)
                    lexer.xref = Rc::new(xref.clone());

                    let prev_trailer = match xref_and_trailer.trailer_or_offset {
                        TrailerOrOffset::Trailer(trailer) => trailer,
                        TrailerOrOffset::Offset(offset) => lexer.lex_trailer(offset, true)?,
                    };

                    prev = prev_trailer.prev;
                }

                trailer
            }
            TrailerOrOffset::Trailer(trailer) => trailer,
        };

        xref = Rc::clone(&lexer.xref);

        let catalog = DocumentCatalog::from_obj(Object::Reference(trailer.root), &mut lexer)?;

        let page_tree = lexer.lex_page_tree(&xref, catalog.pages)?;

        Ok(Self {
            lexer,
            xref,
            trailer,
            catalog,
            page_tree,
        })
    }

    pub fn info(&mut self) -> PdfResult<Option<InformationDictionary>> {
        Ok(Some(InformationDictionary::from_obj(
            match self.trailer.info {
                Some(r) => Object::Reference(r),
                None => return Ok(None),
            },
            &mut self.lexer,
        )?))
    }

    // todo: make this an iterator
    pub fn pages(&self) -> Vec<Rc<PageObject<'a>>> {
        self.page_tree.leaves()
    }

    pub fn page_annotations(&mut self, page: &PageObject) -> PdfResult<Option<Vec<Annotation>>> {
        if let Some(annots) = &page.annots {
            let annotations = annots
                .iter()
                .map(|&annot| {
                    let obj = self.lexer.lex_object_from_reference(annot)?;

                    Annotation::from_obj(obj, &mut self.lexer)
                })
                .collect::<PdfResult<Vec<Annotation>>>()?;

            return Ok(Some(annotations));
        }

        Ok(None)
    }

    pub fn page_contents(&mut self, page: &PageObject<'a>) -> PdfResult<ContentLexer<'a>> {
        let stream = match &page.contents {
            Some(stream) => stream,
            _ => todo!(),
        };

        // todo: no copy
        Ok(ContentLexer::new(Cow::Owned(
            stream.get_ref(&mut self.lexer)?.combined_buffer.clone(),
        )))
    }
}

fn main() -> PdfResult<()> {
    let mut parser = Parser::new("corpus/Christopher Smith Resume.pdf")?;

    for page in parser.pages() {
        let mut content = parser.page_contents(&*page).unwrap();

        let renderer = render::Renderer::new(&mut content, &mut parser.lexer, Rc::clone(&page));

        renderer.render().unwrap();
        break;
    }

    // dbg!(parser.run().unwrap());

    Ok(())
}
