#[macro_export]
macro_rules! pdf_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$doc:meta])*
                $variant:ident = $val:literal
            ),*,
            }
    ) => {
        $(#[$attr])*
        $vis enum $name {
            $(
                $(#[$doc])*
                $variant
            ),*,
        }

        impl $name {
            pub fn from_str(s: &str) -> crate::PdfResult<Self> {
                Ok(match s {
                    $($val => Self::$variant),*,
                    _ => return Err(crate::ParseError::UnrecognizedVariant {
                        ty: stringify!($name),
                        found: s.to_owned(),
                    })
                })
            }
        }
    };
}
