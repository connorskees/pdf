use std::collections::HashMap;

use crate::{
    assert_empty, error::PdfResult, font::Font, objects::Dictionary, pdf_enum,
    shading::ShadingObject, xobject::XObject, Resolve,
};

use self::{graphics_state_parameters::GraphicsStateParameters, pattern::Pattern};

pub mod graphics_state_parameters;
pub mod pattern;

#[derive(Debug)]
pub struct Resources {
    /// A dictionary that maps resource names to
    /// graphics state parameter dictionaries
    ext_g_state: Option<HashMap<String, GraphicsStateParameters>>,

    /// A dictionary that maps each resource name to
    /// either the name of a device-dependent color
    /// space or an array describing a color space
    // color_space: Option<HashMap<String, ColorSpace>>,
    color_space: Option<Dictionary>,

    /// A dictionary that maps resource names to pattern objects
    pattern: Option<HashMap<String, Pattern>>,

    /// A dictionary that maps resource names to shading dictionaries
    shading: Option<HashMap<String, ShadingObject>>,

    /// A dictionary that maps resource names to external objects
    xobject: Option<HashMap<String, XObject>>,

    /// A dictionary that maps resource names to font dictionaries
    font: Option<HashMap<String, Font>>,

    /// An array of predefined procedure set names
    proc_set: Option<Vec<ProcedureSet>>,

    properties: Option<Dictionary>,
    // properties: Option<HashMap<String, PropertyList>>,
}

impl Resources {
    pub(crate) fn from_dict(mut dict: Dictionary, resolver: &mut dyn Resolve) -> PdfResult<Self> {
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
                        Ok((key, Font::from_dict(resolver.assert_dict(obj)?, resolver)?))
                    })
                    .collect::<PdfResult<HashMap<String, Font>>>()
            })
            .transpose()?;
        // let font = dict.get_dict("Font", resolver)?;

        // dbg!(&font);

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

pdf_enum!(
    #[derive(Debug)]
    pub enum ProcedureSet {
        Pdf = "PDF",
        Text = "Text",
        ImageB = "ImageB",
        ImageC = "ImageC",
        ImageI = "ImageI",
    }
);
