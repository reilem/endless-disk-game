use wgpu::{
    include_wgsl, Device, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceConfiguration, TextureFormat,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let (_size, surface, device, queue, mut config, render_pipeline) = init_graphics(&window).await;

    let mut cursor_x: f64 = 0.0;
    let mut cursor_y: f64 = 0.0;

    log::debug!("Starting event_loop");
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&surface, &device, &queue, &config, &render_pipeline);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event, window_id, ..
            } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => {
                    handle_resize(&mut config, &new_size, &surface, &device, &window)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    handle_resize(&mut config, new_inner_size, &surface, &device, &window)
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    cursor_x = position.x / (config.width as f64);
                    cursor_y = position.y / (config.height as f64);
                    window.request_redraw();
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let color = wgpu::Color {
                    r: cursor_x,
                    g: cursor_y * cursor_x,
                    b: cursor_y,
                    a: 1.0,
                };
                handle_redraw(&surface, &device, &render_pipeline, &queue, color)
            }
            _ => {}
        }
    });
}

/**
 * HANDLES
 */

/**
 * Handle resizing of the window
 */
fn handle_resize(
    config: &mut SurfaceConfiguration,
    new_size: &PhysicalSize<u32>,
    surface: &Surface,
    device: &Device,
    window: &Window,
) {
    if new_size.width > 0 && new_size.height > 0 {
        // Reconfigure the surface with the new size
        config.width = new_size.width;
        config.height = new_size.height;
        surface.configure(&device, &config);
        // On macos the window needs to be redrawn manually after resizing
        window.request_redraw();
    }
}

/**
 * Handle redraw events
 */
fn handle_redraw(
    surface: &Surface,
    device: &Device,
    render_pipeline: &RenderPipeline,
    queue: &Queue,
    color: wgpu::Color,
) {
    log::debug!("Redraw!!");
    // Get a TextureSurface "frame" from the surface that we can render to
    let frame = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");
    // Create a texture view with default settings
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    // Create a command encoder, makes command buffers to send to the gpu with commands in them
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("GPU Command Encoder"),
    });
    // Start a new code block so that the mutable encoder borrow is dropped after
    {
        // Clears the screen with a single color
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main render pass"),
            // Describe where to draw color to
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                // Which texture view to save color to
                view: &view,
                // Which texture view receives resolved output (view by default if not multisampling)
                resolve_target: None,
                // What to do with colors on the screen
                ops: wgpu::Operations {
                    // What to do with colors in previous frame, in this case: Clear to BLACK
                    load: wgpu::LoadOp::Clear(color),
                    // Wether to store the render result or not
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        // Give the render pass the pipeline to use
        rpass.set_pipeline(render_pipeline);
        // Draw something with 3 vertices and 1 instance
        rpass.draw(0..3, 0..1);
    }
    // Finish command buffer and submit it to GPU's render
    queue.submit(Some(encoder.finish()));
    frame.present();
}

/**
 * INITIALISATION STUFF
 */

async fn init_graphics(
    window: &Window,
) -> (
    PhysicalSize<u32>,
    wgpu::Surface,
    wgpu::Device,
    wgpu::Queue,
    wgpu::SurfaceConfiguration,
    wgpu::RenderPipeline,
) {
    // Create size, instance, surface & adapter
    let (size, surface, adapter) = init_adapter(&window).await;
    // Create the logical device and command queue
    let (device, queue) = init_device_queue(&adapter).await;
    // Get best texture format for adapter
    let texture_format = surface.get_supported_formats(&adapter)[0];
    // Create default surface config
    let config = init_default_surface_config(&size, &texture_format);
    // Configure the surface to use this device & configuration
    surface.configure(&device, &config);
    // Create render pipeline
    let render_pipeline = init_render_pipeline(&device, &config);
    (size, surface, device, queue, config, render_pipeline)
}

async fn init_adapter(window: &Window) -> (PhysicalSize<u32>, wgpu::Surface, wgpu::Adapter) {
    let size = window.inner_size();
    // Instance = Main purpose of instance: create surface & adapters
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    // Surface = part of the window we draw to (window needs to implement raw-window-handler, winit does this)
    let surface = unsafe { instance.create_surface(&window) };
    // Adapter = handle to graphics card, used to create device & queue
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(), // default, LowPower, HighPower
            force_fallback_adapter: false, // Force to work on all systems (uses software instead of hardware)
            compatible_surface: Some(&surface), // Find an adapter which can render to the requested surface
        })
        .await
        .expect("Failed to find an appropriate adapter");
    (size, surface, adapter)
}

async fn init_device_queue(adapter: &wgpu::Adapter) -> (Device, Queue) {
    // Use adapter to create Device and Queue
    // Device = a connection to a physical device
    // Queue = executes command buffers on device
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                // Do not enable any extra features
                features: wgpu::Features::empty(),
                // Limits = what kind of resources we can create
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                // WebGL doesn't support all wgpu features so select specific ones for webgl if running wasm
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    // Note: If some older devices do not work replace this with: downlevel_default()
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        )
        .await
        .expect("Failed to create device")
}

fn init_default_surface_config(
    size: &PhysicalSize<u32>,
    format: &TextureFormat,
) -> wgpu::SurfaceConfiguration {
    // Defaults to support most devices
    wgpu::SurfaceConfiguration {
        // Define how SurfaceTexture will be used, RENDER_ATTACHMENT = write to screen
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        // Define the format that will be used to store the SurfaceTexture in the GPU
        format: *format,
        // Width of the surface (usually width of window)
        width: size.width,
        // Height of surface (usually height of window)
        height: size.height,
        // Determines how to sync surface with display, Fifo (always supported) = cap display rate to Fps of display (VSync)
        present_mode: wgpu::PresentMode::Fifo,
    }
}

fn init_render_pipeline(
    device: &wgpu::Device,
    config: &SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    // Load in the wgsl shader
    let shader = device.create_shader_module(include_wgsl!("shaders/tutorial.wgsl"));
    // Set pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Main render pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    // Create render pipeline
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main", // Entrypoint vertex shader function inside shader
            buffers: &[],           // Vertices you want to pass to the shader
        },
        fragment: Some(wgpu::FragmentState {
            // Some() because this is optional
            module: &shader,
            entry_point: "fs_main", // Entrypoint fragment shader function inside shader
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,                  // Use the surface's format
                blend: Some(wgpu::BlendState::REPLACE), // Replace all colors
                write_mask: wgpu::ColorWrites::ALL,     // Write to all channels (r,g,b,a)
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList, // Every three vertices correspond to a triangle
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // Arrange triangles in counter-clockwise direction to face front
            cull_mode: Some(wgpu::Face::Back), // Cull any triangles not facing front
            polygon_mode: wgpu::PolygonMode::Fill, // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            unclipped_depth: false,                // Requires Features::DEPTH_CLIP_CONTROL
            conservative: false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,                         // How many samples the pipeline will use (we use 1)
            mask: !0,                         // Which samples should be active (we use all of them)
            alpha_to_coverage_enabled: false, // Related to anti-aliasing
        },
        multiview: None, // How many array layers the render attachment will have
    })
}
