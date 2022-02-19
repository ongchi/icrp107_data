#[macro_export(local)]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[macro_export(local)]
macro_rules! derive_from_str_from_fixed_width {
    ($type:ident) => {
        impl FromStr for $type {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                fixed_width::from_str(s).map_err(|e| Error::Unexpected(e.into()))
            }
        }
    };
}

#[macro_export(local)]
macro_rules! derive_fixed_width_from_fortran_format {
    ($type:ident, $fortran:literal) => {
        impl FixedWidth for $type {
            fn fields() -> Vec<Field> {
                static FIELDS: once_cell::sync::OnceCell<Vec<Field>> =
                    once_cell::sync::OnceCell::new();
                FIELDS
                    .get_or_init(|| {
                        crate::reader::fields_from_fortran_format($fortran)
                            .unwrap()
                            .1
                    })
                    .to_vec()
            }
        }
    };
}
