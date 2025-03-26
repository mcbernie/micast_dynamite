use std::{cell::RefCell, collections::HashMap, rc::Rc};
use log::warn;
use mlua::{AnyUserData, Lua, Result, UserDataMethods, Value};
use regex::Regex;
use reqwest::blocking::get;
use serde_json::Value as JsonValue;
use ulid::Ulid;

use crate::{document::{self, FindByIdMut}, vdom::{self, ElementNode, TextNode, VNode}, render};

#[derive(Clone)]
pub struct ElementContext {
    pub internal_id: Ulid,
    pub temp_node: Rc<RefCell<Option<VNode>>>,
    pub values: Rc<RefCell<HashMap<String, String>>>,
}

impl mlua::UserData for ElementContext {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("set_text", |lua, this, (key, value): (String, String)| {
            let globals = lua.globals();
            this.values.borrow_mut().insert(key.clone(), value.clone());

            let has_node = {
                let temp_node = this.temp_node.borrow();
                temp_node.is_some()
            };

            if has_node {
                let mut temp_node = this.temp_node.borrow_mut();
                let mut taked_node = temp_node.take().unwrap();
                render_texts_in_subtree(&mut taked_node, &this.values.borrow());
                *temp_node = Some(taked_node);
            } else {
                let vdom_ud: mlua::AnyUserData = globals.get("_vdom")?;
                let vdom_context = vdom_ud.borrow::<DynamiteContext>()?;
                let vdom = vdom_context.0.clone();
                let mut borrowed_vdom = vdom.borrow_mut();

                if let Some(node) = borrowed_vdom.root.find_by_internal_id_mut(&this.internal_id) {
                    render_texts_in_subtree(node, &this.values.borrow());
                }
            }

            Ok(())
        });

    }
}

fn render_texts_in_subtree(
    node: &mut vdom::VNode,
    ctx: &HashMap<String, String>,
) {
    let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

    match node {
        VNode::Text(TextNode { template, rendered, .. }) => {
            let replaced = re.replace_all(template, |caps: &regex::Captures| {
                let key = &caps[1];
                ctx.get(key).cloned().unwrap_or_default()
            });
            *rendered = replaced.into_owned();
        }
        VNode::Element(ElementNode { children, .. }) => {
            for child in children.iter_mut() {
                render_texts_in_subtree(child, ctx);
            }
        }
    }
}

fn get_webdata(_lua: &Lua, url: String) -> Result<String> {
    let response = get(&url).and_then(|resp| resp.text()).unwrap_or_else(|_| "{}".to_string());
    Ok(response)
}

/// Erzeugt ein Element aus einer Vorlage und gibt ein leichtes Handle (ElementContext) zurück.
/// Dabei wird eine ULID generiert, die als ID im VDOM verwendet wird.
fn create_element(lua: &Lua, vdom: Rc<RefCell<document::VDom>>, template_id: String) -> Result<mlua::Value> {
    log::info!("create_element: {}", template_id);
    let e = {
        let vdom_ref = vdom.borrow();
        let a = vdom_ref.create_element_from_template(&template_id);
        drop(vdom_ref);
        a
    };
    if let Some(element) = e {
        // Gib ein leichtes Handle (ElementContext) an Lua zurück.
        let handle = lua.create_userdata(ElementContext{
            internal_id: Ulid::new(),
            temp_node: Rc::new(RefCell::new(Some(element))),
            values: Rc::new(RefCell::new(HashMap::new())),
        })?;

        Ok(mlua::Value::UserData(handle))
    } else {
        Ok(mlua::Value::Nil)
    }
}

fn json_to_lua(lua: &Lua, json: &JsonValue) -> Result<Value> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => Ok(Value::Number(n.as_f64().unwrap_or(0.0))),
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn parse_json(lua: &Lua, json_str: String) -> Result<Value> {
    let parsed: JsonValue = serde_json::from_str(&json_str).unwrap_or(JsonValue::Null);
    json_to_lua(lua, &parsed)
}




struct DynamiteContext(Rc<RefCell<document::VDom>>);
impl mlua::UserData for DynamiteContext {}

pub struct Engine {
    pub lua: Lua,
    pub onupdate_fns: Vec<mlua::Function>,
    pub onload_fns: Vec<mlua::Function>,
}

impl Engine {

    pub fn new() -> Self {
        let lua = Lua::new();
        Self::load_lua_api(&lua).unwrap();
        Self {
            lua,
            onupdate_fns: Vec::new(),
            onload_fns: Vec::new(),
        }
    }

    pub fn load_scripts(&mut self, scripts: Vec<String>) -> Result<()> {
        for script in scripts {
            self.lua.load(script).exec()?;
        }
        Ok(())
    }

