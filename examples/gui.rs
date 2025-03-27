use std::rc::Rc;

use femtovg::renderer::OpenGl;
use femtovg::{Canvas, Color, FontId, Paint, Path};
use gl_render::glutin::window::Window;
use gl_render::glutin::{ContextWrapper, PossiblyCurrent};
use gl_render::{glutin, load_gl};
use gl_render::glutin::event::{Event, WindowEvent};
use gl_render::glutin::event_loop::ControlFlow;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use micast_dynamite::{parse_color, Renderer};

pub struct VGRenderer {
    pub default_font: FontId,
}

impl VGRenderer {
    pub fn new(canvas: &mut Canvas<OpenGl>) -> Self {
        let default_font = canvas.add_font("assets/gidole-regular.ttf").expect("Cannot add font");


        Self { 
            default_font 
        }
    }
}

impl Renderer for VGRenderer {
    type Context = Canvas<OpenGl>; 
    fn draw_text(&mut self, ctx: &mut Self::Context, text: &str, attrs: &std::collections::HashMap<String, String>, x: f32, y: f32) {
        let font_size = attrs.get("font-size").and_then(|s| s.parse::<f32>().ok()).unwrap_or(16.0);
        ctx.fill_text(x, y, text, 
            &Paint::color(Color::white())
                    .with_font(&[self.default_font])
                    .with_font_size(font_size)
                    .with_text_baseline(femtovg::Baseline::Top)
            ).unwrap();
    }

    fn draw_element(&mut self, ctx: &mut Self::Context, tag: &str, attrs: &std::collections::HashMap<String, String>, x: f32, y: f32, width: f32, height: f32) {
        let color_from_map = parse_color(attrs.get("background-color").unwrap_or(&String::from("#00000000"))).unwrap();
        let mut p = Path::new();
        p.rect(x, y, width, height);
        p.close();
        ctx.fill_path(
            &p,
            &Paint::color(Color::rgba(color_from_map[0], color_from_map[1], color_from_map[2], color_from_map[3]))
        );
    }

    fn measure_text(&self, ctx: &mut Self::Context, text: &str, attrs: &std::collections::HashMap<String, String>) -> (u32, u32) {
        let font_size = attrs.get("font-size").and_then(|s| s.parse::<f32>().ok()).unwrap_or(16.0);
        let measurements = ctx.measure_text(0.0, 0.0, text, 
            &Paint::color(Color::white())
                    .with_font(&[self.default_font])
                    .with_font_size(font_size)
            ).unwrap_or_default();
        
        (measurements.width() as u32, (measurements.height() + 10.0) as u32)
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("micast_dynamite - GUI Example")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::PhysicalSize::new(1920.0, 1080.0));


    let windowed_context = glutin::ContextBuilder::new()
        .with_multisampling(2)
        .with_vsync(true)
        .with_stencil_buffer(2)
        .with_double_buffer(Some(true))
        .build_windowed(window, &event_loop)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let renderer = unsafe { OpenGl::new_from_function(|s| windowed_context.get_proc_address(s).cast()) }
        .expect("Cannot create renderer");

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

    let mut render = VGRenderer::new(&mut canvas);
    let mut example_renderer = micast_dynamite::Dynamite::new(include_str!("../assets/example1.html"), render).unwrap();
    
    canvas.set_size(1920, 1080, 1.0);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => 
            {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        println!("exit close request");
                    },
                    WindowEvent::ModifiersChanged(_mods) => {
                        //example_renderer.events(glutin::event::WindowEvent::ModifiersChanged(mods));
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                state,
                                virtual_keycode: key,
                                scancode,
                                modifiers,
                            },
                        device_id ,
                        ..
                    } => {
                        
                        println!("triggered events");
                        

                        if glutin::event::ElementState::Pressed == state {
                            if key == Some(glutin::event::VirtualKeyCode::Escape) {
                                *control_flow = ControlFlow::Exit;
                                println!("esc exiting");
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        windowed_context.resize(physical_size);
                        //canvas.set_size(physical_size.width, physical_size.height, 2.0);
                    }
                    _ => (),
                }

            },
            //Event::MainEventsCleared => {
            //    // RENDER HERE
            //    std::thread::yield_now();
            //}
            Event::RedrawRequested(_) => {
                // Übergib den Frame an deinen dynamischen Renderer
                let refresh = example_renderer.run_frame(&mut canvas, (1920, 1080)).unwrap_or_default(); // Beispielname

                if refresh {
                    canvas.flush();
                }
                //windowed_context.swap_buffers().unwrap();
                windowed_context.swap_buffers().unwrap();
            }
            _ => *control_flow = ControlFlow::Poll,
        };

    });
}