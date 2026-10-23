// ██╗  ██╗███╗   ███╗██╗
// ╚██╗██╔╝████╗ ████║██║
//  ╚███╔╝ ██╔████╔██║██║
//  ██╔██╗ ██║╚██╔╝██║██║
// ██╔╝ ██╗██║ ╚═╝ ██║███████╗
// ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝
// FILE: xml

//! A small element tree over quick-xml: the scene XML is read into nodes (name, attributes, text,
//! children) and walked by `scene`. Enough for MVR; never a general XML model.
//!
//! Owns:          Node, parse.
//! Doesn't own:   what the elements mean (scene.rs).
//! Entry points:  parse, Node::attr, child, children, text_of.
//! Decisions:     §34 "34g4 patch hardened" 10.

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::MvrError;


// ████████╗██╗   ██╗██████╗ ███████╗███████╗
// ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝██╔════╝
//    ██║    ╚████╔╝ ██████╔╝█████╗  ███████╗
//    ██║     ╚██╔╝  ██╔═══╝ ██╔══╝  ╚════██║
//    ██║      ██║   ██║     ███████╗███████║
//    ╚═╝      ╚═╝   ╚═╝     ╚══════╝╚══════╝
// PART: TYPES

/// One element: its name, attributes, own text and children in order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub text: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// The first child of that name.
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Every child of that name, in order.
    pub fn children(&self, name: &str) -> impl Iterator<Item = &Node> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// The trimmed text of the first child of that name.
    pub fn text_of(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.trim()).filter(|t| !t.is_empty())
    }
}


// ██╗      ██████╗  ██████╗ ██╗ ██████╗
// ██║     ██╔═══██╗██╔════╝ ██║██╔════╝
// ██║     ██║   ██║██║  ███╗██║██║
// ██║     ██║   ██║██║   ██║██║██║
// ███████╗╚██████╔╝╚██████╔╝██║╚██████╗
// ╚══════╝ ╚═════╝  ╚═════╝ ╚═╝ ╚═════╝
// PART: LOGIC

/// The document's root element.
pub fn parse(bytes: &[u8]) -> Result<Node, MvrError> {
    let mut reader = Reader::from_reader(bytes);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Node> = vec![Node { name: "#document".into(), ..Node::default() }];
    let mut buf = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buf).map_err(|e| MvrError::BadXml(e.to_string()))?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let mut node = Node { name: String::from_utf8_lossy(e.name().as_ref()).into_owned(), ..Node::default() };
                for a in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
                    let value = a.unescape_value().map(|v| v.into_owned()).unwrap_or_default();
                    node.attrs.push((key, value));
                }
                if empty {
                    stack.last_mut().expect("the document").children.push(node);
                } else {
                    stack.push(node);
                }
            }
            Event::End(_) => {
                let node = stack.pop().expect("a started element");
                stack.last_mut().expect("the document").children.push(node);
            }
            Event::Text(t) => {
                let text = t.unescape().map(|v| v.into_owned()).unwrap_or_default();
                stack.last_mut().expect("an element").text.push_str(&text);
            }
            Event::CData(c) => {
                let text = String::from_utf8_lossy(&c).into_owned();
                stack.last_mut().expect("an element").text.push_str(&text);
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    let document = stack.pop().ok_or_else(|| MvrError::BadXml("no root element".into()))?;
    if !stack.is_empty() {
        return Err(MvrError::BadXml("an element isn't closed".into()));
    }
    document.children.into_iter().next().ok_or_else(|| MvrError::BadXml("no root element".into()))
}
