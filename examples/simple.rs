
use std::{collections::HashMap, fs, path::Path};
use log::info;
use micast_dynamite::Dynamite;
use ulid::Ulid;

pub struct Renderer;

impl micast_dynamite::Renderer for Renderer {
    type Context = ();
    fn draw_text(&mut self, ctx: &mut Self::Context, text: &str, attrs: &HashMap<String, String>, x: f32, y: f32) {

        info!("draw_text: {} at ({}, {})\n", text, x, y);
    }
    fn draw_element(&mut self, ctx: &mut Self::Context, tag: &str, attrs: &HashMap<String, String>, x: f32, y: f32, width: f32, height: f32) {
        info!("draw_element: {} at ({}, {}) with size {}x{}\n", tag, x, y, width, height);

    }

    fn measure_text(&self, ctx: &mut Self::Context, text: &str, attrs: &HashMap<String, String>) -> (u32, u32) {
        ((text.len() * 4) as u32, 16)
    }
}

fn main() -> Result<(), String> {
    env_logger::init();

    // 1. Lade `test.html` aus `assets/`
    let path = Path::new("assets/example1.html");
    let html = fs::read_to_string(path).expect("Konnte test.html nicht laden");

    let r = Renderer;

    let mut d = Dynamite::new(&html, r)?;

    let mut ctx = (); 

    d.run_frame(&mut ctx, (800,600))?;
    //d.run_frame(&mut ctx, (800,600))?;
    //d.run_frame(&mut ctx, (800,600))?;
    //d.run_frame(&mut ctx, (800,600))?;
    //d.run_frame(&mut ctx, (800,600))?;


    println!("final, vdom root: {:#?}", d.vdom.root);

    Ok(())
}