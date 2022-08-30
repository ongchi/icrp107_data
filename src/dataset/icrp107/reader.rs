use fixed_width::FieldSet;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use super::super::reader::FileReader;
use super::ndx::{Attribute, NdxEntry};

use crate::error::Error;
use crate::primitive::Nuclide;
use crate::regex;

pub struct IndexReader {
    reader: FileReader,
}

impl IndexReader {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let reader = FileReader::new(path)?.skip_lines(1)?;
        Ok(Self { reader })
    }

    pub fn read(&mut self) -> Result<HashMap<Nuclide, Attribute>, Error> {
        let mut ndx = HashMap::new();

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let row: NdxEntry =
                fixed_width::from_str(&buf).map_err(|e| Error::Unexpected(e.into()))?;
            ndx.insert(row.nuclide, row.into());
        }

        Ok(ndx)
    }
}

pub struct SpectrumReader<T> {
    reader: FileReader,
    _marker: std::marker::PhantomData<T>,
}

impl<T> SpectrumReader<T>
where
    T: FromStr<Err = Error>,
{
    pub fn new(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            reader: FileReader::new(path)?,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn read(&mut self) -> Result<HashMap<Nuclide, Vec<T>>, Error> {
        let mut inner = HashMap::new();

        let mut buf = String::new();
        while self.reader.read_line(&mut buf)? != 0 {
            let nuclide: Nuclide = (&buf[0..7]).parse()?;
            let records = &buf[7..].replace('\0', " ");
            let records = records.split_whitespace().last().ok_or_else(|| {
                Error::Unexpected(anyhow::anyhow!("failed to get spectrum for {}", nuclide))
            })?;
            let records = records
                .parse()
                .map_err(|_| Error::InvalidInteger(records.to_string()))?;

            let mut spectrum = vec![];
            for _ in 0..(records) {
                self.reader.read_line(&mut buf)?;
                spectrum.push(buf.parse()?);
            }
            inner.insert(nuclide, spectrum);
        }

        Ok(inner)
    }
}

pub(crate) fn fields_from_fortran_format(
    fmt: &str,
    offset: usize,
) -> Result<(FieldSet, usize), String> {
    let re = regex!(
        r"(?P<repeat>\d*)(?:(?P<type>[a-z]{1,2})|(?P<nested>\([^\(\)]+?\)))(?:(?P<length>\d+)(?:\.\d+)?)?"
    );
    let mut start = 0;
    let mut fields = FieldSet::Seq(vec![]);

    let fmt: String = fmt
        .trim_matches(|c: char| c.is_whitespace() || c == '(' || c == ')')
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let captures: Vec<_> = re.captures_iter(&fmt).collect();
    if captures.is_empty() {
        Err("invalid fortran format".to_string())
    } else {
        for cap in captures.into_iter() {
            let rep: usize = cap
                .name("repeat")
                .map(|m| m.as_str().parse().unwrap_or(1))
                .unwrap();

            match cap.name("type") {
                Some(m) => {
                    let typ = m.as_str();
                    let len: usize = cap
                        .name("length")
                        .map(|m| m.as_str().parse().unwrap())
                        .unwrap_or(1);
                    match typ {
                        "x" => start += rep * len,
                        _ => {
                            for _ in 0..rep {
                                fields = fields.extend(FieldSet::new_field(
                                    offset + start..offset + start + len,
                                ));
                                start += len
                            }
                        }
                    }
                }
                None => match cap.name("nested") {
                    Some(m) => {
                        if cap.name("length").is_some() {
                            return Err("invalid fortran format".to_string());
                        } else {
                            let fmt = m.as_str();
                            let mut nested_fields = vec![];
                            for _ in 0..rep {
                                let (nested_field, len) =
                                    fields_from_fortran_format(fmt, offset + start)?;
                                nested_fields.push(nested_field);
                                start += len;
                            }
                            fields = fields.append(FieldSet::Seq(nested_fields))
                        }
                    }
                    None => return Err("invalid fortran format".to_string()),
                },
            }
        }

        Ok((fields, start))
    }
}

#[cfg(test)]
mod test {
    use fixed_width::{field, field_seq, FieldConfig};

    use super::fields_from_fortran_format;

    #[test]
    fn test_fields_from_fortran_format() {
        let fortran_format = "(a10,2i10,3f10.0,4e10.0)";
        let (fields, len) = fields_from_fortran_format(fortran_format, 0).unwrap();
        let fields = fields.flatten();

        assert_eq!(len, 100);
        for i in 0..10 {
            assert_eq!(
                format!("{:?}", fields[i]),
                format!("{:?}", FieldConfig::new(i * 10..(i + 1) * 10))
            );
        }
    }

    #[test]
    fn test_nested_fields_from_fortran_format() {
        let fortran_format =
            "(a7,a10,a8,28x,4(a7,6x,e11.0,1x),f7.0,2f8.0,3i4,i5,i4,e11.0,e10.0,e9.0)";
        let (fields, _len) = fields_from_fortran_format(fortran_format, 0).unwrap();

        let complex_fields = field_seq![
            field!(0..7),
            field!(7..17),
            field!(17..25),
            field_seq![
                field_seq![field!(53..60), field!(66..77)],
                field_seq![field!(78..85), field!(91..102)],
                field_seq![field!(103..110), field!(116..127)],
                field_seq![field!(128..135), field!(141..152)],
            ],
            field!(153..160),
            field!(160..168),
            field!(168..176),
            field!(176..180),
            field!(180..184),
            field!(184..188),
            field!(188..193),
            field!(193..197),
            field!(197..208),
            field!(208..218),
            field!(218..227),
        ];

        assert_eq!(format!("{:?}", fields), format!("{:?}", complex_fields));
    }
}
