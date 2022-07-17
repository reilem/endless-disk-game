use wgpu::{
    include_wgsl, util::DeviceExt, Device, PipelineLayoutDescriptor, Queue,
    RenderPipelineDescriptor, SurfaceConfiguration, TextureFormat,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// TODO: Refactor this into multiple files
// Keeping as-is for now to make sure that when we do the division we have all the information required

struct CursorPosition {
    x: f64,
    y: f64,
}

struct GraphicState {
    size: PhysicalSize<u32>,
    cursor: CursorPosition,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    // This tells the render_pipeline how to read the buffer
    // Since the buffer is an array of bytes it will need to be told how to handle those bytes
    fn buffer_layout_description<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // The size of each vertex in bytes
            step_mode: wgpu::VertexStepMode::Vertex, // Go over each one vertex-by-vertex
            attributes: &Self::ATTRIBS,
            // More verbose way:
            // attributes: &[
            //     // Describe the meaning of each struct field
            //     wgpu::VertexAttribute {
            //         offset: 0,
            //         shader_location: 0, // Corresponds to `@location(x)` in WGSL shader
            //         format: wgpu::VertexFormat::Float32x3, // 3 floats = size of position field
            //     },
            //     wgpu::VertexAttribute {
            //         offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress, // Give it an offset corresponding to position field
            //         shader_location: 1,
            //         format: wgpu::VertexFormat::Float32x3, // 3 floats = size of color field
            //     },
            // ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.5, 0.0, 0.5],
    }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4, // Triangle 1
    1, 2, 4, // Triangle 2
    2, 3, 4, // Triangle 3
];

pub async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let mut state = GraphicState::new(&window).await;

    log::debug!("Starting event_loop");
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = &state;

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event, window_id, ..
            } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => state.handle_resize(&new_size, &window),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.handle_resize(&new_inner_size, &window)
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
                    state.handle_cursor(position);
                    window.request_redraw();
                }
                _ => {}
            },
            Event::RedrawRequested(_) => state.handle_redraw(),
            _ => {}
        }
    });
}

impl GraphicState {
    /**
     * INITIALISATION STUFF
     */
    async fn new(window: &Window) -> Self {
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
        // Create vertex buffer
        let vertex_buffer = init_vertex_buffer(&device);
        // Create index buffer
        let index_buffer = init_index_buffer(&device);
        GraphicState {
            size,
            cursor: CursorPosition { x: 0.0, y: 0.0 },
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: INDICES.len() as u32,
        }
    }

    /**
     * HANDLES
     */

    fn handle_cursor(&mut self, position: PhysicalPosition<f64>) {
        self.cursor.x = position.x / (self.size.width as f64);
        self.cursor.y = position.y / (self.size.height as f64);
    }

    /**
     * Handle resizing of the window
     */
    fn handle_resize(&mut self, new_size: &PhysicalSize<u32>, window: &Window) {
        if new_size.width > 0 && new_size.height > 0 {
            // Reconfigure the surface with the new size
            self.size = *new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // On macos the window needs to be redrawn manually after resizing
            window.request_redraw();
        }
    }

    /**
     * Handle redraw events
     */
    fn handle_redraw(&self) {
        // Determine background color based on cursor position
        let background_color = wgpu::Color {
            r: self.cursor.x,
            g: self.cursor.y * self.cursor.x,
            b: self.cursor.y,
            a: 1.0,
        };

        log::debug!("Redraw!!");
        // Get a TextureSurface "frame" from the surface that we can render to
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        // Create a texture view with default settings
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // Create a command encoder, makes command buffers to send to the gpu with commands in them
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                        load: wgpu::LoadOp::Clear(background_color),
                        // Wether to store the render result or not
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            // Give the render pass the pipeline to use
            rpass.set_pipeline(&self.render_pipeline);
            // Set the vertex buffer
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // Set the index buffer
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // Draw indices instead of vertices
            rpass.draw_indexed(0..self.index_count, 0, 0..1);
        }
        // Finish command buffer and submit it to GPU's render
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
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
            buffers: &[
                // Description of the buffers you want to pass to the shader
                Vertex::buffer_layout_description(),
            ],
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

fn init_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Main vertex buffer"),
        contents: bytemuck::cast_slice(&VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

fn init_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Main index buffer"),
        contents: bytemuck::cast_slice(&INDICES),
        usage: wgpu::BufferUsages::INDEX,
    })
}
