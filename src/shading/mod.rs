use crate::{
    color::ColorSpace,
    data_structures::Rectangle,
    error::PdfResult,
    function::StreamOrDict,
    objects::{Dictionary, Object},
    FromObj, Resolve,
};

use self::{
    axial::AxialShading, coons_patch_mesh::CoonsPatchMeshShading, freeform::FreeformShading,
    function_based::FunctionBasedShading, latticeform::LatticeformShading, radial::RadialShading,
    tensor_product_patch_mesh::TensorProductPatchMeshShading,
};

mod axial;
mod coons_patch_mesh;
mod freeform;
mod function_based;
mod latticeform;
mod radial;
mod tensor_product_patch_mesh;

#[derive(Debug, Clone)]
pub struct ShadingObject<'a> {
    base: BaseShadingDictionary<'a>,
    sub_type: SubtypeShadingDictionary<'a>,
}

impl<'a> FromObj<'a> for ShadingObject<'a> {
    fn from_obj(obj: Object<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let mut stream_or_dict = StreamOrDict::from_obj(obj, resolver)?;

        let base = BaseShadingDictionary::from_dict(stream_or_dict.dict(), resolver)?;
        let sub_type = SubtypeShadingDictionary::from_obj(
            stream_or_dict.into_obj(),
            base.shading_type,
            resolver,
        )?;

        Ok(ShadingObject { base, sub_type })
    }
}

#[derive(Debug, Clone)]
pub enum SubtypeShadingDictionary<'a> {
    FunctionBased(FunctionBasedShading<'a>),
    Axial(AxialShading<'a>),
    Radial(RadialShading<'a>),
    Freeform(FreeformShading<'a>),
    Latticeform(LatticeformShading<'a>),
    CoonsPatchMesh(CoonsPatchMeshShading<'a>),
    TensorProductPatchMesh(TensorProductPatchMeshShading<'a>),
}

impl<'a> SubtypeShadingDictionary<'a> {
    pub fn from_obj(
        obj: Object<'a>,
        sub_type: ShadingType,
        resolver: &mut dyn Resolve<'a>,
    ) -> PdfResult<Self> {
        Ok(match sub_type {
            ShadingType::FunctionBased => SubtypeShadingDictionary::FunctionBased(
                FunctionBasedShading::from_obj(obj, resolver)?,
            ),
            ShadingType::Axial => {
                SubtypeShadingDictionary::Axial(AxialShading::from_obj(obj, resolver)?)
            }
            ShadingType::Radial => {
                SubtypeShadingDictionary::Radial(RadialShading::from_obj(obj, resolver)?)
            }
            ShadingType::Freeform => {
                SubtypeShadingDictionary::Freeform(FreeformShading::from_obj(obj, resolver)?)
            }
            ShadingType::Latticeform => {
                SubtypeShadingDictionary::Latticeform(LatticeformShading::from_obj(obj, resolver)?)
            }
            ShadingType::CoonsPatchMesh => SubtypeShadingDictionary::CoonsPatchMesh(
                CoonsPatchMeshShading::from_obj(obj, resolver)?,
            ),
            ShadingType::TensorProductPatchMesh => {
                SubtypeShadingDictionary::TensorProductPatchMesh(
                    TensorProductPatchMeshShading::from_obj(obj, resolver)?,
                )
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct BaseShadingDictionary<'a> {
    shading_type: ShadingType,

    /// The colour space in which colour values shall be expressed. This may be any device,
    /// CIE-based, or special colour space except a Pattern space
    color_space: ColorSpace<'a>,

    /// An array of colour components appropriate to the colour space, specifying a single
    /// background colour value. If present, this colour shall be used, before any painting
    /// operation involving the shading, to fill those portions of the area to be painted
    /// that lie outside the bounds of the shading object
    ///
    /// In the opaque imaging model, the effect is as if the painting operation were performed
    /// twice: first with the background colour and then with the shading
    ///
    /// The background colour is applied only when the shading is used as part of a shading
    /// pattern, not when it is painted directly with the sh operator
    background: Option<Vec<f32>>,

    /// An array of four numbers giving the left, bottom, right, and top coordinates,
    /// respectively, of the shading's bounding box. The coordinates shall be interpreted
    /// in the shading's target coordinate space. If present, this bounding box shall be
    /// applied as a temporary clipping boundary when the shading is painted, in addition
    /// to the current clipping path and any other clipping boundaries in effect at that
    /// time
    bbox: Option<Rectangle>,

    /// A flag indicating whether to filter the shading function to prevent aliasing artifacts
    ///
    /// The shading operators sample shading functions at a rate determined by the resolution
    /// of the output device. Aliasing can occur if the function is not smooth -- that is, if it
    /// has a high spatial frequency relative to the sampling rate. Anti-aliasing can be
    /// computationally expensive and is usually unnecessary, since most shading functions are
    /// smooth enough or are sampled at a high enough frequency to avoid aliasing effects.
    ///
    /// Anti-aliasing may not be implemented on some output devices, in which case this flag is ignored
    ///
    /// Default value: false
    anti_alias: bool,
}

impl<'a> BaseShadingDictionary<'a> {
    pub fn from_dict(dict: &mut Dictionary<'a>, resolver: &mut dyn Resolve<'a>) -> PdfResult<Self> {
        let shading_type = dict.expect("ShadingType", resolver)?;
        let color_space = dict.expect("ColorSpace", resolver)?;
        let background = dict.get("Background", resolver)?;
        let bbox = dict.get("BBox", resolver)?;
        let anti_alias = dict.get("AntiAlias", resolver)?.unwrap_or(false);

        Ok(Self {
            shading_type,
            color_space,
            background,
            bbox,
            anti_alias,
        })
    }
}

#[pdf_enum(Integer)]
pub enum ShadingType {
    FunctionBased = 1,
    Axial = 2,
    Radial = 3,

    /// Free-form Gouraud-shaded triangle mesh
    Freeform = 4,

    /// Lattice-form Gouraud-shaded triangle mesh
    Latticeform = 5,
    CoonsPatchMesh = 6,
    TensorProductPatchMesh = 7,
}
