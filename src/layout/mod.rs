//! # Layout Modul
//!
//! Dieses Modul stellt die Struktur `LayoutMapping` bereit, mit der ein Taffy‑Layout‑Baum
//! aus einem Virtual DOM (VDOM) aufgebaut und inkrementell aktualisiert werden kann.
//!
//! Das Modul bietet folgende Funktionen:
//! - **build_tree**: Rekursiver Aufbau des Taffy‑Baums aus dem VDOM, während ein Mapping von
//!   VDOM‑IDs zu Taffy‑Knoten gepflegt wird.
//! - **apply_diff**: Anwendung von Diff‑Operationen (z. B. Replace, AddChild, RemoveChild),
//!   um gezielt nur die betroffenen Knoten zu aktualisieren.
//! - **compute_layout**: Auslösen der Layout‑Berechnung in Taffy anhand gegebener Container‑Maße.
//!
//! Durch das ID‑Mapping können Änderungen im VDOM effizient auf den Taffy‑Baum übertragen werden,
//! ohne dass der gesamte Baum neu aufgebaut werden muss.

use std::{collections::HashMap, sync::Arc};
use log::warn;
use scraper::Node;
use taffy::prelude::*;
use taffy::geometry::Size;

use crate::{styles::Style, vdom::{TextNode, VNode}, DiffOp, Renderer};

use ulid::Ulid;

pub type MeasureTextFn = Arc<dyn Fn(&str, &crate::styles::Style) -> (u32, u32) + Send + Sync>;

pub enum NodeContext {
    Text(TextNode),
    Element,
}

/// `LayoutMapping` hält die Taffy‑Instanz und ein Mapping von VDOM‑IDs zu Taffy‑Knoten.
/// Dadurch können Änderungen im VDOM (mittels Diff-Operationen) direkt in den Taffy‑Baum übertragen werden.
pub struct LayoutMapping {
    /// Die Taffy‑Layout‑Engine.
    pub taffy: TaffyTree<NodeContext>,
    /// Mapping von VDOM‑interner ID (`Ulid`) zu Taffy‑Knoten (`Node`).
    pub id_map: HashMap<Ulid, NodeId>,

}

impl LayoutMapping {
    /// Erzeugt eine neue `LayoutMapping`‑Instanz mit einer leeren Taffy‑Instanz und einem leeren Mapping.
    pub fn new() -> Self {
        LayoutMapping {
            taffy: TaffyTree::new(),
            id_map: HashMap::new(),
        }
    }

    /// Baut rekursiv den Taffy‑Baum aus dem gegebenen VDOM‑Knoten auf.
    ///
    /// Für jeden VDOM‑Knoten (Element oder Text) wird ein entsprechender Taffy‑Knoten erzeugt und
    /// im Mapping unter der internen ID gespeichert.
    ///
    /// # Argumente
    ///
    /// * `vnode` – Referenz auf den VDOM‑Knoten, der in einen Taffy‑Node überführt werden soll.
    ///
    /// # Rückgabewert
    ///
    /// Gibt den Taffy‑Node zurück, der dem VDOM‑Knoten entspricht.
    pub fn build_tree(&mut self, vnode: &VNode, parent_node: Option<NodeId>) -> NodeId {
        match vnode {
            VNode::Element(element) => {
                // Konvertiere den eigenen Style in einen Taffy‑Style.
                let style = element.style.to_taffy_style();


                // Verarbeite rekursiv alle Kindknoten.
                let children: Vec<NodeId> = element
                    .children
                    .iter()
                    .map(|child| {
                        self.build_tree(child, parent_node)
                    })
                    .collect();

                // hier bekommt die vnode ihre ID zugewiesen
                let node = self.taffy.new_with_children(style, &children)
                    .expect("Fehler beim Erstellen des Taffy-Leaf-Nodes für ein Element");

                let _ = self.taffy.set_node_context(node, Some(NodeContext::Element));

                // Speichere die Zuordnung der internen ID zum Taffy‑Knoten.
                self.id_map.insert(element.internal_id, node);

                if let Some(parent) = parent_node {
                    // Füge den Knoten dem Elternknoten hinzu.
                    self.taffy.add_child(parent, node).expect("Fehler beim Hinzufügen des Knotens zum Elternknoten");
                }

                node
            }
            VNode::Text(text) => {
                
                let style = text.style.to_taffy_style();
                let context = NodeContext::Text(text.clone());
                let node = self.taffy
                    .new_leaf_with_context(style, context)
                    .expect("Fehler beim Erstellen des Taffy-Leaf-Nodes für Text");
                self.id_map.insert(text.internal_id, node);
                if let Some(parent) = parent_node {
                    // Füge den Knoten dem Elternknoten hinzu.
                    self.taffy.add_child(parent, node).expect("Fehler beim Hinzufügen des Knotens zum Elternknoten");
                }
                node
            }
        }
    }

