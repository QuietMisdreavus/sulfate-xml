//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A limited XML representation to aid in serializing/deserializing SOAP messages.

#![warn(missing_docs)]

extern crate xml;

use std::borrow::{Borrow, Cow};
use std::io::Write;

use xml::writer::{self, EventWriter, XmlEvent};

/// A representation of an XML element.
pub struct Element<'a> {
    /// The name of the element.
    pub name: Name<'a>,
    /// The content of the iterator.
    pub content: Vec<ElemContent<'a>>,
}

/// A representation of the name of an XML element.
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

/// A representation of the types of content available to an XML element.
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

    /// Add the given text content to the `Element`.
    pub fn push_text<T: Into<Cow<'a, str>>>(&mut self, content: T) {
        self.content.push(ElemContent::Text(content.into()));
    }

    /// Add a new child `Element` to this `Element`'s children.
    pub fn push_child(&mut self, child: Element<'a>) {
        self.content.push(ElemContent::Child(child));
    }

    /// Serialize this `Element` to the given writer.
    pub fn serialize<W: Write>(&self, sink: &mut EventWriter<W>) -> writer::Result<()> {
        match (&self.name.namespace, &self.name.prefix) {
            (&Some(ref ns), &Some(ref prefix)) => {
                sink.write(XmlEvent::start_element(self.name.local_name.borrow())
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
}
