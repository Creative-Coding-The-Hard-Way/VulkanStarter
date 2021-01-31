use std::error::Error;
use std::sync::Arc;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::debug::DebugCallback;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
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
    _pipeline: Arc<GraphicsPipelineComplete>,
    _render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    _swapchain: Arc<Swapchain<Window>>,
    _swapchain_images: Vec<Arc<SwapchainImage<Window>>>,

    // devices and queues
    _device: Arc<Device>,
    _graphics_queue: Arc<Queue>,
    _present_queue: Arc<Queue>,
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
            .with_visible(true)
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

        Ok(Self {
            // library resources
            _instance: instance,
            _debug_callback: debug_callback,

            // window/surface resources
            surface,
            event_loop: Option::Some(event_loop),
            _pipeline: pipeline,
            _render_pass: render_pass,
            _swapchain: swapchain,
            _swapchain_images: swapchain_images,

            // devices and queues
            _device: device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
        })
    }

    /**
     * Render the screen.
     */
    fn render(&self) {
        self.surface.window().request_redraw();
    }

    /**
     * Main application loop for this window. Blocks the thread until the
     * window is closed.
     */
    pub fn main_loop(mut self) {
        let event_loop = self.event_loop.take().unwrap();

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
