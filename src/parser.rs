use std::collections::HashMap;

use im::Vector;
use scraper::{ElementRef, Html, Node, Selector};
use ulid::Ulid;

use crate::{document::VDom, vdom::{ElementNode, TextNode, VNode}};


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
    //let template = element.value().attr("template").map(|s| s.to_string());
    //let for_each = element.value().attr("for-each").map(|s| s.to_string());

    let mut attrs = HashMap::new();
    //let mut styles = HashMap::new();

    for attr in element.value().attrs() {
        let key = attr.0.to_string();
        let value = attr.1.to_string();
        if key == "style" {
            let a = parse_styles(&value);
            a.into_iter().for_each(|(k, v)| {attrs.insert(k, v);});
        } else {
            attrs.insert(key, value);
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
                    if el.name() == "script" {
                        return None; // ❌ script ignorieren
                    }
                }
            }
            match child.value() {
                scraper::Node::Text(text) => {
                    if text.trim().is_empty() {
                        None
                    } else {
                        Some(Ok(VNode::Text(TextNode{
                            internal_id: Ulid::new(),
                            id: None,
                            attrs: attrs.clone(),
                            template: text.trim().to_string(),
                            rendered: String::new(),
                        })))
                    }
                },
                scraper::Node::Element(_) => {
                    let el = ElementRef::wrap(child).unwrap();
                    Some(parse_element_internal(&el, inside_template, false, ignore_templates).map(|n| n))
                }
                _ => None,
            }
        })
        .collect::<Result<Vector<_>, _>>()?;

    Ok(VNode::Element( ElementNode {
        internal_id: Ulid::new(),
        id,
        tag,
        attrs,
        children,
    }))
}

pub fn parse_html_to_vdom(html: &str) -> Result<VDom, String> {
    let document = Html::parse_document(html);
    let template_selector = Selector::parse("template").unwrap();
    let body_selector = Selector::parse("body").unwrap();

    let mut templates = HashMap::new();
    let mut id_map = HashMap::new();

    fn index_node(
        node: &VNode,
        id_map: &mut HashMap<String, Ulid>,
    ) {
        match node {
            VNode::Element(ElementNode { internal_id, id, children, .. }) => {
                if let Some(html_id) = id {
                    id_map.insert(html_id.clone(), *internal_id);
                }
                for child in children {
                    index_node(child, id_map);
                }
            }
            VNode::Text( TextNode { internal_id, .. }) => {
                //id_map.insert(*internal_id, Rc::clone(node));
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
        templates.insert(id, vnode);
    }

    let body = document.select(&body_selector).next().ok_or("<body> not found")?;

    let root = parse_element(&body);

    index_node(&root, &mut id_map);


    Ok(VDom {
        root,
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

pub fn load_lua_scripts(html: &str) -> Result<Vec<String>, String> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").unwrap();

    let mut scripts = Vec::new();
    for script in document.select(&script_selector) {
        if let Some(script_content) = script.text().next() {
            //lua.load(script_content).exec()?;
            scripts.push(script_content.to_string());
        }
    }
    Ok(scripts)
}

pub fn parse_color(input: &str) -> Option<[u8; 4]> {
    let hex = input.strip_prefix('#')?;

    match hex.len() {
        6 => {
            // Format: RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r, g, b, 255])
        }
        8 => {
            // Format: RRGGBBAA
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some([r, g, b, a])
        }
        _ => None,
    }
}