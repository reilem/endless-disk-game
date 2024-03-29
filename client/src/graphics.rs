use instant::{Duration, Instant};
use std::collections::HashSet;
use wgpu::{
    include_wgsl, util::DeviceExt, Device, PipelineLayoutDescriptor, Queue,
    RenderPipelineDescriptor, SurfaceConfiguration, TextureFormat,
};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyboardInput, MouseButton, StartCause, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

/// Take an fps (Frames Per Second) float number and returns the amount of nanoseconds between updates
/// required to achieve that fps.
macro_rules! refresh_time {
    ( $fps:expr ) => {
        (1000000000.0 as f64 / $fps as f64) as u32
    };
}

type WindowSize = LogicalSize<u32>;

// TODO: Refactor this into multiple files
// Keeping as-is for now to make sure that when we do the division we have all the information required

enum MoveDirection {
    Left,
    DownLeft,
    Down,
    DownRight,
    Right,
    UpRight,
    Up,
    UpLeft,
}

#[derive(Debug)]
struct Position {
    x: f64,
    y: f64,
}

struct GraphicState {
    size: WindowSize,
    scale_factor: f64,
    cursor: Position,
    pressed_keys: HashSet<VirtualKeyCode>,
    mouse_down: bool,
    player: Position, // TODO: this needs to be extracted since it is not at all graphics related
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    diffuse_bind_group: wgpu::BindGroup,
    projection_bind_group: wgpu::BindGroup,
    projection_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    // This tells the render_pipeline how to read the buffer
    // Since the buffer is an array of bytes it will need to be told how to handle those bytes
    fn buffer_layout_description<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // The size of each vertex in bytes
            step_mode: wgpu::VertexStepMode::Vertex, // Go over each one vertex-by-vertex
            attributes: &Self::ATTRIBS,
        }
    }
}

const SQUARE_SIZE: f32 = 96.0;
const DEFAULT_UPDATE_TIME: u32 = refresh_time!(60.0);
const SPEED: f64 = 0.004;

fn next_update(wait_time: u32) -> Instant {
    Instant::now()
        .checked_add(Duration::new(0, wait_time))
        .expect("Failed to set next update time")
}

