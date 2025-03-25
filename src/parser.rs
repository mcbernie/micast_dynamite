use std::{cell::RefCell, collections::HashMap, rc::Rc};

use log::info;
use scraper::{ElementRef, Html, Node, Selector};
use ulid::Ulid;

use crate::vdom::{VDom, VNode};


pub fn parse_element(element: &ElementRef) -> VNode {
    parse_element_internal(element, false, true, true).expect("parse_element failed")
}

pub fn parse_templates(element: &ElementRef) -> VNode {
    parse_element_internal(element, true, true, false).expect("parse_templates failed")
}

fn parse_element_internal(element: &ElementRef, inside_template: bool, start: bool, ignore_templates: bool) -> Result<VNode, String> {
    let tag = element.value().name().to_string();

    if inside_template && tag == "template" && start == false {
        panic!("Nested <template> tags are not allowed");
    }

    let id = element.value().attr("id").map(|s| s.to_string());
    let template = element.value().attr("template").map(|s| s.to_string());
    let for_each = element.value().attr("for-each").map(|s| s.to_string());

    let mut attributes = HashMap::new();
    let mut styles = HashMap::new();

    for attr in element.value().attrs() {
        let key = attr.0.to_string();
        let value = attr.1.to_string();
        if key == "style" {
            styles = parse_styles(&value);
        } else {
            attributes.insert(key, value);
        }
    }

    let children = element
        .children()
        .filter_map(|child| {
            {
                let node = child.value();
                if let Node::Element(el) = node {
                    if el.name() == "template" {
                        return None; // ❌ template ignorieren
                    }
                }
            }
            match child.value() {
                scraper::Node::Text(text) => {
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(Ok(Rc::new(RefCell::new(VNode::Text {
                            internal_id: Ulid::new(),
                            template: text.trim().to_string(),
                            rendered: String::new(),
                            is_dirty: true,
                        }))))
                    }
                },
                scraper::Node::Element(_) => {
                    let el = ElementRef::wrap(child).unwrap();
                    Some(parse_element_internal(&el, inside_template, false, ignore_templates).map(|n| Rc::new(RefCell::new(n))))
                }
                _ => None,
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(VNode::Element {
        internal_id: Ulid::new(),
        id,
        tag,
        attributes,
        styles,
        children,
        is_dirty: true,
        template,
        for_each,
    })
}

pub fn parse_html_to_vdom(html: &str) -> Result<VDom, String> {
    let document = Html::parse_document(html);
    let template_selector = Selector::parse("template").unwrap();
    let body_selector = Selector::parse("body").unwrap();

    let mut templates = HashMap::new();
    let mut id_index = HashMap::new();
    let mut id_map = HashMap::new();

    fn index_node(
        node: &Rc<RefCell<VNode>>,
        id_index: &mut HashMap<Ulid, Rc<RefCell<VNode>>>,
        id_map: &mut HashMap<String, Ulid>,
    ) {
        match &*node.borrow() {
            VNode::Element { internal_id, id, children, .. } => {
                id_index.insert(*internal_id, Rc::clone(node));
                if let Some(html_id) = id {
                    id_map.insert(html_id.clone(), *internal_id);
                }
                for child in children {
                    index_node(child, id_index, id_map);
                }
            }
            VNode::Text { internal_id, .. } => {
                id_index.insert(*internal_id, Rc::clone(node));
            }
        }
    }

    // Templates zuerst extrahieren
    for tpl in document.select(&template_selector) {
        let id = tpl.value().attr("id").ok_or("template without id")?.to_string();

        let count = tpl
        .first_child().unwrap()
        .children()
        .filter_map(|child| {
            ElementRef::wrap(child)
        })
        .count();


        let mut tpl_children = tpl
            .first_child().unwrap()
            .children()
            .filter_map(|child| {
                ElementRef::wrap(child)
            });
        let main_child = tpl_children.next().ok_or("template tag must contain one child element")?;

        if tpl_children.next().is_some() {
            return Err("template tag must contain only one root element".to_string());
        }

        let vnode = parse_templates(&main_child);
        templates.insert(id, Rc::new(RefCell::new(vnode)));
    }

    let body = document.select(&body_selector).next().ok_or("<body> not found")?;

    let root = Rc::new(RefCell::new(parse_element(&body)));

    index_node(&root, &mut id_index, &mut id_map);


    Ok(VDom {
        root,
        id_index,
        id_map,
        templates,
    })
}


fn parse_styles(style_str: &str) -> HashMap<String, String> {
    let mut styles = HashMap::new();
    for rule in style_str.split(';') {
        let mut parts = rule.splitn(2, ':').map(|s| s.trim().to_string());
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            styles.insert(key, value);
        }
    }
    styles
}