use std::{collections::HashMap, rc::Rc};

use crate::{
    assert_empty,
    color::ColorSpace,
    error::PdfResult,
    font::Font,
    objects::{Dictionary, Object},
    shading::ShadingObject,
    xobject::XObject,
    FromObj, Resolve,
};

use self::{graphics_state_parameters::GraphicsStateParameters, pattern::Pattern};

pub mod graphics_state_parameters;
pub mod pattern;

#[derive(Debug, Clone)]
pub struct Resources<'a> {
    /// A dictionary that maps resource names to graphics state parameter
    /// dictionaries
    pub ext_g_state: Option<HashMap<String, GraphicsStateParameters<'a>>>,

    /// A dictionary that maps each resource name to either the name of a
    /// device-dependent color space or an array describing a color space
    pub color_space: Option<HashMap<String, ColorSpace<'a>>>,

    /// A dictionary that maps resource names to pattern objects
    pub pattern: Option<HashMap<String, Rc<Pattern<'a>>>>,

    /// A dictionary that maps resource names to shading dictionaries
    pub shading: Option<HashMap<String, ShadingObject<'a>>>,

    /// A dictionary that maps resource names to external objects
    pub xobject: Option<HashMap<String, XObject<'a>>>,

    /// A dictionary that maps resource names to font dictionaries
    pub font: Option<HashMap<String, Rc<Font<'a>>>>,

    /// An array of predefined procedure set names
    pub proc_set: Option<Vec<ProcedureSet>>,

    pub properties: Option<Dictionary<'a>>,
    // properties: Option<HashMap<String, PropertyList>>,
}

impl<'a> FromObj<'a> for Resources<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut dict = resolver.assert_dict(obj)?;

        let ext_g_state = dict.get("ExtGState", resolver)?;
        let color_space = dict.get("ColorSpace", resolver)?;
        let pattern = dict.get("Pattern", resolver)?;
        let shading = dict.get("Shading", resolver)?;
        let xobject = dict.get("XObject", resolver)?;

        let font = dict.get("Font", resolver)?;

        let proc_set = dict
            .get_arr("ProcSet", resolver)?
            // alternative name for key found in practice (not in spec)
            // todo: do we want to allow this?
            .or(dict.get_arr("ProcSets", resolver)?)
            .map(|proc| {
                proc.into_iter()
                    .map(|proc| ProcedureSet::from_str(&resolver.assert_name(proc)?))
                    .collect::<PdfResult<Vec<ProcedureSet>>>()
            })
            .transpose()?;
        let properties = dict.get_dict("Properties", resolver)?;

        assert_empty(dict);

        Ok(Resources {
            ext_g_state,
            color_space,
            pattern,
            shading,
            xobject,
            font,
            proc_set,
            properties,
        })
    }
}

#[pdf_enum]
pub enum ProcedureSet {
    Pdf = "PDF",
    Text = "Text",
    ImageB = "ImageB",
    ImageC = "ImageC",
    ImageI = "ImageI",
}