pub async fn run_loop(event_loop: EventLoop<()>, window: Window) {
    let mut state = GraphicState::new(&window).await;

    let mut last_update = Instant::now();
    let update_wait_time = window
        .current_monitor()
        .and_then(|monitor| monitor.video_modes().next())
        .map(|mode| refresh_time!(mode.refresh_rate()))
        .unwrap_or(DEFAULT_UPDATE_TIME);

    log::info!("Starting event loop");
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = &state;

        match event {
            Event::NewEvents(start_cause) => match start_cause {
                StartCause::Init => {
                    *control_flow = ControlFlow::WaitUntil(next_update(update_wait_time))
                }
                StartCause::ResumeTimeReached { .. } => {
                    // We use our own custom time check because the ResumeTimeReached time is based on loop events.
                    // If you have lots of interrupting events (e.g. mouse movement) then this will interrupt the WaitUntil and
                    // create a new "start" time. So when the resume event is reached the diff between the "start" time and
                    // requested_resume could be very small because 1ms earlier a mouse event had occurred.
                    let now = Instant::now();
                    state.update(&window, now.duration_since(last_update));
                    *control_flow = ControlFlow::WaitUntil(next_update(update_wait_time));
                    last_update = now;
                }
                _ => {}
            },
            Event::WindowEvent {
                event, window_id, ..
            } if window_id == window.id() => match event {
                WindowEvent::Resized(new_size) => state.handle_resize(&new_size, None, &window),
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor,
                } => state.handle_resize(new_inner_size, Some(scale_factor), &window),
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
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    } => state.handle_key_press(&keycode),
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(keycode),
                        ..
                    } => state.handle_key_release(&keycode),
                    _ => {}
                },
                WindowEvent::CursorMoved { position, .. } => state.handle_cursor(position),
                WindowEvent::MouseInput {
                    state: mouse_state,
                    button: MouseButton::Left,
                    ..
                } => state.handle_mouse_input(&mouse_state),
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
        let player = Position { x: 0.0, y: 0.0 };
        // Create size, instance, surface & adapter
        let (size, scale_factor, surface, adapter) = init_adapter(window).await;
        // Create the logical device and command queue
        let (device, queue) = init_device_queue(&adapter).await;
        // Get best texture format for adapter
        let texture_format = surface.get_supported_formats(&adapter)[0];
        // Create default surface config
        let config = init_default_surface_config(&size.to_physical(scale_factor), &texture_format);
        // Configure the surface to use this device & configuration
        surface.configure(&device, &config);
        // Create texture bind group
        let (diffuse_bind_group, diffuse_bind_group_layout) = init_texture(&device, &queue);
        // Create projection matrix buffer
        let projection_bind_group_layout = init_projection_bind_group_layout(&device);
        let projection_buffer =
            init_projection_matrix_buffer(&device, &size, &player, wgpu::BufferUsages::COPY_DST);
        let projection_bind_group =
            init_projection_bind_group(&device, &projection_buffer, &projection_bind_group_layout);
        // Create render pipeline
        let render_pipeline = init_render_pipeline(
            &device,
            &config,
            &diffuse_bind_group_layout,
            &projection_bind_group_layout,
        );
        // Create vertex buffer
        let (vertex_buffer, index_buffer, index_count) =
            init_vertex_index_buffer(&device, &size, &player);

        GraphicState {
            size,
            scale_factor,
            cursor: Position { x: 0.0, y: 0.0 },
            pressed_keys: HashSet::new(),
            mouse_down: false,
            player,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count,
            diffuse_bind_group,
            projection_buffer,
            projection_bind_group,
        }
    }

    fn update(&mut self, window: &Window, time_elapsed: Duration) {
        let delta_time = time_elapsed.as_millis() as f64;
        let mut move_direction: Option<MoveDirection> = None;
        let delta_space = SPEED * delta_time;
        if !self.pressed_keys.is_empty() {
            if self.pressed_keys.contains(&VirtualKeyCode::Left) {
                if self.pressed_keys.contains(&VirtualKeyCode::Up) {
                    move_direction = Some(MoveDirection::UpLeft);
                } else if self.pressed_keys.contains(&VirtualKeyCode::Down) {
                    move_direction = Some(MoveDirection::DownLeft);
                } else if self.pressed_keys.contains(&VirtualKeyCode::Right) {
                    move_direction = None;
                } else {
                    move_direction = Some(MoveDirection::Left);
                }
            } else if self.pressed_keys.contains(&VirtualKeyCode::Down) {
                if self.pressed_keys.contains(&VirtualKeyCode::Up) {
                    move_direction = None;
                } else if self.pressed_keys.contains(&VirtualKeyCode::Left) {
                    move_direction = Some(MoveDirection::DownLeft);
                } else if self.pressed_keys.contains(&VirtualKeyCode::Right) {
                    move_direction = Some(MoveDirection::DownRight);
                } else {
                    move_direction = Some(MoveDirection::Down);
                }
            } else if self.pressed_keys.contains(&VirtualKeyCode::Right) {
                if self.pressed_keys.contains(&VirtualKeyCode::Up) {
                    move_direction = Some(MoveDirection::UpRight);
                } else if self.pressed_keys.contains(&VirtualKeyCode::Left) {
                    move_direction = None;
                } else if self.pressed_keys.contains(&VirtualKeyCode::Down) {
                    move_direction = Some(MoveDirection::DownRight);
                } else {
                    move_direction = Some(MoveDirection::Right);
                }
            } else if self.pressed_keys.contains(&VirtualKeyCode::Up) {
                if self.pressed_keys.contains(&VirtualKeyCode::Right) {
                    move_direction = Some(MoveDirection::UpRight);
                } else if self.pressed_keys.contains(&VirtualKeyCode::Down) {
                    move_direction = None;
                } else if self.pressed_keys.contains(&VirtualKeyCode::Left) {
                    move_direction = Some(MoveDirection::UpLeft);
                } else {
                    move_direction = Some(MoveDirection::Up);
                }
            }
            if let Some(direction) = &move_direction {
                match direction {
                    MoveDirection::Left => self.player.x -= delta_space,
                    MoveDirection::DownLeft => {
                        self.player.x -= delta_space / 2.0;
                        self.player.y -= delta_space / 2.0;
                    }
                    MoveDirection::Down => self.player.y -= delta_space,
                    MoveDirection::DownRight => {
                        self.player.x += delta_space / 2.0;
                        self.player.y -= delta_space / 2.0;
                    }
                    MoveDirection::Right => self.player.x += delta_space,
                    MoveDirection::UpRight => {
                        self.player.x += delta_space / 2.0;
                        self.player.y += delta_space / 2.0;
                    }
                    MoveDirection::Up => self.player.y += delta_space,
                    MoveDirection::UpLeft => {
                        self.player.x -= delta_space / 2.0;
                        self.player.y += delta_space / 2.0;
                    }
                }
            }
        } else if self.mouse_down {
            let x = self.cursor.x;
            let y = self.cursor.y;
            // Project the coordinates into clip-space
            // Old space: top-left: (0,0), bottom-right: (1,1)
            // Clip space: top-left (-1,1), bottom-right: (1,-1)
            let clip_x = (x - 0.5) * 2.0;
            let clip_y = (y - 0.5) * -2.0;
            // Cursor deadzone is at -0.25 to 0.25
            // If the cursor is in this deadzone the sprite will move slower the closer the cursor is
            // and faster the further away the cursor is. Beyond the deadzone the sprite will move at max speed.
            let deadzone_percentage = 0.25;
            let threshold_x = (clip_x / deadzone_percentage).min(1.0).max(-1.0);
            let threshold_y = (clip_y / deadzone_percentage).min(1.0).max(-1.0);
            // Calculate the relative weights to be used to correct the x and y movement components to ensure constant speed
            let alpha = clip_y / clip_x;
            let k_x = 1.0 / (1.0 + alpha.abs());
            let k_y = 1.0 - k_x;
            // Multiply the thresholded values by these weights to find the true x and y movement components
            let corrected_x = k_x * threshold_x;
            let corrected_y = k_y * threshold_y;
            self.player.x += corrected_x * delta_space;
            self.player.y += corrected_y * delta_space;
        }

        if move_direction.is_some() || self.mouse_down {
            self.refresh_buffers();
            window.request_redraw();
        }
    }

    /**
     * HANDLES
     */

    fn handle_cursor(&mut self, physical: PhysicalPosition<f64>) {
        let position: LogicalPosition<f64> = physical.to_logical(self.scale_factor);
        self.cursor.x = position.x / (self.size.width as f64);
        self.cursor.y = position.y / (self.size.height as f64);
    }

    fn handle_key_press(&mut self, keycode: &VirtualKeyCode) {
        self.pressed_keys.insert(*keycode);
    }

    fn handle_key_release(&mut self, keycode: &VirtualKeyCode) {
        self.pressed_keys.remove(keycode);
    }

    fn handle_mouse_input(&mut self, state: &ElementState) {
        self.mouse_down = *state == ElementState::Pressed;
    }

    /**
     * Handle resizing of the window
     */
    fn handle_resize(
        &mut self,
        new_size: &PhysicalSize<u32>,
        scale_factor: Option<f64>,
        window: &Window,
    ) {
        if new_size.width > 0 && new_size.height > 0 {
            // Reconfigure the surface with the new size
            self.scale_factor = scale_factor.unwrap_or(self.scale_factor);
            self.size = new_size.to_logical(self.scale_factor);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.refresh_buffers();
            // On macos the window needs to be redrawn manually after resizing
            window.request_redraw();
        }
    }

    fn refresh_buffers(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Refresh Buffer Command Encoder"),
            });
        // Creates a new projection buffer and copies it over to the existing one
        let next_projection_buffer = init_projection_matrix_buffer(
            &self.device,
            &self.size,
            &self.player,
            wgpu::BufferUsages::COPY_SRC,
        );
        encoder.copy_buffer_to_buffer(
            &next_projection_buffer,
            0,
            &self.projection_buffer,
            0,
            std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        );

        (self.vertex_buffer, self.index_buffer, self.index_count) =
            init_vertex_index_buffer(&self.device, &self.size, &self.player);
        self.queue.submit(Some(encoder.finish()));
    }

    /**
     * Handle redraw events
     */
    fn handle_redraw(&self) {
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        // Wether to store the render result or not
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            // Give the render pass the pipeline to use
            rpass.set_pipeline(&self.render_pipeline);
            // Set the texture bind group
            rpass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            // Set the projection matrix bind group
            rpass.set_bind_group(1, &self.projection_bind_group, &[]);
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

async fn init_adapter(window: &Window) -> (WindowSize, f64, wgpu::Surface, wgpu::Adapter) {
    let physical_size = window.inner_size();
    let scale_factor = window.scale_factor();
    let size = LogicalSize::from_physical(physical_size, scale_factor);
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
    (size, scale_factor, surface, adapter)
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

// TODO: this is way too big. I think textures will need their own module in the future
// We can then also use some compiler flags to load in images in different ways
fn init_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    // Load in bytes from file (good enough for now since all our textures will be very small)
    let diffuse_bytes = include_bytes!("textures/atlas-1.png");
    // Turn the bytes into an image
    let diffuse_image = image::load_from_memory(diffuse_bytes).expect("Failed to load image");
    // Get Vec of rgba bytes
    let diffuse_rgba = diffuse_image.to_rgba8();
    use image::GenericImageView;
    // Get dimensions of the image
    let dimensions = diffuse_image.dimensions();

    // Define texture size
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    // Create texture object
    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2, // 2D texture
        // Most images use sRGB
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        // TEXTURE_BINDING = use this texture in shaders
        // COP_DST = copy data to this texture
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("Happy tree texture"),
    });

    // Write texture data into the texture
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &diffuse_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
            rows_per_image: std::num::NonZeroU32::new(dimensions.1),
        },
        texture_size,
    );

    // Texture view offers a view into our texture
    let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    // Sampler controls how the texture is sampled in the shaders
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge, // horizontal clamping (like GLSL) what to do if sampler gets coords outside of texture
        address_mode_v: wgpu::AddressMode::ClampToEdge, // vertical clamping (like GLSL)
        mag_filter: wgpu::FilterMode::Nearest,          // magnification filtering (like GLSL)
        min_filter: wgpu::FilterMode::Nearest,          // minimisation filtering (like GLSL)
        ..Default::default()
    });

    // Bind group layout will be used to crate a bind group
    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // Sampled texture at binding 0
                    binding: 0,
                    // Only visible to fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // Type of binding
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    // If Some = Indicates that this entry is an array or a TEXTURE_BINDING_ARRAY
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // Sampler at binding 1
                    binding: 1,
                    // Only visible to fragment shader
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Main texture bind group layout"),
        });

    // Create texture bind group, each texture will require their own bind group
    // This is the final object required to use the texture
    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                // entry binds the previously created texture view
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            },
            wgpu::BindGroupEntry {
                // entry binds the previously created sampler
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ],
        label: Some("Main texture bind group"),
    });
    (texture_bind_group, texture_bind_group_layout)
}

