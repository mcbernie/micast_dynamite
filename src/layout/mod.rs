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

use std::collections::HashMap;
use taffy::prelude::*;
use taffy::geometry::Size;

use crate::{vdom::VNode, DiffOp};

use ulid::Ulid;

/// `LayoutMapping` hält die Taffy‑Instanz und ein Mapping von VDOM‑IDs zu Taffy‑Knoten.
/// Dadurch können Änderungen im VDOM (mittels Diff-Operationen) direkt in den Taffy‑Baum übertragen werden.
pub struct LayoutMapping {
    /// Die Taffy‑Layout‑Engine.
    pub taffy: TaffyTree,
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
    pub fn build_tree(&mut self, vnode: &VNode) -> NodeId {
        match vnode {
            VNode::Element(element) => {
                // Verarbeite rekursiv alle Kindknoten.
                let children: Vec<NodeId> = element
                    .children
                    .iter()
                    .map(|child| self.build_tree(child))
                    .collect();
                // Konvertiere den eigenen Style in einen Taffy‑Style.
                let style = element.style.to_taffy_style();
                let node = self.taffy
                    .new_with_children(style, &children)
                    .expect("Fehler beim Erstellen des Taffy-Nodes für ein Element");
                // Speichere die Zuordnung der internen ID zum Taffy‑Knoten.
                self.id_map.insert(element.internal_id, node);
                node
            }
            VNode::Text(text) => {
                let style = text.style.to_taffy_style();
                let node = self.taffy
                    .new_leaf(style)
                    .expect("Fehler beim Erstellen des Taffy-Leaf-Nodes für Text");
                self.id_map.insert(text.internal_id, node);
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
    pub fn apply_diff(&mut self, diff: DiffOp) {
        match diff {
            DiffOp::Replace(old_vnode, new_vnode) => {
                // Ermittle die interne ID des alten VDOM-Knotens.
                let id = match &old_vnode {
                    VNode::Element(el) => el.internal_id,
                    VNode::Text(t) => t.internal_id,
                };
                if let Some(_old_node) = self.id_map.remove(&id) {
                    // Erstelle den neuen Knoten und aktualisiere das Mapping.
                    let new_node = self.build_tree(&new_vnode);
                    self.id_map.insert(id, new_node);
                }
            }
            DiffOp::ChangeAttributes { tag: _tag, changes: _changes } => {
                // TODO: Aktualisiere die Attribute (z. B. den Style) des betroffenen Knotens in Taffy.
            }
            DiffOp::AddChild(_index, _new_vnode) => {
                // TODO: Implementiere das Hinzufügen eines neuen Kindes und aktualisiere das Mapping.
            }
            DiffOp::RemoveChild(_index) => {
                // TODO: Implementiere das Entfernen eines Kindes und aktualisiere das Mapping.
            }
            DiffOp::PatchChild(_index, boxed_diff) => {
                self.apply_diff(*boxed_diff);
            }
            DiffOp::Composite(ops) => {
                for op in ops {
                    self.apply_diff(op);
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
    pub fn compute_layout(&mut self, container_width: f32, container_height: f32) {
        if let Some(&root_node) = self.id_map.values().next() {
            self.taffy
                .compute_layout(
                    root_node,
                    Size {
                        width: AvailableSpace::Definite(container_width),
                        height: AvailableSpace::Definite(container_height),
                    },
                )
                .expect("Layout-Berechnung fehlgeschlagen");
        }
    }
}
