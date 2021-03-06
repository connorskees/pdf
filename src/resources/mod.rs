use std::collections::HashMap;

use crate::{assert_empty, error::PdfResult, objects::Dictionary, pdf_enum, Resolve};

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

    shading: Option<Dictionary>,
    xobject: Option<Dictionary>,
    font: Option<Dictionary>,
    proc_set: Option<Vec<ProcedureSet>>,
    properties: Option<Dictionary>,
    // shading: Option<HashMap<String, Shading>>,
    // xobject: Option<HashMap<String, XObject>>,
    // font: Option<HashMap<String, Font>>,
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

        let shading = dict.get_dict("Shading", resolver)?;
        let xobject = dict.get_dict("XObject", resolver)?;
        let font = dict.get_dict("Font", resolver)?;

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