fn init_render_pipeline(
    device: &wgpu::Device,
    config: &SurfaceConfiguration,
    diffuse_bind_group_layout: &wgpu::BindGroupLayout,
    projection_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Load in the wgsl shader
    let shader = device.create_shader_module(include_wgsl!("shaders/tutorial.wgsl"));
    // Set pipeline layout
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Main render pipeline layout"),
        bind_group_layouts: &[diffuse_bind_group_layout, projection_bind_group_layout],
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
                format: config.format,                         // Use the surface's format
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), // Replace all colors
                write_mask: wgpu::ColorWrites::ALL,            // Write to all channels (r,g,b,a)
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

/**
 * Always returns an uneven number, adds one if even to ensure full grid coverage.
 */
fn keep_uneven(value: u32) -> u32 {
    if value % 2 == 0 {
        value + 1
    } else {
        value
    }
}

/**
 * Returns the number of squares that will be needed to fill the window in squares
 * with size SQUARE_SIZE based on given parameter.
 */
fn number_of_squares(parameter: u32) -> u32 {
    let divided = (parameter as f32) / SQUARE_SIZE;
    let ceiled = divided.ceil();
    keep_uneven(ceiled as u32)
}

/**
 * Returns the number of squares that will be needed to fill the window horizontally
 */
