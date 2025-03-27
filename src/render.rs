use std::collections::HashMap;

use log::warn;
use ulid::Ulid;

use crate::vdom::{DiffOp, VNode};

#[derive(Debug, Clone)]
pub struct LayoutBox<'a> {
    pub vnode: &'a VNode,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<LayoutBox<'a>>,
}

pub trait Renderer {
    type Context;
    fn draw_text(&mut self, ctx: &mut Self::Context, text: &str, attrs: &HashMap<String, String>, x: f32, y: f32);
    fn draw_element(&mut self, ctx: &mut Self::Context, tag: &str, attrs: &HashMap<String, String>, x: f32, y: f32, width: f32, height: f32);
    fn measure_text(&self, ctx: &mut Self::Context, text: &str, attrs: &HashMap<String, String>) -> (u32, u32);
}

fn parse_size(val: Option<&String>, relative_to: u32) -> Option<u32> {
    val.and_then(|v| {
        if v.ends_with('%') {
            let pct = v.trim_end_matches('%').parse::<f32>().ok()?;
            Some(((pct / 100.0) * (relative_to as f32)) as u32)
        } else {
            if v.ends_with("px") {
                let v = v.trim_end_matches("px");
                v.parse::<u32>().ok()
            } else {
                v.parse::<u32>().ok()
            }
        }
    })
}

fn get_box_value(attrs: &HashMap<String, String>, name: &str, fallback: u32) -> u32 {
    parse_size(attrs.get(name), 0).unwrap_or(fallback)
}

fn get_box_sides(attrs: &HashMap<String, String>, prefix: &str) -> (u32, u32, u32, u32) {
    let all = get_box_value(attrs, prefix, 0);
    let top = get_box_value(attrs, &format!("{}-top", prefix), all);
    let right = get_box_value(attrs, &format!("{}-right", prefix), all);
    let bottom = get_box_value(attrs, &format!("{}-bottom", prefix), all);
    let left = get_box_value(attrs, &format!("{}-left", prefix), all);
    (top, right, bottom, left)
}

pub fn render_node_postorder<R: Renderer>(
    node: &VNode,
    ctx: &mut R::Context,
    renderer: &mut R,
    sizes: &mut HashMap<Ulid, (u32, u32)>,
    container_size: (u32, u32),
) -> (u32, u32) {
    match node {
        VNode::Text(t) => {
            let size = renderer.measure_text(ctx, &t.rendered, &t.attrs);
            sizes.insert(t.internal_id, size);
            size
        }
        VNode::Element(el) => {
            let mut children_sizes = Vec::new();
            let mut max_width = 0;
            let mut total_height = 0;

            for child in &el.children {
                let size = render_node_postorder(child, ctx , renderer, sizes, container_size);
                children_sizes.push((child.get_internal_id(), size));
                // maximale breite der kinder
                max_width = max_width.max(size.0);
                total_height += size.1;
            }

            let style = &el.attrs;
            let mut width = max_width;
            let mut height = total_height;

            if let Some(w) = parse_size(style.get("width"), container_size.0) {
                width = w;
            }
            if let Some(h) = parse_size(style.get("height"), container_size.1) {
                height = h;
            }
            if let Some(max_w) = parse_size(style.get("max-width"), container_size.0) {
                width = width.min(max_w);
            }
            if let Some(max_h) = parse_size(style.get("max-height"), container_size.1) {
                height = height.min(max_h);
            }

            let (pad_top, pad_right, pad_bottom, pad_left) = get_box_sides(style, "padding");
            width += pad_left + pad_right; // erweiter die breite mit padding left und right
            height += pad_top + pad_bottom;

            // Begrenze Kindgrößen auf Parent-Innenbereich (effizient!)
            let inner_width = width.saturating_sub(pad_left + pad_right);
            let inner_height = height.saturating_sub(pad_top + pad_bottom);

            for (child_id, child_size) in &mut children_sizes {
                let clamped_width = child_size.0.min(inner_width);
                let clamped_height = child_size.1.min(inner_height);
                sizes.insert(**child_id, (clamped_width, clamped_height));
            }

            sizes.insert(el.internal_id, (width, height));
            (width, height)
        }
    }
}


pub fn build_layout_tree<'a>(
    node: &'a VNode,
    x: f32,
    y: f32,
    sizes: &HashMap<Ulid, (u32, u32)>,
) -> LayoutBox<'a> {
    match node {
        VNode::Text(t) => {
            let (w, h) = sizes[&t.internal_id];
            //let x = x + parent_padding.0 as f32;
            //let y = y + parent_padding.1 as f32;
            LayoutBox {
                vnode: node,
                x,
                y,
                width: w as f32,
                height: h as f32,
                children: vec![],
            }
        }
        VNode::Element(el) => {
            let (w, h) = sizes[&el.internal_id];
            let (pad_top, pad_right, pad_bottom, pad_left) = get_box_sides(&el.attrs, "padding");
            let (mar_top, mar_right, mar_bottom, _mar_left) = get_box_sides(&el.attrs, "margin");

            let align = el.attrs.get("align").map(|v| v.as_str()).unwrap_or("start");
            let direction = el.attrs.get("flex-direction").map(|v| v.as_str()).unwrap_or("column");

            let mut layout_children = vec![];

            let mut offset_main = match direction {
                "row" => x + pad_left as f32,
                _ => y + pad_top as f32,
            };

            for child in &el.children {
                let child_id = child.get_internal_id();
                let (cw, ch) = sizes[&child_id];

                let (child_x, child_y) = match direction {
                    "row" => {
                        let y_align = match align {
                            "center" => y + ((h - ch) as f32) / 2.0,
                            "end" => y + (h - ch) as f32,
                            _ => y,
                        };
                        let current = (offset_main, y_align);
                        offset_main += cw as f32;
                        current
                    }
                    _ => {
                        let x_align = match align {
                            "center" => x + ((w - cw) as f32) / 2.0,
                            "end" => x + (w - cw) as f32,
                            _ => x,
                        };
                        let current = (x_align, offset_main);

                        offset_main += ch as f32 + mar_bottom as f32;
                        if mar_bottom > 0 {
                            warn!("Margin bottom added to offset_main {}", mar_bottom);
                        }
                        current
                    }
                };

                // Child elemente an den richtigen Positionen layouten
                // child_x sollte bei margin top um X verschoben werden
                let child_box = build_layout_tree(
                    child, 
                    child_x + pad_left as f32, 
                    child_y + pad_top as f32 + 20.0, 
                    sizes, 
                );
                layout_children.push(child_box);
            }

            LayoutBox {
                vnode: node,
                x,
                y,
                width: w as f32,
                height: h as f32,
                children: layout_children,
            }
        }
    }
}

pub fn render_layout_tree<'a, R: Renderer>(ctx: &mut R::Context, layout: &LayoutBox<'a>, renderer: &mut R) {
    match layout.vnode {
        VNode::Text(t) => {
            renderer.draw_text(ctx, &t.rendered, &t.attrs, layout.x, layout.y);
        }
        VNode::Element(el) => {
            renderer.draw_element(
                ctx,
                &el.tag,
                &el.attrs,
                layout.x,
                layout.y,
                layout.width,
                layout.height,
            );
            for child in &layout.children {
                render_layout_tree(ctx, child, renderer);
            }
        }
    }
}
