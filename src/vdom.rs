use std::{cell::RefCell, collections::HashMap, rc::Rc};

use log::info;
use ulid::Ulid;

#[derive(Debug, Clone)]
pub enum VNode {
    Element {
        internal_id: Ulid,
        id: Option<String>,
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<Rc<RefCell<VNode>>>,
        is_dirty: bool,
        template: Option<String>, // Falls es ein Template-Element ist
        for_each: Option<String>, // Falls eine Schleife definiert wurde (z. B. "weather_data")
        styles: HashMap<String, String>,
    },
    Text {
        internal_id: Ulid,
        is_dirty: bool,
        template: String,
        rendered: String,
    },
}

impl VNode {
    pub fn get_internal_id(&self) -> &Ulid {
        match self {
            VNode::Element { internal_id, .. } => internal_id,
            VNode::Text { internal_id,.. } => internal_id,
        }
    }
}

#[derive(Debug)]
pub struct VDom {
    pub root: Rc<RefCell<VNode>>,
    pub id_index: HashMap<Ulid, Rc<RefCell<VNode>>>,
    pub id_map: HashMap<String, Ulid>, // HTML-ID -> interne Ulid
    pub templates: HashMap<String, Rc<RefCell<VNode>>>,
}


impl VDom {

    //pub fn rebuild_index(&mut self) {
    //    self.id_index.clear();
    //    self.index_node(Rc::clone(&self.root));
    //}

    //fn index_node(&mut self, node: Rc<RefCell<VNode>>) {
    //    let node_ref = node.borrow();
    //    if let VNode::Element { internal_id, children, .. } = &*node_ref {
    //        self.id_index.insert(internal_id.clone(), Rc::clone(&node));
    //        for child in children {
    //            self.index_node(Rc::clone(child));
    //        }
    //    }
    //}

    pub fn get_element_by_id(&self, id: &str) -> Option<Rc<RefCell<VNode>>> {
        self.id_map.get(id).and_then(|id| self.id_index.get(id)).cloned()
    }

    //pub fn insert_element(&mut self, target: Rc<RefCell<VNode>>, child: Rc<RefCell<VNode>>) {
    //    let mut target_node = target.borrow_mut();
    //    if let VNode::Element { children, is_dirty, .. } = &mut *target_node {
    //        children.push(child.clone());
    //        *is_dirty = true;
    //    }
    //    self.index_node(child);
    //}

    pub fn add_element(&mut self, target_id: &str, child: Rc<RefCell<VNode>>) -> Result<(), String> {
        let target_ulid = self.id_map.get(target_id).ok_or("target id not found")?;
        
        // 💡 Clone Rc zuerst, um Borrow-Konflikt zu vermeiden
        let target_node_rc = self.id_index.get(target_ulid).ok_or("target ulid not in index")?.clone();
    
        {
            let mut target_node = target_node_rc.borrow_mut();
            match &mut *target_node {
                VNode::Element { children, is_dirty, .. } => {
                    children.push(Rc::clone(&child));
                    *is_dirty = true;
                }
                _ => return Err("target is not an element".to_string()),
            }
        }
    
        // ✅ Jetzt ist kein RefMut mehr aktiv – sicheres Reindexing möglich
        fn index_deep(
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
                        index_deep(child, id_index, id_map);
                    }
                }
                VNode::Text { internal_id, .. } => {
                    id_index.insert(*internal_id, Rc::clone(node));
                }
            }
        }
    
        index_deep(&child, &mut self.id_index, &mut self.id_map);
        Ok(())
    }
    

    pub fn create_element_from_template(&self, template_id: &str) -> Option<Rc<RefCell<VNode>>> {
        if let Some(template) = self.templates.get(template_id) {
            let cloned_template = Self::deep_clone_node(template, true);
            match &mut *cloned_template.borrow_mut() {
                VNode::Element { tag, is_dirty,..} => {
                    *tag = "div".to_string();
                    *is_dirty = true;
                }
                _ => {}
            }
            Some(cloned_template)
        } else {
            None
        }
    }

    pub fn deep_clone_node(node: &Rc<RefCell<VNode>>, new_id: bool) -> Rc<RefCell<VNode>> {

        let node_ref = node.borrow();
        match &*node_ref {
            VNode::Element {
                internal_id, id, tag, attributes, children, is_dirty, for_each, styles, template
            } => Rc::new(RefCell::new(VNode::Element {
                internal_id: if new_id { Ulid::new() } else {internal_id.clone()},
                id: id.clone(),
                tag: tag.clone(),
                attributes: attributes.clone(),
                children: children.iter().map(|child| Self::deep_clone_node(child, new_id)).collect(),
                is_dirty: *is_dirty,
                for_each: for_each.clone(),
                styles: styles.clone(),
                template: template.clone(),
            })),
            VNode::Text { internal_id, template, rendered, is_dirty } => Rc::new(RefCell::new(VNode::Text {
                internal_id: if new_id { Ulid::new() } else {internal_id.clone()},
                template: template.clone(),
                rendered: rendered.clone(),
                is_dirty: *is_dirty
            })),
        }
    }

    pub fn get_element_by_internal_id(&self, internal_id: &Ulid) -> Option<Rc<RefCell<VNode>>> {
        info!("get_element_by_internal_id: {:?}", internal_id);
        self.id_index.get(internal_id).cloned()
    }
}