extern crate euclid;
extern crate image;
extern crate glium;

extern crate vitral;
extern crate vitral_glium;

use std::path::Path;
use image::GenericImage;
use glium::{DisplayBuild, glutin};
use euclid::{Rect, Point2D, Size2D};
use vitral::{Context, PropPoint2D, Align};
use vitral_glium::{Backend, DefaultVertex};

fn load_image<V>(display: &glium::Display,
                 backend: &mut Backend<V>,
                 path: &str)
                 -> vitral::ImageData<usize>
    where V: vitral::Vertex + glium::Vertex
{
    let image = image::open(&Path::new(path)).unwrap();
    let (w, h) = image.dimensions();
    let pixels = image.pixels()
                      .map(|(_, _, p)| unsafe { ::std::mem::transmute::<image::Rgba<u8>, u32>(p) })
                      .collect();
    let image = vitral::ImageBuffer {
        size: Size2D::new(w, h),
        pixels: pixels,
    };

    let id = backend.make_texture(display, image);

    vitral::ImageData {
        texture: id,
        size: Size2D::new(w, h),
        tex_coords: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(1.0, 1.0)),
    }
}

struct ContextBase {
    state: vitral::State<usize, DefaultVertex>,
}

impl vitral::Context for ContextBase {
    type T = usize;
    type V = DefaultVertex;

    fn state<'a>(&'a self) -> &'a vitral::State<usize, DefaultVertex> {
        &self.state
    }

    fn state_mut<'a>(&'a mut self) -> &'a mut vitral::State<usize, DefaultVertex> {
        &mut self.state
    }

    fn new_vertex(&mut self,
                  pos: Point2D<f32>,
                  tex_coord: Point2D<f32>,
                  color: [f32; 4])
                  -> DefaultVertex {
        DefaultVertex {
            pos: [pos.x, pos.y],
            tex_coord: [tex_coord.x, tex_coord.y],
            color: color,
        }
    }
}

fn main() {
    // Construct Glium backend.
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let size = Size2D::new(640.0, 360.0);

    let mut backend = Backend::new(&display,
                                   vitral_glium::default_program(&display).unwrap(),
                                   size.width as u32,
                                   size.height as u32);

    // Construct Vitral context.
    let state: vitral::State<usize, DefaultVertex>;
    let builder = vitral::Builder::new();
    let image = load_image(&display, &mut backend, "julia.png");
    state = builder.build(size, |img| backend.make_texture(&display, img));

    let mut context = ContextBase { state: state };

    let mut test_input = String::new();

    // Run the program.
    loop {
        context.begin_frame();

        context.draw_image(&image, Point2D::new(0.0, 0.0), [1.0, 1.0, 1.0, 1.0]);

        context.draw_line(3.0,
                          [1.0, 0.0, 0.0, 1.0],
                          PropPoint2D::new(0.1, 0.1),
                          PropPoint2D::new(0.9, 0.9));

        let mut text_pos = PropPoint2D::new(0.5, 0.0);
        text_pos = context.draw_text(text_pos, Align::Center, [0.0, 1.0, 0.0, 1.0], "Hello,");
        context.draw_text(text_pos, Align::Center, [0.0, 1.0, 0.0, 1.0], "world!");

        {
            let mut c = context.bound_clipped(Rect::new(Point2D::new(100.0, 100.0),
                                                        Size2D::new(320.0, 240.0)));

            c.draw_image(&image, Point2D::new(0.0, 0.0), [1.0, 1.0, 1.0, 1.0]);

            // Demonstrate proportional coordinates
            c.draw_line(5.0,
                        [1.0, 1.0, 1.0, 1.0],
                        PropPoint2D::new(0.1, 0.1),
                        PropPoint2D::new(0.9, 0.9));

            c.fill_rect(Rect::new(Point2D::new(0.0, 0.0), Size2D::new(80.0, 16.0)),
                        [0.0, 0.0, 0.0, 1.0]);
            // Text in bounds
            c.draw_text(Point2D::new(0.0, 0.0),
                        Align::Left,
                        [1.0, 1.0, 0.0, 1.0],
                        "Window");
        }
        if context.bound(Rect::new(Point2D::new(10.0, 30.0), Size2D::new(120.0, 20.0)))
                  .button("Hello, world")
                  .left_clicked() {
            println!("Click");
        }

        if context.bound(Rect::new(Point2D::new(10.0, 60.0), Size2D::new(120.0, 20.0)))
                  .button("Another button")
                  .left_clicked() {
            println!("Clack {}", test_input);
        }

        context.text_input([0.8, 0.8, 0.8, 1.0], &mut test_input);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
