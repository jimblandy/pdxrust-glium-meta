#[macro_use]
extern crate glium;

use glium::{Program, Surface, VertexBuffer};
use glium::glutin;
use glium::index::PrimitiveType;
use glium::glutin::{WindowEvent};

use std::error::Error;
use std::time::{Instant};

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

/// Return the point `angle` radians around the origin-centered ellipse whose
/// major axis (center to zero-radians point) is `i` and whose minor axis
/// (center to π/2) is `j`.
fn mix_by_angle(i: &[f32; 3], j: &[f32; 3], angle: f32) -> [f32; 3] {
    add(&scale(i, angle.cos()),
        &scale(j, angle.sin()))
}

/// Properties identifying an isosceles triangle spinning about its axis of
/// symmetry in 3-space.
struct Triangle {
    /// Location of the triangle's tip (the corner that lies on the axis of
    /// rotation).
    tip: [f32; 3],

    /// Vector from the trangle's tip to the midpoint of its base (the side
    /// opposite the tip).
    tip_to_base_midpt: [f32; 3],

    /// Vector from the midpoint of the base to one of the base corners,
    /// in the unrotated state.
    base_midpt_to_corner: [f32; 3],

    /// Vector normal to `base_midpt` and `corner`, of the same length as
    /// `corner`. (This could just be derived from base_midpt and corner.)
    normal: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3]
}

implement_vertex!(Vertex, position);

impl Triangle {
    /// Return the positions of this triangle's three corners, with the triangle
    /// rotated about its axis by `spin` radians.
    fn corners(&self, spin: f32) -> [[f32; 3]; 3] {
        let base_midpt = add(&self.tip, &self.tip_to_base_midpt);
        let base_midpt_to_corner = mix_by_angle(&self.base_midpt_to_corner,
                                                &self.normal,
                                                spin);
        let corner1 = add(&base_midpt, &base_midpt_to_corner);
        let corner2 = subtract(&base_midpt, &base_midpt_to_corner);
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

    let triangle_interiors =
        Program::from_source(&display,
                             &include_str!("tri.vert"),
                             &include_str!("interior.frag"),
                             None)
        .expect("building program");

    let triangle = Triangle {
        tip: [ 0.5, 0.0, 0.0 ],
        tip_to_base_midpt: [ -0.5, 0.0, 0.5 ],
        base_midpt_to_corner: [ 0.0, 0.707, 0.0 ],
        normal: [ -0.5, 0.0, -0.5 ]
    };

    let start_time = Instant::now();

    loop {
        let frame_time = Instant::now() - start_time;

        let seconds = frame_time.as_secs() as f32 +
            (frame_time.subsec_nanos() as f32 * 1e-9);
        let spin = seconds * 0.25 * 2.0 * std::f32::consts::PI;

        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 1.0, 1.0);

        let positions = triangle.corners(spin);
        let vertices: Vec<_> = positions.iter()
            .map(|&position| Vertex { position })
            .collect();
        assert_eq!(vertices.len(), 3);
        let vertex_buffer = VertexBuffer::new(&display, &vertices)?;

        frame.draw(&vertex_buffer, &glium::index::NoIndices(PrimitiveType::TrianglesList),
                   &triangle_interiors,
                   &uniform! {}, &Default::default())?;
        frame.finish()?;

        let mut should_exit = false;

        events_loop.poll_events(|event| {
            match event {
                // Break from the main loop when the window is closed.
                glutin::Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                    should_exit = true;
                }
                _ => (),
            }
        });

        if should_exit {
            return Ok(());
        }
    }
}