fn number_of_squares_horionztally(size: &WindowSize) -> u32 {
    number_of_squares(size.width)
}

/**
 * Returns the number of squares that will be needed to fill the window vertically
 */
fn number_of_squares_vertically(size: &WindowSize) -> u32 {
    number_of_squares(size.height)
}

/**
 * Takes a square count of the world and an offset (player position) and calculates the lowest index in the range which will
 * be used to calculate the world grid.
 *
 * By dividing and ceiling the negation of the size by two we get the lowest index of the left-most (or bottom-most) square in the grid.
 * By adding the floor of the offset we ensure that negative numbers are rounded to their lowest value
 * and an extra square is always prepared in the grid before it comes into view.
 */
fn grid_range_start(square_count: f32, offset: f32) -> i32 {
    ((-1.0 * square_count) / 2.0).ceil() as i32 + offset.floor() as i32
}

/**
 * Take square count of the world and an offset (player position) and calculates the highest index in the range which will
 * be used during calculation of the world grid.
 *
 * We divide the square count by two and floor it to get the bottom-left coordinate of the right most square in the grid. This
 * could cause glitches when even numbers are passed to this calculation. But we fix this by ensuring the sizes
 * of the grid are always uneven numbers. At the end we add one because range calculations are not inclusive in rust.
 */
