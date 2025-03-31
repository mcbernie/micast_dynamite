use std::collections::HashMap;

use ulid::Ulid;

use crate::{vdom::VNode, parse_html_to_vdom};



#[derive(Clone)]
pub struct VDom {
    pub root: VNode,
    pub templates: HashMap<String, VNode>,
    pub id_map: HashMap<String, Ulid>,
}

impl VDom {
    pub fn new(html: &str) -> Result<Self, String> {
        parse_html_to_vdom(html)
    }

    pub fn find_element_by_id(&self, id: &str) -> Option<&VNode> {
        self.id_map.get(id).and_then(|id| self.root.find_by_internal_id(id))
    }

    pub fn find_element_by_internal_id(&self, id: &Ulid) -> Option<&VNode> {
        self.root.find_by_internal_id(id)
    }

    pub fn create_element_from_template(&self, template_id: &str) -> Option<VNode> {
        let mut template_node = self.templates.get(template_id).cloned();

        // iterate over each child of the template_node and generate new internal_id
        if let Some(mut template_node) = template_node.take() {
            template_node.generate_new_ids();
            Some(template_node)
        } else {
            None
        }
    }

    pub fn add_element(&mut self, target_id: &str, child: VNode) -> Result<Ulid, String> {
        let target_ulid = self.id_map.get(target_id).ok_or("target id not found")?;
        let target = self.root.find_by_internal_id_mut(target_ulid).ok_or("target not found")?;
        if let VNode::Element(el) = target {
            let child_id = child.get_internal_id().clone();
            el.children.push_back(child);
            Ok(child_id)
        } else {
            Err("target is not an element".to_string())
        }
    }


}

pub trait FindBy {
    fn find_by_internal_id(&self, id: &Ulid) -> Option<&VNode>;
}

pub trait FindByIdMut {
    fn find_by_internal_id_mut(&mut self, id: &Ulid) -> Option<&mut VNode>;
}
