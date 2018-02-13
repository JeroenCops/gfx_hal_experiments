extern crate env_logger;
extern crate gfx_hal as hal;
extern crate gfx_backend_vulkan as back;

extern crate winit;
extern crate image;

use hal::{buffer as b, command as c, device as d, format as f, image as i, memory as m, pso, pool};
use hal::{Device, Instance, PhysicalDevice, Surface, Swapchain};

// A data structure that represents a point in 3D space.
struct  Vertex {
    a_Pos: [f32; 2],
}

const QUAD: [Vertex; 6] = [
    // First triangle
    Vertex { a_Pos: [ -0.5, 0.33 ] },
    Vertex { a_Pos: [ -0.5, 0.33 ] },
    Vertex { a_Pos: [ -0.5, 0.33 ] },
    // Second triangle
    Vertex { a_Pos: [ -0.5, 0.33 ] },
    Vertex { a_Pos: [ -0.5, 0.33 ] },
    Vertex { a_Pos: [ -0.5, 0.33 ] },
];

fn main() {
    let (window, mut events_loop) = init_winit();

    let window_size = window.get_inner_size().unwrap();



    let mut running = true;

    while running {
        events_loop.poll_events(|event| {
            if let winit::Event::WindowEvent { event, .. } = event {
                match event {
                    winit::WindowEvent::KeyboardInput {
                        input: winit::KeyboardInput {
                            virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                            .. },
                        ..
                    } | winit::WindowEvent::Closed => running = false,
                    _ => (),
                }
            }
        });
    }
}

fn init_gfx() {
    unimplemented!();
}

fn cleanup() {
    unimplemented!();
}


// Initialize a new window
fn init_winit() -> (winit::Window, winit::EventsLoop) {
    // Create a new WindowBuilder
    let wb = winit::WindowBuilder::new()
        .with_dimensions(640, 480)                  // window dimensions in u32
        .with_title("Gfx HAL Demo".to_string());    // title of the window

    // Create a new event loop
    let mut events_loop = winit::EventsLoop::new();

    // Build the window
    let window = wb
        .build(&events_loop)    // Build a window with the given eventloop
        .unwrap();

    (window, events_loop)
}
