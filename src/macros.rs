#[macro_export(inner)]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

#[macro_export(inner)]
macro_rules! derive_from_str {
    ($type:ident) => {
        impl FromStr for $type {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                fixed_width::from_str(s).map_err(|e| Error::Unexpected(e.into()))
            }
        }
    };
}
