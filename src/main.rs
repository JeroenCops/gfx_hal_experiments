#[cfg(feature = "dx12")]
extern crate gfx_backend_dx12 as back;
#[cfg(feature = "vulkan")]
extern crate gfx_backend_vulkan as back;
extern crate gfx_hal as hal;

extern crate winit;

use hal::{buffer, format, command, format as f, image as i, memory as m, pass, pso, pool};
use hal::{Device, Instance, Surface, IndexType, PhysicalDevice, Swapchain};
use hal::{
    DescriptorPool, FrameSync, Primitive,
    SwapchainConfig, Backbuffer
};
use hal::format::{ChannelType, Swizzle};
use hal::pso::{PipelineStage, ShaderStageFlags, Specialization};
use hal::queue::Submission;

// mod context;

const ENTRY_NAME: &str = "main";

const COLOR_RANGE: i::SubresourceRange = i::SubresourceRange {
    aspects: f::Aspects::COLOR,
    levels: 0 .. 1,
    layers: 0 .. 1,
};

// Vertex Layout
#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Vertex {
    a_Pos: [f32; 3],
}

const VERTICES: [Vertex; 2] = [
    Vertex { a_Pos: [ 0.0,  0.0, 0.0 ] },
    Vertex { a_Pos: [ 0.5, -0.5, 0.0 ] },
];

const VERTEX_SIZE: u32 = 50;

const CLEAR_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

