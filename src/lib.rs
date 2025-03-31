mod document;
mod vdom;
mod scripting;
mod parser;
mod render;
pub mod layout;
pub mod styles;

use std::collections::HashMap;

use log::warn;
use render::render_dom;
use taffy::{NodeId, Style};
use vdom::diff_vnode;
use parser::load_lua_scripts;
use scripting::Engine;
pub use parser::parse_html_to_vdom;
pub use render::Renderer;

pub use vdom::DiffOp;

pub use parser::parse_color;

pub struct Dynamite<R: Renderer> {
    pub vdom: document::VDom,
    pub layout: layout::LayoutMapping,
    engine: Engine,
    first_run: bool,
    renderer: R
}

impl<R: Renderer> Dynamite<R> {

    pub fn new(html: &str, render_backend: R) -> Result<Self, String> {
        let vdom = parse_html_to_vdom(html)?;
        let mut engine = Engine::new();
        engine.search_onupdate_functions(&vdom)?;
        let scripts = load_lua_scripts(html)?;
        engine.load_scripts(scripts).map_err(|e| e.to_string())?;

        let mut layout = layout::LayoutMapping::new();
        let _ = layout.build_tree(&vdom.root, None);

        Ok(Self {
            vdom,
            engine,
            first_run: true,
            renderer: render_backend,
            layout
        })
    }

    /// called when a frame should be prepared for rendering
    /// 
    /// This function will call all `onupdate` functions in the Lua scripts
    /// and then calculate the difference between the old and the new vdom
    /// and apply the changes to the real DOM.
    pub fn run_frame(&mut self, ctx: &mut R::Context, size: (u32, u32)) -> Result<bool, String> {
        let old_vdom = self.vdom.root.clone();
        self.engine.begin(&self.vdom).unwrap();

        let mut first_draw = true;
        if self.first_run {
            self.first_run = false;
            self.engine.call_onload()?;
        } else {
            first_draw = false;
            self.engine.call_onupdates()?;
        }

        let vdom = self.engine.commit().unwrap();

        let patch = diff_vnode(&old_vdom, &vdom);

        //warn!("old_vdom: {:?}", old_vdom);

        if let Some(patch) = patch { //if patch.is_some() || first_draw {

            //warn!("vdom: {:#?}", vdom);

            // take the old VDOM, apply the patch and generate a new layout
            self.layout.apply_diff(&old_vdom, &patch);
            //self.layout.taffy.clear();
            //self.layout.id_map.clear();
            // create first node!
            let node = self.layout.build_tree(&vdom, None);

            self.vdom.root = vdom;

            let dirty = self.layout.taffy.dirty(node).unwrap_or(false);

            // render the new layout
            // TODO: Implement the rendering
            if dirty {
                // compute the layout for the new layout
                self.layout.compute_layout(
                    &node,
                    size.0 as f32,
                    size.1 as f32,
                    &self.renderer, 
                    ctx
                );

                self.layout.taffy.print_tree(node);
                let _ = render_dom(&self.layout, &self.vdom.root, &mut self.renderer, ctx, (0.0, 0.0));
                warn!("something is dirty");
            }
            
            return Ok(dirty)
        }

        Ok(false)
    }
}