    fn load_lua_api(lua: &mlua::Lua) -> Result<()> {
        let globals = lua.globals();
        globals.set("create_element", lua.create_function(move |lua, id: String| {
            let globals = lua.globals();
            let vdom_ud: mlua::AnyUserData = globals.get("_vdom")?;
            let vdom_context = vdom_ud.borrow::<DynamiteContext>()?;
            let vdom = vdom_context.0.clone();
            create_element(lua, vdom, id)
        })?)?;
        //globals.set("set_text", lua.create_function(set_text)?)?;
        globals.set("get_webdata", lua.create_function(get_webdata)?)?;
        globals.set("parse_json", lua.create_function(parse_json)?)?;

        globals.set("get_element_by_id", lua.create_function_mut(move |lua, id: String| {
            let globals = lua.globals();
            let vdom_ud: mlua::AnyUserData = globals.get("_vdom")?;
            let vdom_context = vdom_ud.borrow::<DynamiteContext>()?;
            let vdom = vdom_context.0.clone();
            let borrow_vdom = vdom.borrow();
            let e = borrow_vdom.find_element_by_id(&id);

            if let Some(element) = e {
                let id = element.get_internal_id().clone();
                let handle = lua.create_userdata(ElementContext{
                    internal_id: id,
                    temp_node: Rc::new(RefCell::new(None)),
                    values: Rc::new(RefCell::new(HashMap::new())),
                })?;
                Ok(mlua::Value::UserData(handle))
            } else {
                Ok(mlua::Value::Nil)
            }
            
        })?)?;

        let add_element_func = lua.create_function(move |lua, (target_id, node_ud): (String, AnyUserData)| {
            let globals = lua.globals();
            let vdom_ud: mlua::AnyUserData = globals.get("_vdom")?;
            let vdom_context = vdom_ud.borrow::<DynamiteContext>()?;
            let vdom = vdom_context.0.clone();
            // Hole zuerst den VNodeHandle und klone das Rc, um das Element zu extrahieren.
            let node = node_ud.borrow_mut::<ElementContext>()?;

            if let Some(temp_node) = node.temp_node.take() {
                let mut vdom = vdom.borrow_mut();
                vdom.add_element(&target_id, temp_node)
                    .map_err(|e| mlua::Error::external(format!("add_element failed: {}", e)))?;
            } else {
                warn!("temp_node is None");
            }
            Ok(())
        })?;
        
        globals.set("add_element", add_element_func)?;
        Ok(())
    }

    pub fn begin(&self, vdom: &document::VDom) -> Result<()> {
        let l = vdom.clone();
        let globals = self.lua.globals();
        globals.set("_vdom", 
           self.lua.create_userdata(
                DynamiteContext(Rc::new(RefCell::new(l)))
            )?
        )?;

        Ok(())
    }

    pub fn commit(&self) -> Result<vdom::VNode> {
        let globals = self.lua.globals();
        let dyn_userdata: mlua::AnyUserData = globals.get("_vdom")?;
        let tmp_ctx = dyn_userdata.borrow::<DynamiteContext>()?;
        let tmp = tmp_ctx.0.borrow().clone();

        // und jetzt...
        Ok(tmp.root)
    }

    pub fn search_onupdate_functions(&mut self, vdom: &document::VDom) -> std::result::Result<(), String> {
        let mut onupdate_fns = Vec::new();
        let mut onload_fns  = Vec::new();
        let root = &vdom.root;

        let mut search_in_node = |node: &vdom::VNode| {
            if let vdom::VNode::Element( ElementNode { attrs, .. }) = node {
                if let Some(onupdate) = attrs.get("onupdate") {
                    let onupdate_fn = self.lua.load(format!("{}()", onupdate)).into_function().unwrap();
                    onupdate_fns.push(onupdate_fn);
                }
                if let Some(onload) = attrs.get("onload") {
                    warn!("found onload: {}", onload);
                    let onload_fn = self.lua.load(format!("{}()", onload)).into_function().unwrap();
                    onload_fns.push(onload_fn);
                }
            }
        };

        search_in_node(root);

        self.onupdate_fns = onupdate_fns;
        self.onload_fns = onload_fns;
        Ok(())
    }

    pub fn call_onupdates(&self) -> std::result::Result<(), String> {
        for onupdate_fn in &self.onupdate_fns {
            warn!("onupdate_fn: {:?}", onupdate_fn);
            onupdate_fn.call::<()>(()).map_err(|e| format!("onupdate failed: {}", e))?;
        }
        Ok(())
    }

    pub fn call_onload(&self) -> std::result::Result<(), String> {
        for onload_fn in &self.onload_fns {
            warn!("onload_fn: {:?}", onload_fn);
            onload_fn.call::<()>(()).map_err(|e| format!("onload failed: {}", e))?;
        }
        Ok(())
    }


}