fn grid_range_end(square_count: f32, offset: f32) -> i32 {
    (square_count / 2.0).floor() as i32 + offset.ceil() as i32 + 1
}

fn texture_coords(index: u16) -> [[f32; 2]; 4] {
    let tex_width = 1.0 / 3.0;
    let tex_start = (index as f32) * tex_width;
    let tex_end = tex_start + (1.0 * tex_width);
    [
        [tex_start, 1.0],
        [tex_end, 1.0],
        [tex_end, 0.0],
        [tex_start, 0.0],
    ]
}

fn vertices_for_coords(x: f32, y: f32, tex_index: u16) -> Vec<Vertex> {
    let is_mid_ground = x == 0.0 && y == 0.0 && tex_index == 0;
    let tex_coords = texture_coords(tex_index);
    Vec::from([
        Vertex {
            position: [x, y],
            tex_coords: if is_mid_ground {
                tex_coords[1]
            } else {
                tex_coords[0]
            },
        },
        Vertex {
            position: [x + 1.0, y],
            tex_coords: if is_mid_ground {
                tex_coords[2]
            } else {
                tex_coords[1]
            },
        },
        Vertex {
            position: [x + 1.0, y + 1.0],
            tex_coords: if is_mid_ground {
                tex_coords[3]
            } else {
                tex_coords[2]
            },
        },
        Vertex {
            position: [x, y + 1.0],
            tex_coords: if is_mid_ground {
                tex_coords[0]
            } else {
                tex_coords[3]
            },
        },
    ])
}

fn indices_for_index(index: u16, offset: u16) -> Vec<u16> {
    let i = index * offset;
    Vec::from([i, i + 1, i + 3, i + 1, i + 2, i + 3])
}

// NOTE: this function is just a temp fix to make the rest of the code more readable
fn add_int_square(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    index: &mut u16,
    x: i32,
    y: i32,
    tex_index: u16,
) {
    add_square(vertices, indices, index, x as f64, y as f64, tex_index)
}

fn add_square(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    index: &mut u16,
    x: f64,
    y: f64,
    tex_index: u16,
) {
    let mut next_vertices = vertices_for_coords(x as f32, y as f32, tex_index);
    let vertex_count = next_vertices.len() as u16;
    vertices.append(&mut next_vertices);
    indices.append(&mut indices_for_index(*index, vertex_count).to_vec());
    *index += 1;
}

// TODO: Get rid of all these tuple returns and make sure it returns a proper struct instead to avoid confusion
fn init_vertex_index_buffer(
    device: &wgpu::Device,
    size: &WindowSize,
    player: &Position,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let horizontal_len = number_of_squares_horionztally(size) as f32;
    let vertical_len = number_of_squares_vertically(size) as f32;

    let y_start = grid_range_start(vertical_len, player.y as f32);
    let y_end = grid_range_end(vertical_len, player.y as f32);
    let x_start = grid_range_start(horizontal_len, player.x as f32);
    let x_end = grid_range_end(horizontal_len, player.x as f32);

    // Add world ground grid
    let mut index = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            add_int_square(&mut vertices, &mut indices, &mut index, x, y, 0);
        }
    }
    // Add static world objects
    add_int_square(&mut vertices, &mut indices, &mut index, 2, -1, 1);
    add_int_square(&mut vertices, &mut indices, &mut index, -3, 2, 1);
    add_int_square(&mut vertices, &mut indices, &mut index, -4, -1, 1);
    add_int_square(&mut vertices, &mut indices, &mut index, 5, 3, 1);
    // Add player
    add_square(
        &mut vertices,
        &mut indices,
        &mut index,
        player.x,
        player.y,
        2,
    );

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Main vertex buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Main index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    (vertex_buffer, index_buffer, indices.len() as u32)
}

