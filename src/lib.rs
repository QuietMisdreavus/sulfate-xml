//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A limited XML representation to aid in serializing/deserializing SOAP messages.

#![warn(missing_docs)]

extern crate xml;

use std::borrow::{Borrow, Cow};
use std::io::{Read, Write};
use std::fmt;

use xml::name::OwnedName;
use xml::reader::{self, EventReader};
use xml::reader::XmlEvent as ReaderEvent;
use xml::writer::{self, EventWriter, XmlEvent, EmitterConfig};

/// A representation of an XML element.
#[derive(Debug)]
pub struct Element<'a> {
    /// The name of the element.
    pub name: Name<'a>,
    /// The content of the iterator.
    pub content: Vec<ElemContent<'a>>,
}

/// A representation of the name of an XML element.
#[derive(Debug, PartialEq)]
pub struct Name<'a> {
    /// The "local name of the element.
    pub local_name: Cow<'a, str>,
    /// The fully-qualified URL of the element's namespace, if present.
    pub namespace: Option<Cow<'a, str>>,
    /// The shortened prefix corresponding to the element's namespace, if present. If `namespace`
    /// is present but `prefix` is not, the namespace corresponds to the "default" namespace for
    /// this element and its children.
    pub prefix: Option<Cow<'a, str>>,
}

impl From<OwnedName> for Name<'static> {
    fn from(from: OwnedName) -> Name<'static> {
        Name {
            local_name: from.local_name.into(),
            namespace: from.namespace.map(|ns| ns.into()),
            prefix: from.prefix.map(|p| p.into()),
        }
    }
}

/// A representation of the types of content available to an XML element.
#[derive(Debug)]
pub enum ElemContent<'a> {
    /// Text content.
    Text(Cow<'a, str>),
    /// A child element.
    Child(Element<'a>),
}

/// Helper trait to provide a generalized conversion from a given struct to an `Element`.
pub trait ToXml {
    /// Create an `Element` from the current instance.
    fn to_xml(&self) -> Element;
}

/// Helper trait to convert an `Element` to a given type.
pub trait FromXml: Sized {
    /// Create an instance of `Self` from the given `Element`.
    fn from_xml(&Element) -> Self;
}

impl<'a> Element<'a> {
    /// Create an empty `Element` with no namespace in its name.
    pub fn new<T: Into<Cow<'a, str>>>(name: T) -> Element<'a> {
        Element {
            name: Name {
                local_name: name.into(),
                namespace: None,
                prefix: None,
            },
            content: Vec::new(),
        }
    }

    /// Create an empty `Element` with the given namespace but no prefix.
    pub fn new_default_ns<T, N>(name: T, ns: N) -> Element<'a>
            where T: Into<Cow<'a, str>>,
                  N: Into<Cow<'a, str>>
    {
        Element {
            name: Name {
                local_name: name.into(),
                namespace: Some(ns.into()),
                prefix: None,
            },
            content: Vec::new(),
        }
    }

    /// Create an empty `Element` with the given namespace and prefix.
    pub fn new_ns_prefix<T, N, P>(name: T, ns: N, prefix: P) -> Element<'a>
        where T: Into<Cow<'a, str>>,
              N: Into<Cow<'a, str>>,
              P: Into<Cow<'a, str>>
    {
        Element {
            name: Name {
                local_name: name.into(),
                namespace: Some(ns.into()),
                prefix: Some(prefix.into()),
            },
            content: Vec::new(),
        }
    }

    ///Reads an `Element` from the given stream.
    pub fn from_stream<R: Read>(stream: R) -> reader::Result<Element<'static>> {
        let reader = EventReader::new(stream);

        let mut elem_stack = Vec::<Element<'static>>::new();
        let mut ret = None;

        for event in reader {
            let event = event?;

            match event {
                ReaderEvent::StartElement { name, .. } => {
                    //NOTE: if/when i support attributes, that .. is hiding an `attributes` field
                    let elem = Element {
                        name: name.into(),
                        content: vec![],
                    };
                    elem_stack.push(elem);
                }
                ReaderEvent::EndElement { name } => {
                    let mut child = None;
                    let name: Name = name.into();
                    for i in (0..elem_stack.len()).rev() {
                        if elem_stack[i].name == name {
                            child = Some(elem_stack.remove(i));
                            break;
                        }
                    }

                    if let Some(child) = child {
                        if let Some(head) = elem_stack.last_mut() {
                            head.push_child(child);
                        } else {
                            assert!(ret.is_none());
                            ret = Some(child);
                        }
                    }
                }
                ReaderEvent::Characters(text) => {
                    if let Some(head) = elem_stack.last_mut() {
                        head.push_text(text);
                    }
                }
                _ => {}
            }
        }

        if let Some(head) = ret {
            Ok(head)
        } else {
            Err((&xml::common::TextPosition { row: 0, column: 0 }, "empty stream").into())
        }
    }

    /// Add the given text content to the `Element`.
    pub fn push_text<T: Into<Cow<'a, str>>>(&mut self, content: T) {
        self.content.push(ElemContent::Text(content.into()));
    }

    /// Add a new child `Element` to this `Element`'s children.
    pub fn push_child(&mut self, child: Element<'a>) {
        self.content.push(ElemContent::Child(child));
    }

    /// Serialize this `Element` to the given writer.
    fn serialize<W: Write>(&self, sink: &mut EventWriter<W>) -> writer::Result<()> {
        match (&self.name.namespace, &self.name.prefix) {
            (&Some(ref ns), &Some(ref prefix)) => {
                let full_name = format!("{}:{}", prefix, self.name.local_name);
                sink.write(XmlEvent::start_element(&*full_name)
                                    .ns(prefix.borrow(), ns.borrow()))?;
            },
            (&Some(ref ns), &None) => {
                sink.write(XmlEvent::start_element(self.name.local_name.borrow())
                                    .default_ns(ns.borrow()))?;
            },
            _ => {
                sink.write(XmlEvent::start_element(self.name.local_name.borrow()))?;
            }
        }

        for item in &self.content {
            match item {
                &ElemContent::Text(ref text) => {
                    sink.write(text.borrow())?;
                },
                &ElemContent::Child(ref child) => {
                    child.serialize(sink)?;
                },
            }
        }

        sink.write(XmlEvent::end_element())?;

        Ok(())
    }

    ///Writes this `Element` into the given stream.
    pub fn into_stream<W: Write>(&self, stream: W) -> writer::Result<()> {
        let mut writer = EventWriter::new(stream);

        self.serialize(&mut writer)
    }
}

/// Display impl that formats this `Element` into XML and writes it to the given writer.
///
/// Providing the "alternate" flag by using a formatting flag like `"{:#}"` will pretty-print the
/// XML by adding line breaks and indentation.
///
/// Performance note: Due to the design of the `xml-rs` `EmitterWriter`, this impl writes the XML
/// into a new Vec<u8> before writing it to the stream.
impl<'a> fmt::Display for Element<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let config = EmitterConfig::new().perform_indent(f.alternate());

        let mut buf = Vec::<u8>::new();
        {
            let mut writer = EventWriter::new_with_config(&mut buf, config);

            self.serialize(&mut writer).map_err(|_| fmt::Error)?;
        }
        let result = String::from_utf8(buf).map_err(|_| fmt::Error)?;

        f.write_str(&result)
    }
}
