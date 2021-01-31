use std::error::Error;
use std::sync::Arc;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState,
};
use vulkano::device::{Device, Queue};
use vulkano::format::ClearValue;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::debug::DebugCallback;
use vulkano::instance::Instance;
use vulkano::pipeline::vertex::BufferlessVertices;
use vulkano::swapchain::{acquire_next_image, Surface, Swapchain};
use vulkano::sync::{GpuFuture, SharingMode};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod device;
mod instance;
mod swapchain;
mod triangle_pipeline;

use triangle_pipeline::GraphicsPipelineComplete;

type DynResult<T> = Result<T, Box<dyn Error>>;

pub struct Application {
    // vulkan library resources
    _instance: Arc<Instance>,
    _debug_callback: Option<DebugCallback>,

    // window/surface resources
    surface: Arc<Surface<Window>>,
    event_loop: Option<EventLoop<()>>,
    pipeline: Arc<GraphicsPipelineComplete>,
    _render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    swapchain: Arc<Swapchain<Window>>,
    _swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    framebuffer_images: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    // devices and queues
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,

    // command buffers
    command_buffers: Vec<Arc<AutoCommandBuffer>>,
}

impl Application {
    pub fn initialize() -> DynResult<Self> {
        let instance = instance::create_instance()?;
        let debug_callback = instance::setup_debug_callback(&instance);

        let event_loop: EventLoop<()> = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_title("vulkan experiments")
            .with_resizable(false)
            .with_decorations(true)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(1366, 768))
            .build_vk_surface(&event_loop, instance.clone())?;

        let physical_device =
            device::pick_physical_device(&surface, &instance)?;
        let (device, graphics_queue, present_queue) =
            device::create_logical_device(&surface, &physical_device)?;
        let (swapchain, swapchain_images) = swapchain::create_swap_chain(
            &surface,
            &physical_device,
            &device,
            &graphics_queue,
            &present_queue,
        )?;

        let render_pass =
            triangle_pipeline::create_render_pass(&device, swapchain.format())?;

        let pipeline = triangle_pipeline::create_graphics_pipeline(
            &device,
            swapchain.dimensions(),
            &render_pass,
        )?;

        let framebuffer_images =
            swapchain::create_framebuffers(&swapchain_images, &render_pass);

        let mut app = Self {
            // library resources
            _instance: instance,
            _debug_callback: debug_callback,

            // window/surface resources
            surface,
            event_loop: Option::Some(event_loop),
            pipeline,
            _render_pass: render_pass,
            swapchain,
            _swapchain_images: swapchain_images,
            framebuffer_images,

            // devices and queues
            device,
            graphics_queue,
            present_queue,

            // command buffers
            command_buffers: vec![],
        };

        app.build_command_buffers();

        Ok(app)
    }

    fn build_command_buffers(&mut self) {
        let family = self.graphics_queue.family();
        // TODO: add an actual command to this example
        self.command_buffers = self
            .framebuffer_images
            .iter()
            .map(|framebuffer_image| {
                let vertices = BufferlessVertices {
                    vertices: 3,
                    instances: 1,
                };
                let mut builder =
                    AutoCommandBufferBuilder::primary_simultaneous_use(
                        self.device.clone(),
                        family,
                    )
                    .unwrap();

                builder
                    .begin_render_pass(
                        framebuffer_image.clone(),
                        vulkano::command_buffer::SubpassContents::Inline,
                        vec![ClearValue::Float([0.0, 0.0, 0.0, 1.0])],
                    )
                    .unwrap()
                    .draw(
                        self.pipeline.clone(),
                        &DynamicState::none(),
                        vertices,
                        (),
                        (),
                    )
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                Arc::new(builder.build().unwrap())
            })
            .collect();
    }

    /**
     * Render the screen.
     */
    fn render(&self) {
        let (image_index, _suboptimal, acquire_future) =
            acquire_next_image(self.swapchain.clone(), None).unwrap();

        let command_buffer = self.command_buffers[image_index].clone();

        let future = acquire_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.present_queue.clone(),
                self.swapchain.clone(),
                image_index,
            )
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        self.surface.window().request_redraw();
    }

    /**
     * Main application loop for this window. Blocks the thread until the
     * window is closed.
     */
    pub fn main_loop(mut self) {
        let event_loop = self.event_loop.take().unwrap();

        // render once before showing the window so it's not garbage
        self.render();
        self.surface.window().set_visible(true);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::MainEventsCleared => {
                    // redraw here
                    self.render();
                }

                _ => (),
            }
        });
    }
}