fn main() {
    let mut events_loop = winit::EventsLoop::new();
    let window_builder = winit::WindowBuilder::new()
                            .with_dimensions(500, 500)
                            .with_title("Triangle Example".to_string());
    let window = window_builder.build(&events_loop).unwrap();

    let instance = back::Instance::create("Gfx Render", 1);
    let mut surface = instance.create_surface(&window);
    let mut adapter = instance.enumerate_adapters().remove(0);

    let window_size = window.get_inner_size().unwrap();
    let pixel_width = window_size.0 as u16;
    let pixel_height = window_size.1 as u16;

    println!("Adapter: {:?}", adapter.info);

    let surface_format = surface
        .capabilities_and_formats(&adapter.physical_device)
        .1
        .map_or(
            format::Format::Rgba8Srgb,
                |formats| {
                    formats
                        .into_iter()
                        .find(|format| {
                            format.base_format().1 == ChannelType::Srgb
                        })
                        .unwrap()
                }
        );

    let memory_types = adapter.physical_device.memory_properties().memory_types;
    let limits = adapter.physical_device.limits();

    let (mut device, mut queue_group) = adapter
        .open_with::<_, hal::General>(1, |family| {
            surface.supports_queue_family(family)
        }).unwrap();

    let mut command_pool = device.create_command_pool_typed(&queue_group, pool::CommandPoolCreateFlags::empty(), 1);
    let mut queue = &mut queue_group.queues[0];

    println!("Surface format: {:?}", surface_format);
    let swap_config = SwapchainConfig::new()
                        .with_color(surface_format)
                        .with_image_usage(i::Usage::COLOR_ATTACHMENT);

    let (mut swap_chain, backbuffer) = device.create_swapchain(&mut surface, swap_config);

    let render_pass = {
        let color_attachment = pass::Attachment {
            format: Some(surface_format),
            ops: pass::AttachmentOps::new(pass::AttachmentLoadOp::Clear, pass::AttachmentStoreOp::Store),
            stencil_ops: pass::AttachmentOps::DONT_CARE,
            layouts: i::Layout::Undefined .. i::Layout::Present,
        };

        let subpass = pass::SubpassDesc {
            colors: &[(0, i::Layout::ColorAttachmentOptimal)],
            depth_stencil: None,
            inputs: &[],
            preserves: &[],
        };

        device.create_render_pass(&[color_attachment], &[subpass], &[])
    };

    let set_layout = device.create_descriptor_set_layout(&[],);

    let pipeline_layout = device.create_pipeline_layout(
        Some(&set_layout),                          // Descriptor Set layouts
        &[(pso::ShaderStageFlags::VERTEX, 0..1)],    // Push Constants Ranges
    );

    let mut descriptor_pool = device.create_descriptor_pool(
        1,
        &[],
    );

    // Allocate a new descriptor set from the global descriptor pool
    let descriptor_set = descriptor_pool.allocate_set(&set_layout).unwrap();

    let pipeline = {
        let vs_module = device
            .create_shader_module(include_bytes!("shader/vert.spv"))
            .unwrap();
        let fs_module = device
            .create_shader_module(include_bytes!("shader/frag.spv"))
            .unwrap();

        let pipeline = {
            let (vs_entry, fs_entry) = (
                // Vertex shader
                pso::EntryPoint::<back::Backend> {
                    // Main entry point for the shader
                    entry: ENTRY_NAME,
                    // Vertex Shader module
                    module: &vs_module,
                    specialization: &[],
                },
                // Fragment shader
                pso::EntryPoint::<back::Backend> {
                    // Main entry point for the shader
                    entry: ENTRY_NAME,
                    // Fragment Shader module
                    module: &fs_module,
                    specialization: &[],
                },
            );

            let shader_entries = pso::GraphicsShaderSet {
                vertex: vs_entry,
                hull: None,
                domain: None,
                geometry: None,
                fragment: Some(fs_entry),
            };

            let subpass = pass::Subpass { index: 0, main_pass: &render_pass };

            let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
                shader_entries,
                Primitive::PointList,
                pso::Rasterizer {
                    polygon_mode: pso::PolygonMode::Point,
                    cull_face: None,
                    front_face: pso::FrontFace::CounterClockwise,
                    depth_clamping: false,
                    depth_bias: None,
                    conservative: false,
                },
                &pipeline_layout,
                subpass,
            );

            // Color blend state describes how blend factors are calculated (if used).
            // We need one blend attachment state per color attachment (even if blending is not used
            pipeline_desc.blender.targets.push(pso::ColorBlendDesc(
                pso::ColorMask::ALL,
                pso::BlendState::Off,
            ));

            pipeline_desc.vertex_buffers.push(pso::VertexBufferDesc {
                stride: std::mem::size_of::<Vertex>() as u32,
                rate: 0, // rate (0 = INPUT_RATE_VERTEX) or (1 = INPUT_RATE_INSTANCE)
            });

            // Inpute attribute bindings describe shader attribute locations and memory layouts
            // These match the following shader layout (see vertex.vert):
            //	layout (location = 0) in vec3 inPos;

            // Attribute location 0: Position
            pipeline_desc.attributes.push(pso::AttributeDesc {
                location: 0,
                binding: 0,
                element: pso::Element {
                    // Position attribute is three 32 bit signed (SFLOAT) floats (R32 G32 B32)
                    format: f::Format::Rgb32Float,
                    offset: 0,
                },
            });

            device.create_graphics_pipelines(&[pipeline_desc])
        };

        device.destroy_shader_module(vs_module);
        device.destroy_shader_module(fs_module);

        pipeline
    };

    // Image view and render target creation.
    let (frame_images, framebuffers) = match backbuffer {
        Backbuffer::Images(images) => {
            let extent = i::Extent { width: pixel_width as _, height: pixel_height as _, depth: 1 };
            let pairs = images
                .into_iter()
                .map(|image| {
                    let rtv = device.create_image_view(
                        &image, i::ViewKind::D2, surface_format, Swizzle::NO, COLOR_RANGE.clone()
                        ).unwrap();
                    (image, rtv)
                })
                .collect::<Vec<_>>();
            let fbos = pairs
                .iter()
                .map(|&(_, ref rtv)| {
                    device.create_framebuffer(&render_pass, Some(rtv), extent).unwrap()
                })
                .collect();
            (pairs, fbos)
        }
        Backbuffer::Framebuffer(fbo) => {
            (Vec::new(), vec![fbo])
        }
    };

    // Vertex Buffer allocation
    let (vertex_buffer, vertex_memory) = {
        let buffer_stride = std::mem::size_of::<Vertex>() as u64;
        let buffer_len = VERTICES.len() as u64 * buffer_stride;
        let buffer_unbound = device.create_buffer(buffer_len, buffer::Usage::VERTEX).unwrap();
        let buffer_req = device.get_buffer_requirements(&buffer_unbound);

        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                buffer_req.type_mask & (1 << id) != 0 &&
                mem_type.properties.contains(m::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        let buffer_memory = device.allocate_memory(upload_type, buffer_req.size).unwrap();
        let vertex_buffer = device.bind_buffer_memory(&buffer_memory, 0, buffer_unbound).unwrap();

        {
            let mut vertices = device
                .acquire_mapping_writer::<Vertex>(&buffer_memory, 0..buffer_len)
                .unwrap();
            vertices.copy_from_slice(&VERTICES);
            device.release_mapping_writer(vertices);
        }

        (vertex_buffer, buffer_memory)
    };

    let viewport = pso::Viewport {
        rect: pso::Rect {
            x: 0, y: 0,
            w: pixel_width, h: pixel_height,
        },
        depth: 0.0 .. 1.0,
    };

    // Used for correct command ordering
    let mut frame_semaphore = device.create_semaphore();
    // Used to check draw command buffer completion
    let mut frame_fence = device.create_fence(false);

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

        device.reset_fence(&frame_fence);
        command_pool.reset();
        let frame = swap_chain.acquire_frame(FrameSync::Semaphore(&mut frame_semaphore));

        let submit = {
            let mut cmd_buffer = command_pool.acquire_command_buffer(false);

            // Update dynamic viewport state
            cmd_buffer.set_viewports(0, &[viewport.clone()]);
            // Update dynamic scissor state
            cmd_buffer.set_scissors(0, &[viewport.rect]);
            // Update Push Constants
            cmd_buffer.push_graphics_constants(&pipeline_layout, pso::ShaderStageFlags::VERTEX, 0, &[VERTEX_SIZE]);
            // Bind the rendering pipeline
            cmd_buffer.bind_graphics_pipeline(&pipeline[0].as_ref().unwrap());
            // Bind descriptor sets describing shader binding points
            cmd_buffer.bind_graphics_descriptor_sets(&pipeline_layout, 0, Some(&descriptor_set));
            // Bind triangle vertex buffer (contains position and colors)
            cmd_buffer.bind_vertex_buffers(pso::VertexBufferSet(vec![(&vertex_buffer, 0)]));

            {
                let mut encoder = cmd_buffer.begin_render_pass_inline(
                    &render_pass,
                    &framebuffers[frame.id()],
                    viewport.rect,
                    &[
                        command::ClearValue::Color(command::ClearColor::Float(CLEAR_COLOR)),
                    ],
                );
                encoder.draw(0..2, 0..1);
            }

            cmd_buffer.finish()
        };
        let submission = Submission::new()
            .wait_on(&[(&frame_semaphore, PipelineStage::BOTTOM_OF_PIPE)])
            .submit(Some(submit));

        queue.submit(submission, Some(&mut frame_fence));
        device.wait_for_fence(&frame_fence, !0);
        swap_chain.present(&mut queue, &[]);
    }
}
