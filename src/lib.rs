extern crate xml;

use std::borrow::{Borrow, Cow};
use std::io::Write;
use std::mem;

use xml::writer::{self, EventWriter, XmlEvent};

pub struct Element<'a> {
    pub name: Name<'a>,
    pub content: ElemContent<'a>,
}

pub struct Name<'a> {
    pub local_name: Cow<'a, str>,
    pub namespace: Option<Cow<'a, str>>,
    pub prefix: Option<Cow<'a, str>>,
}

pub enum ElemContent<'a> {
    Text(Cow<'a, str>),
    Children(Vec<Element<'a>>),
}

pub trait ToXml {
    fn to_xml(&self) -> Element;
}

pub trait FromXml: Sized {
    fn from_xml(&Element) -> Self;
}

impl<'a> Element<'a> {
    pub fn new<T: Into<Cow<'a, str>>>(name: T) -> Element<'a> {
        Element {
            name: Name {
                local_name: name.into(),
                namespace: None,
                prefix: None,
            },
            content: ElemContent::Children(vec![]),
        }
    }

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
            content: ElemContent::Children(vec![]),
        }
    }

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
            content: ElemContent::Children(vec![]),
        }
    }

    pub fn set_content<T: Into<Cow<'a, str>>>(&mut self, content: T) {
        self.content = ElemContent::Text(content.into());
    }

    pub fn push_child(&mut self, child: Element<'a>) {
        match &mut self.content {
            &mut ElemContent::Children(ref mut children) => {
                children.push(child);
            },
            content => {
                let children = vec![child];
                mem::replace(content, ElemContent::Children(children));
            },
        }
    }

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

        match &self.content {
            &ElemContent::Text(ref text) => {
                sink.write(text.borrow())?;
            },
            &ElemContent::Children(ref children) => {
                for child in children {
                    child.serialize(sink)?;
                }
            },
        }

        sink.write(XmlEvent::end_element())?;

        Ok(())
    }
}
