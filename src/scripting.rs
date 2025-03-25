use std::{cell::RefCell, rc::Rc};

use log::{info, warn};
use mlua::{AnyUserData, Lua, Result, UserData, UserDataMethods, Value};
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use serde_json::Value as JsonValue;
use ulid::Ulid;

use crate::vdom::{VDom, VNode};

#[derive(Clone)]
pub struct VDomContext(pub Rc<RefCell<VDom>>);

impl mlua::UserData for VDomContext {}


#[derive(Clone)]
pub struct ElementContext(pub Ulid);

impl mlua::UserData for ElementContext {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // Beispiel: set_text-Methode, die anhand der ID das Element im VDOM updatet.
        methods.add_method("set_text", |lua, this, (key, value): (String, String)| {
            // Hole das globale VDOM – hier gehen wir davon aus, dass du es z. B. im Lua-Registry abgelegt hast.
            let globals = lua.globals();
            let vdom_ud: mlua::AnyUserData = globals.get("_vdom")?;
            let vdom_context = vdom_ud.borrow::<VDomContext>()?;
            
            // Jetzt kannst du auf das VDOM zugreifen:
            let vdom = vdom_context.0.clone();
            info!("set_text: {}", key);
            
            // Suche das Element im VDOM anhand der ULID.
            if let Some(element) = vdom.borrow_mut().get_element_by_internal_id(&this.0) {
                info!("some element!");
                // Setze den Text oder ein Attribut; hier als Beispiel:
                if let VNode::Element { internal_id, id, attributes, is_dirty, .. } = &mut *element.borrow_mut() {
                    warn!("Setting text for element with ID: {} internal:{}", id.as_deref().unwrap_or("None"), internal_id);
                    attributes.insert(key, value);
                    *is_dirty = true;
                }
            }
            Ok(())
        });
    }
}

//impl UserData for VNode {
//    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
//        methods.add_method_mut("set_style", |_, this, (key, value): (String, String)| {
//            if let VNode::Element { styles, is_dirty, .. } = this {
//                styles.insert(key, value);
//                *is_dirty = true; // Rendering notwendig
//            }
//            Ok(())
//        });
//    }
//}

fn get_webdata(lua: &Lua, url: String) -> Result<String> {
    let response = get(&url).and_then(|resp| resp.text()).unwrap_or_else(|_| "{}".to_string());
    Ok(response)
}

/// Erzeugt ein Element aus einer Vorlage und gibt ein leichtes Handle (ElementContext) zurück.
/// Dabei wird eine ULID generiert, die als ID im VDOM verwendet wird.
fn create_element(lua: &Lua, vdom: Rc<RefCell<VDom>>, template_id: String) -> Result<mlua::Value> {
    let e = {
        let vdom_ref = vdom.borrow();
        let a = vdom_ref.create_element_from_template(&template_id);
        drop(vdom_ref);
        a
    };
    if let Some(element) = e {


        let id = {
            element.borrow().get_internal_id().clone()
        };

        {
            let mut vdom_mut = vdom.borrow_mut();
            let root = vdom_mut.root.clone();
            vdom_mut.insert_element(root, element);
        }
        // Registriere das Element intern im VDOM (z. B. in einer HashMap)
        //vdom_mut.register_element(id.clone(), Rc::clone(&element));
        
        // Gib ein leichtes Handle (ElementContext) an Lua zurück.
        let handle = lua.create_userdata(ElementContext(id))?;
        Ok(mlua::Value::UserData(handle))
    } else {
        Ok(mlua::Value::Nil)
    }
}

pub fn register_lua_api(lua: &Lua, vdom: Rc<RefCell<VDom>>) -> Result<()> {
    let globals = lua.globals();

    globals.set("_vdom", lua.create_userdata(VDomContext(vdom.clone()))?)?;



    let cloned_vdom = vdom.clone();
    globals.set("create_element", lua.create_function(move |lua, id: String| {
        create_element(lua, cloned_vdom.clone(), id)
    })?)?;
    //globals.set("set_text", lua.create_function(set_text)?)?;
    globals.set("get_webdata", lua.create_function(get_webdata)?)?;
    globals.set("parse_json", lua.create_function(parse_json)?)?;

    let cloned_vdom = vdom.clone();
    globals.set("get_element_by_id", lua.create_function_mut(move |lua, id: String| {
        let vdom = cloned_vdom.clone();
        let e = {
            let vdom = vdom.borrow();
            vdom.get_element_by_id(&id)
        };

        if let Some(element) = e {

            let id = element.borrow().get_internal_id().clone();

            let handle = lua.create_userdata(ElementContext(id))?;
            Ok(mlua::Value::UserData(handle))
        } else {
            Ok(mlua::Value::Nil)
        }
        
    })?)?;

    let cloned_vdom = vdom.clone();
    // 📌 add_element(target_id, child_node_userdata)
    let add_element_func = lua.create_function(move |_, (target_id, node_ud): (String, AnyUserData)| {
        let cloned_vdom = cloned_vdom.clone();
        // Hole zuerst den VNodeHandle und klone das Rc, um das Element zu extrahieren.
        let node = node_ud.borrow::<ElementContext>()?;

        let element_rc = {
            let vdom = cloned_vdom.borrow();

            info!("get element for add_element: {}", node.0);
            vdom.get_element_by_internal_id(&node.0)
                .ok_or(mlua::Error::external("Element not found"))?
                .clone()
        };

        {
            // Nun sollte kein anderer Borrow mehr aktiv sein.
            let mut vdom = cloned_vdom.borrow_mut();
            vdom.add_element(&target_id, element_rc)
                .map_err(|e| mlua::Error::external(format!("add_element failed: {}", e)))?;
        }
        Ok(())
    })?;
    
    globals.set("add_element", add_element_func)?;

    Ok(())
}

pub fn load_lua_scripts(lua: &Lua, html: &str) -> Result<()> {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").unwrap();

    for script in document.select(&script_selector) {
        if let Some(script_content) = script.text().next() {
            println!("script_content: {}", script_content);
            lua.load(script_content).exec()?;
        }
    }
    Ok(())
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

pub fn trigger_onload(lua: &Lua, vdom: &Rc<RefCell<VDom>>) -> Result<()> {
    let root = {
        vdom.borrow().root.clone()
    };

    let attributes = {
        let element = root.borrow();
        if let VNode::Element { attributes, .. } = &*element {
            Some(attributes.clone())
        } else {
            None
        }
    };

    if let Some(attributes) = attributes {
        if let Some(onload) = attributes.get("onload") {
            lua.load(format!("{}()", onload)).exec()?; // ✅ Lua-Funktion ausführen
        }
    }
    Ok(())
}
