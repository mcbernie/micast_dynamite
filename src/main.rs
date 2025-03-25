use std::{cell::RefCell, fs, path::Path, rc::Rc};
use parser::parse_html_to_vdom;
use mlua::{Lua, Result};
use scripting::{load_lua_scripts, register_lua_api, trigger_onload};

mod vdom;
mod scripting;
mod parser;

fn main() -> Result<()> {
    env_logger::init();

    // 1. Lade `test.html` aus `assets/`
    let path = Path::new("assets/example1.html");
    let html = fs::read_to_string(path).expect("Konnte test.html nicht laden");

    // 2. vDOM aus HTML erzeugen
    let vdom = parse_html_to_vdom(&html).unwrap();
    let vdom = Rc::new(RefCell::new(vdom));


    // 3. Lua-Interpreter initialisieren
    let lua = Lua::new();

    // 4. Lua-API registrieren
    register_lua_api(&lua, Rc::clone(&vdom))?;

    // 5. Lade Lua-Skripte aus `<script>`-Tags
    load_lua_scripts(&lua, &html)?;

    trigger_onload(&lua, &vdom)?;
    // 7. Zeige den geänderten vDOM
    println!("{:#?}", vdom.borrow().root);

    Ok(())
}
