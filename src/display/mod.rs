use std::error::Error;
use std::sync::Arc;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::image::swapchain::SwapchainImage;
use vulkano::instance::debug::DebugCallback;
use vulkano::instance::Instance;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

mod device;
mod instance;
mod swapchain;

type DynResult<T> = Result<T, Box<dyn Error>>;

pub struct Display {
    // vulkan library resources
    pub instance: Arc<Instance>,
    pub debug_callback: Option<DebugCallback>,

    // window/surface resources
    pub surface: Arc<Surface<Window>>,
    pub event_loop: Option<EventLoop<()>>,
    pub render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    pub framebuffer_images: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    // devices and queues
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
    pub present_queue: Arc<Queue>,
}

impl Display {
    pub fn create() -> DynResult<Self> {
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
            swapchain::create_render_pass(&device, swapchain.format())?;

        let framebuffer_images =
            swapchain::create_framebuffers(&swapchain_images, &render_pass);

        Ok(Display {
            // library resources
            instance,
            debug_callback,

            // window/surface resources
            surface,
            event_loop: Option::Some(event_loop),
            render_pass,
            swapchain,
            swapchain_images,
            framebuffer_images,

            // devices and queues
            device,
            graphics_queue,
            present_queue,
        })
    }
}
