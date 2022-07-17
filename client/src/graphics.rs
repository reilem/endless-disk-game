use std::borrow::Cow;
use wgpu::{
    ColorTargetState, Device, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, Surface, SurfaceConfiguration, TextureFormat,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// EXAMPLE: FROM WGPU EXAMPLE HELLO_TRIANGLE

pub async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let (_size, surface, device, queue, mut config) = init_graphics(&window).await;

    // Load in the wgsl shader
    let shader = load_wgsl_shader(&device, include_str!("shader.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&create_render_pipeline_descriptor(
        &shader,
        &pipeline_layout,
        &[Some(config.format.into())],
    ));

    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| {
        log::debug!("event_loop called");
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = &shader;

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(new_size) => {
                    handle_resize(&mut config, &new_size, &surface, &device, &window)
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    handle_resize(&mut config, new_inner_size, &surface, &device, &window)
                }
                WindowEvent::CloseRequested => *control_flow = handle_close(),
                _ => {}
            },
            Event::RedrawRequested(_) => handle_redraw(&surface, &device, &render_pipeline, &queue),
            _ => {}
        }
    });
}

/**
 * HANDLES
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

fn handle_redraw(
    surface: &Surface,
    device: &Device,
    render_pipeline: &RenderPipeline,
    queue: &Queue,
) {
    let frame = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(render_pipeline);
        rpass.draw(0..3, 0..1);
    }

    queue.submit(Some(encoder.finish()));
    frame.present();
}

fn handle_close() -> ControlFlow {
    ControlFlow::Exit
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
) {
    // Create size, instance, surface & adapter
    let (size, surface, adapter) = init_adapter(&window).await;
    // Create the logical device and command queue
    let (device, queue) = init_device_queue(&adapter).await;
    // Get best texture format for adapter
    let texture_format = surface.get_supported_formats(&adapter)[0];
    // Create default surface config
    let config = init_default_surface_config(&size, &texture_format);
    (size, surface, device, queue, config)
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

fn load_wgsl_shader(device: &Device, shader_path: &str) -> ShaderModule {
    // Load the shaders from disk
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_path)),
    })
}

fn create_render_pipeline_descriptor<'a>(
    shader: &'a ShaderModule,
    pipeline_layout: &'a PipelineLayout,
    targets: &'a [Option<ColorTargetState>],
) -> RenderPipelineDescriptor<'a> {
    RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets,
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    }
}
