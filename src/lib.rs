mod document;
mod vdom;
mod new_vdom;
mod scripting;
mod parser;
mod render;

use log::warn;
use new_vdom::diff_vnode;
use parser::load_lua_scripts;
use scripting::Engine;
pub use vdom::{VDom, VNode};
pub use parser::parse_html_to_vdom;
pub use render::Renderer;

pub use new_vdom::DiffOp;

pub struct Dynamite<R: Renderer> {
    vdom: document::VDom,
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
    pub fn run_frame(&mut self) -> Result<(), String> {
        let old_vdom = self.vdom.root.clone();
        self.engine.begin(&self.vdom).unwrap();

        if self.first_run {
            self.first_run = false;
            self.engine.call_onload()?;
        } else {
            self.engine.call_onupdates()?;
        }

        let new_vdom = self.engine.commit().unwrap();

        let patch = diff_vnode(&old_vdom, &new_vdom);

        warn!("old_vdom: {:?}", old_vdom);
        warn!("new_vdom: {:?}", new_vdom);

        if let Some(patch) = patch {
            self.renderer.render(&patch);
        }
        Ok(())
    }
}