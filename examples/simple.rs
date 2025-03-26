
use std::{fs, path::Path};
use micast_dynamite::Dynamite;

pub struct Renderer;

impl micast_dynamite::Renderer for Renderer {
    fn render(&mut self, node: &micast_dynamite::DiffOp) {
        println!("-> {:#?}", node);
    }
}

fn main() -> Result<(), String> {
    env_logger::init();

    // 1. Lade `test.html` aus `assets/`
    let path = Path::new("assets/example1.html");
    let html = fs::read_to_string(path).expect("Konnte test.html nicht laden");

    let r = Renderer;

    let mut d = Dynamite::new(&html, r)?;

    d.run_frame()?;
    d.run_frame()?;
    d.run_frame()?;
    d.run_frame()?;
    d.run_frame()?;



    Ok(())
}