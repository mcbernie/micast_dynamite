mod document;
mod vdom;
mod scripting;
mod parser;
mod render;
pub mod layout;
pub mod styles;

use std::collections::HashMap;

use log::warn;
use render::{build_layout_tree, render_layout_tree, render_node_postorder};
use vdom::diff_vnode;
use parser::load_lua_scripts;
use scripting::Engine;
pub use parser::parse_html_to_vdom;
pub use render::Renderer;

pub use vdom::DiffOp;

pub use parser::parse_color;

pub struct Dynamite<R: Renderer> {
    pub vdom: document::VDom,
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

        Ok(Self {
            vdom,
            engine,
            first_run: true,
            renderer: render_backend
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

        if self.first_run {
            self.first_run = false;
            self.engine.call_onload()?;
        } else {
            self.engine.call_onupdates()?;
        }

        let vdom = self.engine.commit().unwrap();

        let patch = diff_vnode(&old_vdom, &vdom);

        //warn!("old_vdom: {:?}", old_vdom);
        //warn!("vdom: {:?}", vdom);

        if let Some(patch) = patch {
            let mut sizes = HashMap::new();
            let size = render_node_postorder(&vdom, ctx, &mut self.renderer, &mut sizes, size);

            let layout_box = build_layout_tree(&vdom, 0.0,0.0, &sizes);
            println!("{:#?}",&layout_box);
            render_layout_tree(ctx, &layout_box, &mut self.renderer);
            self.vdom.root = vdom;
            return Ok(true);
            //self.renderer.render(&patch);
        }

        Ok(false)
    }
}