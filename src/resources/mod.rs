use std::{collections::HashMap, rc::Rc};

use crate::{
    assert_empty, error::PdfResult, font::Font, objects::Dictionary, shading::ShadingObject,
    xobject::XObject, Resolve,
};

use self::{graphics_state_parameters::GraphicsStateParameters, pattern::Pattern};

pub mod graphics_state_parameters;
pub mod pattern;

#[derive(Debug, Clone)]
pub struct Resources<'a> {
    /// A dictionary that maps resource names to
    /// graphics state parameter dictionaries
    pub ext_g_state: Option<HashMap<String, GraphicsStateParameters<'a>>>,

    /// A dictionary that maps each resource name to
    /// either the name of a device-dependent color
    /// space or an array describing a color space
    // color_space: Option<HashMap<String, ColorSpace>>,
    pub color_space: Option<Dictionary<'a>>,

    /// A dictionary that maps resource names to pattern objects
    pub pattern: Option<HashMap<String, Pattern<'a>>>,

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

impl<'a> Resources<'a> {
    pub(crate) fn from_dict(
        mut dict: Dictionary<'a>,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        let ext_g_state = dict
            .get_dict("ExtGState", resolver)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, value)| {
                        let dict = resolver.assert_dict(value)?;
                        let graphics = GraphicsStateParameters::from_dict(dict, resolver)?;

                        Ok((key, graphics))
                    })
                    .collect::<PdfResult<HashMap<String, GraphicsStateParameters>>>()
            })
            .transpose()?;

        let color_space = dict.get_dict("ColorSpace", resolver)?;

        let pattern = dict
            .get_dict("Pattern", resolver)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, obj)| Ok((key, Pattern::from_object(obj, resolver)?)))
                    .collect::<PdfResult<HashMap<String, Pattern>>>()
            })
            .transpose()?;

        let shading = dict
            .get_dict("Shading", resolver)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, obj)| Ok((key, ShadingObject::from_obj(obj, resolver)?)))
                    .collect::<PdfResult<HashMap<String, ShadingObject>>>()
            })
            .transpose()?;

        let xobject = dict
            .get_dict("XObject", resolver)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, obj)| {
                        Ok((
                            key,
                            XObject::from_stream(resolver.assert_stream(obj)?, resolver)?,
                        ))
                    })
                    .collect::<PdfResult<HashMap<String, XObject>>>()
            })
            .transpose()?;

        let font = dict
            .get_dict("Font", resolver)?
            .map(|dict| {
                dict.entries()
                    .map(|(key, obj)| {
                        Ok((
                            key,
                            Rc::new(Font::from_dict(resolver.assert_dict(obj)?, resolver)?),
                        ))
                    })
                    .collect::<PdfResult<HashMap<String, Rc<Font>>>>()
            })
            .transpose()?;

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
