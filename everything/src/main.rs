#![allow(dead_code)]

#[macro_use]
extern crate glium;

use glium::{Program, Surface, VertexBuffer};
use glium::glutin;
use glium::index::PrimitiveType;
use glium::glutin::{WindowEvent};

use std::error::Error;

fn scale(a: &[f32; 3], scale: f32) -> [f32; 3] {
    [ a[0] * scale, a[1] * scale, a[2] * scale ]
}

fn negate(a: &[f32; 3]) -> [f32; 3] {
    scale(a, -1.0)
}

fn add(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    add(a, &negate(b))
}

fn length(a: &[f32; 3]) -> f32 {
    f32::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
}

fn normalize(a: &[f32; 3]) -> [f32; 3] {
    let inverse_length = 1.0 / length(a);
    assert!(!inverse_length.is_infinite());
    scale(a, inverse_length)
}

/// Return the point `angle` radians around the origin-centered ellipse whose
/// major axis (center to zero-radians point) is `i` and whose minor axis
/// (center to π/2) is `j`.
fn mix_by_angle(i: &[f32; 3], j: &[f32; 3], angle: f32) -> [f32; 3] {
    add(&scale(i, angle.cos()),
        &scale(j, angle.sin()))
}

/// Properties identifying an isosceles triangle spinning about its axis of
/// symmetry in 3-space, with a distinguished front face.
struct Triangle {
    /// Location of the triangle's tip (the corner that lies on the axis of
    /// rotation).
    tip: [f32; 3],

    /// The midpoint of the triangle's base (the side opposite the tip).
    base_midpt: [f32; 3],

    /// Vector from base_midpt to the corner clockwise from the tip.
    base_midpt_to_corner: [f32; 3]
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3]
}

implement_vertex!(Vertex, position);

impl Triangle {
    /// Return the positions of this triangle's three corners, with the triangle
    /// rotated about its axis by `spin` radians.
    fn corners(&self) -> [[f32; 3]; 3] {
        let corner1 = add(&self.base_midpt, &self.base_midpt_to_corner);
        let corner2 = subtract(&self.base_midpt, &self.base_midpt_to_corner);
        // Viewed from the front, our vertices must appear in clockwise order.
        [self.tip, corner1, corner2]
    }
}

fn main() -> Result<(), Box<Error>> {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(1000, 1000);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let triangle_interiors_program =
        Program::from_source(&display,
                             &include_str!("tri.vert"),
                             &include_str!("interior.frag"),
                             None)
        .expect("building program");

    let triangle = Triangle {
        tip: [ 0.5, 0.0, 0.0 ],
        base_midpt: [ 0.0, 0.0, -0.5 ],
        base_midpt_to_corner: [ 0.0, -0.5, 0.0 ]
    };

    let mut window_open = true;
    while window_open {
        let mut frame = display.draw();
        frame.clear_color(1.0, 1.0, 1.0, 1.0);

        let mut vertices = Vec::new();

        vertices.extend(triangle.corners().iter()
                        .map(|&position| Vertex { position }));
        assert_eq!(vertices.len(), 3);
        let vertex_buffer = VertexBuffer::new(&display, &vertices)?;

        frame.draw(&vertex_buffer, &glium::index::NoIndices(PrimitiveType::TrianglesList),
                   &triangle_interiors_program,
                   &uniform! {}, &Default::default())?;
        frame.finish()?;

        events_loop.poll_events(|event| {
            match event {
                // Break from the main loop when the window is closed.
                glutin::Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                    window_open = false;
                }
                _ => (),
            }
        });
    }
    Ok(())
}
