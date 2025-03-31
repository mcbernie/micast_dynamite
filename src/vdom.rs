use im::Vector;
use ulid::Ulid;
use std::collections::HashMap;

use crate::{document::{FindBy, FindByIdMut}, layout::NodeContext, styles::Style};

#[derive(Clone, PartialEq, Debug)]
pub enum VNode {
    Element(ElementNode),
    Text(TextNode),
}

#[derive(Clone, PartialEq, Debug)]
pub struct ElementNode {
    pub internal_id: Ulid,
    pub id: Option<String>,
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub style: Style,
    pub children: Vector<VNode>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TextNode {
    pub internal_id: Ulid,
    pub id: Option<String>,
    pub attrs: HashMap<String, String>,
    pub style: Style,
    pub template: String,
    pub rendered: String,

}

#[derive(Debug)]
pub enum DiffOp {
    Replace(VNode, VNode),
    ChangeAttributes {
        tag: String,
        changes: Vec<(String, Option<String>, Option<String>)>,
    },
    AddChild(usize, VNode),
    RemoveChild(usize),
    PatchChild(usize, Box<DiffOp>),
    Composite(Vec<DiffOp>),
}

/// Vergleicht zwei VNode-Bäume und gibt ein optionales Diff zurück.
pub fn diff_vnode(old: &VNode, new: &VNode) -> Option<DiffOp> {
    match (old, new) {
        (VNode::Text(a), VNode::Text(b)) => {
            if a != b {
                Some(DiffOp::Replace(old.clone(), new.clone()))
            } else {
                None
            }
        }
        (VNode::Element(a), VNode::Element(b)) => {
            if a.tag != b.tag {
                return Some(DiffOp::Replace(old.clone(), new.clone()));
            }

            let mut attr_changes = vec![];
            for key in a.attrs.keys().chain(b.attrs.keys()) {
                let old_val = a.attrs.get(key);
                let new_val = b.attrs.get(key);
                if old_val != new_val {
                    attr_changes.push((
                        key.clone(),
                        old_val.cloned(),
                        new_val.cloned(),
                    ));
                }
            }

            let mut child_diffs = vec![];

            let max_len = a.children.len().max(b.children.len());
            for i in 0..max_len {
                let old_child = a.children.get(i);
                let new_child = b.children.get(i);
                match (old_child, new_child) {
                    (Some(oc), Some(nc)) => {
                        if let Some(diff) = diff_vnode(oc, nc) {
                            child_diffs.push(DiffOp::PatchChild(i, Box::new(diff)));
                        }
                    }
                    (None, Some(nc)) => {
                        child_diffs.push(DiffOp::AddChild(i, nc.clone()));
                    }
                    (Some(_), None) => {
                        child_diffs.push(DiffOp::RemoveChild(i));
                    }
                    (None, None) => {}
                }
            }

            if attr_changes.is_empty() && child_diffs.is_empty() {
                None
            } else {
                let mut ops = vec![];
            
                if !attr_changes.is_empty() {
                    ops.push(DiffOp::ChangeAttributes {
                        tag: a.tag.clone(),
                        changes: attr_changes,
                    });
                }
            
                ops.extend(child_diffs);
            
                Some(DiffOp::Composite(ops))
            }
        }
        _ => Some(DiffOp::Replace(old.clone(), new.clone())),
    }
}

/// Wendet ein DiffOp auf einen VNode an.
pub fn apply_patch(node: &VNode, op: &DiffOp) -> VNode {
    match op {
        DiffOp::Replace(_, new_node) => new_node.clone(),
        DiffOp::ChangeAttributes { tag, changes } => {
            if let VNode::Element(elem) = node {
                let mut new_attrs = elem.attrs.clone();
                for (key, _, new_val) in changes {
                    match new_val {
                        Some(v) => {
                            new_attrs.insert(key.clone(), v.clone());
                        }
                        None => {
                            new_attrs.remove(key);
                        }
                    }
                }
                VNode::Element(ElementNode {
                    internal_id: elem.internal_id.clone(),
                    id: elem.id.clone(),
                    tag: tag.clone(),
                    attrs: new_attrs,
                    style: elem.style.clone(),
                    children: elem.children.clone(),
                })
            } else {
                node.clone()
            }
        }
        DiffOp::AddChild(index, child) => {
            if let VNode::Element(elem) = node {
                let mut new_children = elem.children.clone();
                new_children.insert(*index, child.clone());
                VNode::Element(ElementNode {
                    internal_id: elem.internal_id.clone(),
                    id: elem.id.clone(),
                    tag: elem.tag.clone(),
                    attrs: elem.attrs.clone(),
                    style: elem.style.clone(),
                    children: new_children,
                })
            } else {
                node.clone()
            }
        }
        DiffOp::RemoveChild(index) => {
            if let VNode::Element(elem) = node {
                let mut new_children = elem.children.clone();
                new_children.remove(*index);
                VNode::Element(ElementNode {
                    internal_id: elem.internal_id.clone(),
                    id: elem.id.clone(),
                    tag: elem.tag.clone(),
                    attrs: elem.attrs.clone(),
                    style: elem.style.clone(),
                    children: new_children,
                })
            } else {
                node.clone()
            }
        }
        DiffOp::PatchChild(index, subop) => {
            if let VNode::Element(elem) = node {
                let mut new_children = elem.children.clone();
                if let Some(child) = new_children.get(*index) {
                    let patched = apply_patch(child, subop);
                    new_children.set(*index, patched);
                }
                VNode::Element(ElementNode {
                    internal_id: elem.internal_id.clone(),
                    id: elem.id.clone(),
                    tag: elem.tag.clone(),
                    attrs: elem.attrs.clone(),
                    style: elem.style.clone(),
                    children: new_children,
                })
            } else {
                node.clone()
            }
        }
        DiffOp::Composite(ops) => {
            let mut result = node.clone();
            for op in ops {
                result = apply_patch(&result, op);
            }
            result
        }
    }
}

impl FindBy for VNode {
    fn find_by_internal_id(&self, id: &Ulid) -> Option<&VNode> {
        match self {
            VNode::Element(el) => {
                if &el.internal_id == id {
                    return Some(self);
                }
                for child in &el.children {
                    if let Some(found) = child.find_by_internal_id(id) {
                        return Some(found);
                    }
                }
                None
            }
            VNode::Text(t) => {
                if &t.internal_id == id {
                    Some(self)
                } else {
                    None
                }
            }
        }
    }

}

impl VNode {
    pub fn get_internal_id(&self) -> &Ulid {
        match self {
            VNode::Element(el) => &el.internal_id,
            VNode::Text(t) => &t.internal_id,
        }
    }
    pub fn generate_new_ids(&mut self) {
        match self {
            VNode::Element(el) => {
                el.internal_id = Ulid::new();
                for child in el.children.iter_mut() {
                    child.generate_new_ids();
                }
            }
            VNode::Text(t) => {
                t.internal_id = Ulid::new();
            }
        }
    }

    pub fn get_node_context(&self) -> NodeContext {
        match self {
            VNode::Element(_el) => NodeContext::Element,
            VNode::Text(text) => NodeContext::Text(text.clone())
        }
    }

    pub fn get_style(&self) -> &Style {
        match self {
            VNode::Element(el) => &el.style,
            VNode::Text(t) => &t.style,
        }
    }
}

impl FindByIdMut for VNode {
    fn find_by_internal_id_mut(&mut self, target: &Ulid) -> Option<&mut VNode> {
        if let VNode::Text(t) = self {
            if t.internal_id == *target {
                return Some(self);
            } else {
                return None;
            }
        }

        if let VNode::Element(_) = self {
            let is_match = match self {
                VNode::Element(el) => el.internal_id == *target,
                _ => false,
            };

            if is_match {
                return Some(self);
            }

            match self {
                VNode::Element(el) => {
                    for child in el.children.iter_mut() {
                        if let Some(found) = child.find_by_internal_id_mut(target) {
                            return Some(found);
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
}