/**
 * The following is an explanation for the used projection matrix.
 *
 * Scaling x by (2 / horizontal_squares) & y by (2 / vertical_squares) will scale the squares to fit in the clip space.
 * This is because the clip space has a width of 2 (-1 to +1), and we are trying to fit in "horizontal_squares" number of squares in the width
 * and "vertical_squares" number of squares in the height, so (2 / horizontal_squares) will give us the size each square will need
 * to be to fit inside the clip-space. Multiplying each grid coordinate by this number will resize each vertex to fit inside the grid.
 * This gives us: clip_scale = 2 / horizontal_squares
 *
 * After this transformation the grid will be contained within the clip space (-1 to 1). However it will be streched!
 * To solve this we add a correction scaling. We want each square to be of size SQUARE_SIZE but after initial scaling they will have a width
 * of (real_square_width = window_width / horizontal_squares). To correct this we want to find X for: (real_square_width * X = SQUARE_SIZE).
 * Some basic algebra:
 * Given: real_square_width = window_width / horizontal_squares
 * Find "correction" in: real_square_width * correction = SQUARE_SIZE
 * => real_square_width * correction = SQUARE_SIZE
 * => correction = SQUARE_SIZE / real_square_width
 * => correction = SQUARE_SIZE / (window_width / horizontal_squares)
 * => correction = SQUARE_SIZE * (horizontal_squares / window_width)
 *
 * We want to scale each vector by both the clip_scale (to fit them in clip space) and correction (to give them correct size).
 * As the final scaling factor we use:
 * scale = clip_scale * correction
 * => scale = (2 / horizontal_squares) * SQUARE_SIZE * (horizontal_squares / window_width)
 * => scale = (2 * SQUARE_SIZE) * (1 / window_width)
 * => scale = (2 * SQUARE_SIZE) / window_width
 * Which give us a final scaling factors of:
 * scale_x = (2 * SQUARE_SIZE) / window_width
 * scale_y = (2 * SQUARE_SIZE) / window_height
 * [solution for height is analogous]
 *
 * Scale_x and scale_y are coincidentally also the width and height of a square in clip space. These two values are not the same
 * because clip space goes from a constant -1 to 1, but the screen is a dynamic width and height. So if the width is greater than the
 * height than the width of a square in clip space will be less than it's height in clip space.
 *
 * Because of this we add (-1 * scale_x / 2) and  (-1 * scale_y / 2) to the x and y transformations respectively. This will shift the
 * entire grid by half a square to the left and half a square down. This ensures that the center square is presented in the middle of
 * the screen.
 */
fn init_projection_matrix_buffer(
    device: &wgpu::Device,
    size: &WindowSize,
    player: &Position,
    usage: wgpu::BufferUsages, // COPY_DST if it's the main reference, COPY_SRC for update sources
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Main projection matrix buffer"),
        contents: bytemuck::cast_slice(&calculate_projection_matrix(size, player)),
        usage: wgpu::BufferUsages::UNIFORM | usage,
    })
}

fn calculate_projection_matrix(size: &WindowSize, player: &Position) -> [[f32; 4]; 4] {
    let scale_x = (2.0 * SQUARE_SIZE) / (size.width as f32);
    let scale_y = (2.0 * SQUARE_SIZE) / (size.height as f32);
    let transform_x = scale_x / -2.0 - (scale_x * player.x as f32);
    let transform_y = scale_y / -2.0 - (scale_y * player.y as f32);
    [
        [scale_x, 0.0, 0.0, 0.0],
        [0.0, scale_y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [transform_x, transform_y, 0.0, 1.0],
    ]
}

fn init_projection_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("Projection matrix bind group layout"),
    })
}

fn init_projection_bind_group(
    device: &Device,
    projection_buffer: &wgpu::Buffer,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: projection_buffer.as_entire_binding(),
        }],
        label: Some("Project matrix bind group"),
    })
}