    /// Wendet eine Diff‑Operation auf den Taffy‑Baum an.
    ///
    /// Mit Hilfe der internen ID wird der betroffene Knoten im Mapping gesucht und durch
    /// den neu erzeugten Knoten ersetzt. Andere Diff‑Operationen (z. B. AddChild, RemoveChild)
    /// können später ergänzt werden.
    ///
    /// # Argumente
    ///
    /// * `diff` – Die anzuwendende Diff‑Operation.
    pub fn apply_diff(&mut self, node: &VNode, diff: &DiffOp) {
        let vnode_id = node.get_internal_id();
        let node_id = if let Some(v) = self.id_map.get(vnode_id) {
            Some(v.clone())
        } else {
            None
        };

        match diff {
            DiffOp::Replace(old_vnode, new_vnode) => {
               let internal_id = old_vnode.get_internal_id(); 
                let old_node_id = {
                    if let Some(old_node_id) = self.id_map.get(internal_id) {
                        old_node_id.clone()
                    } else {
                        panic!("Knoten-ID nicht gefunden für Replace");
                    }
                };
                let parent = self.taffy.parent(old_node_id).expect("children not found");
                let children = self.taffy.children(parent).expect("Fehler beim Abrufen der Kinder des Elternknotens");
                let children_position = children.iter().position(|&child| child == old_node_id).unwrap();

                let new_child_id = self.build_tree(new_vnode, None);
                let _ = self.taffy.replace_child_at_index(parent, children_position, new_child_id);
                // Hier wird der alte Knoten entfernt
                self.taffy.remove(old_node_id).expect("Fehler beim Entfernen des Knotens");

                // ich muss hier irgendwie schauen das ggf. die ausgetauschte vnode mehrere children haben kann!
            }
            DiffOp::ChangeAttributes { tag: _tag, changes } => {
                if let Some(current_node_id) = node_id {
                    // Aktuellen Style abrufen und anpassen
                    let current_style = self.taffy
                        .style(current_node_id)
                        .expect("Fehler beim Abrufen des Stils").clone();
                    for (attr, _old, new_value) in changes {
                        if *attr == "style".to_string() {
                            if let Some(style) = new_value {
                                let new_taffy_style = Style::from_str(style).to_taffy_style();
                                if new_taffy_style != current_style {
                                    self.taffy.set_style(node_id.unwrap(), new_taffy_style).expect("couldnt set new style");
                                }
                            }
                        }

                    }
                } else {
                    panic!("Knoten-ID nicht gefunden für ChangeAttributes");
                }
            }
            DiffOp::AddChild(index, new_vnode) => {
                let new_style = new_vnode.get_style().to_taffy_style();
                let new_node_context = new_vnode.get_node_context();
                match self.taffy.new_leaf_with_context(new_style, new_node_context) {
                    Ok(new_node_id) => {
                        
                        if let Some(node_id) = node_id {
                            self.id_map.insert(*vnode_id, new_node_id);
                            let _r = self.taffy.insert_child_at_index(node_id, *index, new_node_id);
                        } else {
                            panic!("Knoten-ID nicht gefunden");
                        }
                    }
                    Err(e) => {
                        panic!("Fehler beim Erstellen des neuen Knotens: {:?}", e);
                    }
                }
            }
            DiffOp::RemoveChild(index) => {
                panic!("remove child");
                if let VNode::Element(elem) = node {
                    if let Some(child) = elem.children.get(*index) {
                        let id = child.get_internal_id();
                        if let Some(node) = self.id_map.remove(id) {
                            self.taffy.remove(node).expect("Fehler beim Entfernen des Knotens");
                        }
                    }

                }
            }
            DiffOp::PatchChild(index, boxed_diff) => {
                match node {
                    VNode::Element(elem) => {
                        if let Some(child) = elem.children.get(*index) {
                            let id = child.get_internal_id();
                            if self.id_map.get(id).is_some() {
                                self.apply_diff(child, boxed_diff);
                            }
                        }
                    }
                    _ => {}
                }
            }
            DiffOp::Composite(ops) => {
                for op in ops {
                    self.apply_diff(node, op);
                }
            }
        }
    }

    /// Führt die Layout‑Berechnung des Taffy‑Baums durch.
    ///
    /// Hierbei wird der Root‑Knoten (angenommen als erster im Mapping) mit der angegebenen Container‑Größe
    /// berechnet.
    ///
    /// # Argumente
    ///
    /// * `container_width` – Die Breite des Containers.
    /// * `container_height` – Die Höhe des Containers.
    /// * `render` – Renderer, der für die Berechnung der Textgröße verwendet wird.
    /// * `ctx` – Kontext, der an den Renderer übergeben wird.
    pub fn compute_layout<R: Renderer>(&mut self, root_node: &NodeId, container_width: f32, container_height: f32, render: &R, ctx: &R::Context) {
        self.taffy
            .compute_layout_with_measure(
                *root_node,
                Size {
                    width: AvailableSpace::Definite(container_width),
                    height: AvailableSpace::Definite(container_height),
                },
                |known_dimesnsions, available_space, _node_id, node_context, _style| {
                    // TODO: Element Based Measurement sollte später ausgelagert werden, eventuell in das Rendering Trait
                    match node_context {
                        Some(NodeContext::Text(text_node)) => {
                            let text = &text_node.rendered;
                            let style = &text_node.style;
                            let size = render.measure_text(ctx, text, style);
                            Size {
                                width: size.0 as f32,
                                height: size.1 as f32,
                            }
                        }
                        _ => {
                            // Hier können Sie den Element-Rendering-Code hinzufügen.
                            Size::ZERO
                        }
                    }
                }
            )
            .expect("Layout-Berechnung fehlgeschlagen");
    }
}
