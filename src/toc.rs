use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, TimeZone};
use failure::*;
use libflate::zlib::Decoder;
use std::fmt;
use std::io::{BufRead, Read, Write};
use xmltree::Element;

#[derive(Fail, Debug)]
pub enum Errors {
    #[fail(display = "<toc> element doesn't exist in Toc.")]
    NoTocElement,
    #[fail(display = "<creation-time> element doesn't exist in Toc.")]
    NoCreationTime,
    #[fail(display = "<checksum> element missing.")]
    NoChecksumElement,
    #[fail(display = "style attribute in <checksum> element missing.")]
    NoChecksumType,
    #[fail(display = "style attribute in <checksum> element missing.")]
    ChecksumOffsetInvalid,
}

#[derive(Debug, Clone)]
pub struct Toc {
    data: Element,
}

impl Toc {
    pub fn from_read<T: Read>(reader: &mut T, expected: usize) -> Result<Toc, Error> {
        let mut decoder = Decoder::new(reader).unwrap();
        let element = Element::parse(&mut decoder)?;

        Ok(Toc { data: element })
    }

    pub fn data(&self) -> &Element {
        &self.data
    }

    pub fn write<W: Write>(&self, writer: W) -> Result<(), xmltree::Error> {
        self.data.write(writer)
    }

    pub fn creation_time(&self) -> Result<NaiveDateTime, Error> {
        let time = self.creation_time_element()?;
        let text = time.text.as_ref().ok_or(Errors::NoCreationTime)?;
        let parsed = NaiveDateTime::parse_from_str(&text, "%Y-%m-%dT%H:%M:%S")?;
        Ok(parsed)
    }

    fn creation_time_element(&self) -> Result<&Element, Errors> {
        self.toc_element()?
            .get_child("creation-time")
            .ok_or(Errors::NoCreationTime)
    }

    pub fn checksum_type(&self) -> Result<&String, Errors> {
        self.checksum_element()?
            .attributes
            .get("style")
            .ok_or(Errors::NoChecksumType)
    }

    pub fn checksum_offset(&self) -> Result<usize, Error> {
        let re = self.checksum_element()?
            .get_child("offset")
            .ok_or(Errors::ChecksumOffsetInvalid)?
            .text
            .as_ref()
            .ok_or(Errors::ChecksumOffsetInvalid)?
            .parse::<usize>()?;
        Ok(re)
    }

    pub fn checksum_size(&self) -> Result<usize, Error> {
        let re = self.checksum_element()?
            .get_child("size")
            .ok_or(Errors::ChecksumOffsetInvalid)?
            .text
            .as_ref()
            .ok_or(Errors::ChecksumOffsetInvalid)?
            .parse::<usize>()?;
        Ok(re)
    }

    fn checksum_element(&self) -> Result<&Element, Errors> {
        self.toc_element()?
            .get_child("checksum")
            .ok_or(Errors::NoChecksumElement)
    }

    fn toc_element(&self) -> Result<&Element, Errors> {
        self.data.get_child("toc").ok_or(Errors::NoTocElement)
    }
}

impl std::fmt::Display for Toc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "creation-time {:?}\n", self.creation_time())?;
        write!(f, "checksum-kind {:?}\n", self.checksum_type())?;
        write!(f, "checksum-offset {:?}\n", self.checksum_offset())?;
        write!(f, "checksum-size {:?}\n", self.checksum_size())
    }
}
