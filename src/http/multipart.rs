use anyhow::{bail, Context, Result};
use log::{trace, warn};
use std::io::{BufRead, BufReader, Cursor, Read};

#[derive(Debug, PartialEq, Eq)]
pub struct MultipartBody {
    pub parts: Vec<MultipartBodyPart>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MultipartBodyPart {
    pub name: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl MultipartBody {
    pub fn from_bytes(boundary: &str, bytes: &[u8]) -> Result<MultipartBody> {
        warn!("for now, only single part multipart body is supported... I know, that does not make sense");
        let cursor = Cursor::new(bytes);
        let mut reader = BufReader::new(cursor);

        let actual_boundary = format!("--{}", boundary);

        let mut read_boundary = String::new();
        let bytes_read = reader.read_line(&mut read_boundary)?;
        trace!("read boundary: {read_boundary:?} ({bytes_read} bytes)");
        if !read_boundary.trim().eq(&actual_boundary) {
            bail!(
                "boundaries do not match: expected '{actual_boundary}' but got '{read_boundary}'"
            );
        }

        let mut content_disposition = String::new();
        let bytes_read = reader.read_line(&mut content_disposition)?;
        trace!("read content disposition: {content_disposition} ({bytes_read} bytes)");
        let content_disposition = ContentDispositionHeader::from_line(&content_disposition)?;
        trace!("parse content_disposition: {content_disposition:?}");

        let mut content_type = String::new();
        let bytes_read = reader.read_line(&mut content_type)?;
        trace!("read content type: {content_type} ({bytes_read} bytes)");
        let content_type = content_type
            .strip_prefix("Content-Type:")
            .context("expected Content-Type prefix")?
            .replace('"', "")
            .trim()
            .to_owned();

        let mut empty_line = String::new();
        let bytes_read = reader.read_line(&mut empty_line)?;
        trace!("read empty line: {bytes_read} bytes");
        if !empty_line.trim().is_empty() {
            bail!("expected to read an empty line but got: {empty_line}");
        }

        let mut buffer = Vec::new();
        let bytes_read = reader.read_to_end(&mut buffer)?;
        trace!("read to end: {bytes_read} bytes");

        let end_boundary = format!("{}--", actual_boundary);
        buffer.truncate(buffer.len() - end_boundary.len());

        let res = MultipartBody {
            parts: vec![MultipartBodyPart {
                name: content_disposition.form_name,
                filename: content_disposition.filename,
                content_type,
                data: buffer,
            }],
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub struct ContentDispositionHeader {
    pub form_name: String,
    pub filename: Option<String>,
}

impl ContentDispositionHeader {
    pub fn from_line(line: &str) -> Result<ContentDispositionHeader> {
        let data = line
            .strip_prefix("Content-Disposition:")
            .context("expected Content-Disposition prefix")?;

        let directives: Vec<_> = data.split(';').map(|d| d.trim()).collect();
        let form_data_dir = *(directives
            .first()
            .context("directives should not be empty")?);
        if !form_data_dir.eq("form-data") {
            bail!(
                "expected first directive to be form-data but got: {}",
                form_data_dir
            );
        }

        let name_dir = directives
            .iter()
            .find(|d| d.starts_with("name="))
            .context("expected the name= directive")?;
        let name = name_dir
            .split_once('=')
            .context("name= directive should have a value")?
            .1
            .replace('"', "");

        let filename_dir = directives.iter().find(|d| d.starts_with("filename="));

        let filename = match filename_dir {
            Some(val) => Some(
                val.split_once('=')
                    .context("filename= directive should have a value")?
                    .1
                    .replace('"', "")
                    .to_owned(),
            ),
            None => None,
        };

        Ok(ContentDispositionHeader {
            form_name: name.to_owned(),
            filename,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_body_single_text_part_ok() {
        let boundary = "ExampleBoundaryString";
        let body = "--ExampleBoundaryString
Content-Disposition: form-data; name=\"description\"
Content-Type: text/html

This is a description
--ExampleBoundaryString--"
            .as_bytes();

        let actual = MultipartBody::from_bytes(boundary, &body).unwrap();
        let expected = MultipartBody {
            parts: vec![MultipartBodyPart {
                name: "description".to_owned(),
                filename: None,
                content_type: "text/html".to_owned(),
                data: "This is a description\n".as_bytes().to_vec(),
            }],
        };

        assert_eq!(expected, actual);
    }

    #[test]
    // TODO: This test checks for err because Multipart support right now is only with single part
    // In normal situation, the body in this function would denote a valid Multipart body
    fn test_mutlipart_body_multiple_parts_is_err() {
        let boundary = "--delimiter123";
        let body = "
--delimiter123
Content-Disposition: form-data; name=\"field1\"

value1
--delimiter123
Content-Disposition: form-data; name=\"field2\"; filename=\"example.txt\"

value2
--delimiter123--"
            .as_bytes();

        assert!(MultipartBody::from_bytes(boundary, &body).is_err());
    }
}
