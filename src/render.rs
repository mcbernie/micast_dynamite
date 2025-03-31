use std::{collections::HashMap, sync::Arc};

use log::warn;
use ulid::Ulid;

use crate::{layout::LayoutMapping, styles::Style, vdom::{DiffOp, VNode}};


pub trait Renderer {
    type Context;
    fn draw_text(&mut self, ctx: &mut Self::Context, text: &str, style: &Style, x: f32, y: f32);
    fn draw_element(&mut self, ctx: &mut Self::Context, tag: &str, attrs: &Style, x: f32, y: f32, width: f32, height: f32);
    fn measure_text(&self, ctx: &Self::Context, text: &str, style: &Style) -> (u32, u32);
}

pub fn render_dom<R: Renderer>(
    l: &LayoutMapping,
    node: &VNode,
    render: &mut R,
    ctx: &mut R::Context,
    parent_offset: (f32, f32), // Akkumulierte Verschiebung vom Root
) -> bool {
    let mut is_dirty = false;
    match node {
        VNode::Text(text) => {
            if let Some(taffy_node_id) = l.id_map.get(&text.internal_id) {
                match l.taffy.layout(*taffy_node_id) {
                    Ok(layout) => {
                        // Berechne die absolute Position, indem der Elternoffset addiert wird.
                        let abs_x = parent_offset.0 + layout.location.x;
                        let abs_y = parent_offset.1 + layout.location.y;
                        //warn!("draw text at ({}, {})", abs_x, abs_y);
                        //if l.taffy.dirty(*taffy_node_id).ok() == Some(true) {
                            render.draw_text(ctx, &text.rendered, &text.style, abs_x, abs_y);
                        //    is_dirty = true;
                        //}
                    }
                    Err(e) => {
                        warn!("Failed to get layout for text node: {:?}", e);
                    }
                }
            } else {
                warn!("Text node not found in layout mapping");
            }
            is_dirty
        }
        VNode::Element(el) => {
            if let Some(taffy_node_id) = l.id_map.get(&el.internal_id) {
                match l.taffy.layout(*taffy_node_id) {
                    Ok(layout) => {
                        // Berechne auch hier die absolute Position.
                        let abs_x = parent_offset.0 + layout.location.x;
                        let abs_y = parent_offset.1 + layout.location.y;
                        // Zuerst werden die Kinder mit dem neuen Offset gerendert.
                        //if l.taffy.dirty(*taffy_node_id).ok() == Some(true) {
                            render.draw_element(
                                ctx,
                                &el.tag,
                                &el.style,
                                abs_x,
                                abs_y,
                                layout.size.width,
                                layout.size.height,
                            );
                            is_dirty = true;
                        //}
                        for child in &el.children {
                            if render_dom(l, child, render, ctx, (abs_x, abs_y)) {
                                is_dirty = true;
                            }
                        }
                        //warn!("draw element at ({}, {})", abs_x, abs_y);
                    }
                    Err(e) => {
                        warn!("Failed to get layout for element node: {:?}", e);
                    }
                }
            } else {
                warn!("Element node not found in layout mapping");
            }
            is_dirty
        }
    }
}
