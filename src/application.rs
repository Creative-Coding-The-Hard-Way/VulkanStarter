use log;
use std::error::Error;
use std::sync::Arc;
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::{
    layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice,
    Version,
};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];
const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);

type DynResult<T> = Result<T, Box<dyn Error>>;

struct QueueFamilyIndices {
    graphics_family: Option<i32>,
}

impl QueueFamilyIndices {
    fn new() -> Self {
        Self {
            graphics_family: None,
        }
    }

    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

pub struct Application {
    _debug_callback: Option<DebugCallback>,
    instance: Arc<Instance>,
    event_loop: Option<EventLoop<()>>,
    window: Window,
    physical_device_index: usize,
}

impl Application {
    pub fn initialize() -> DynResult<Self> {
        let event_loop: EventLoop<()> = EventLoop::new();
        let window: Window = WindowBuilder::new()
            .with_title("vulkan experiments")
            .with_resizable(false)
            .with_decorations(true)
            .with_visible(true)
            .with_inner_size(LogicalSize::new(1366, 768))
            .build(&event_loop)?;
        let instance = Self::create_instance()?;
        let debug_callback = Self::setup_debug_callback(&instance);
        let physical_device_index = Self::pick_physical_device(&instance)?;

        Ok(Self {
            _debug_callback: debug_callback,
            instance,
            event_loop: Option::Some(event_loop),
            window,
            physical_device_index,
        })
    }

    fn pick_physical_device(instance: &Arc<Instance>) -> Result<usize, String> {
        let devices: Vec<PhysicalDevice> =
            PhysicalDevice::enumerate(&instance).collect();
        let names: Vec<String> = devices
            .iter()
            .map(|properties| properties.name().to_owned())
            .collect();
        log::info!("available devices\n  {}", names.join("\n  "));

        devices
            .iter()
            .position(|device| Self::is_device_suitable(&device))
            .ok_or("unable to find a physical device".to_owned())
    }

    /// Find a device which suits the application's needs
    fn is_device_suitable(device: &PhysicalDevice) -> bool {
        let indices: QueueFamilyIndices = Self::find_queue_families(device);
        indices.is_complete()
    }

    /// Find a device which has support for a graphics command queue
    fn find_queue_families(device: &PhysicalDevice) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices::new();

        for (i, family) in device.queue_families().enumerate() {
            if family.supports_graphics() {
                indices.graphics_family = Some(i as i32);
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn check_debug_layers() -> DynResult<bool> {
        let available_layers: Vec<String> = layers_list()?
            .map(|layer| layer.name().to_owned())
            .collect();

        log::info!("available debug layers \n{}", available_layers.join("\n"));

        let all_available = VALIDATION_LAYERS.iter().all(|required_layer| {
            available_layers.contains(&required_layer.to_string())
        });
        Ok(all_available)
    }

    fn required_extensions() -> InstanceExtensions {
        let mut required_extensions = vulkano_win::required_extensions();
        if ENABLE_VALIDATION_LAYERS {
            required_extensions.ext_debug_utils = true;
        }
        required_extensions
    }

    fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }

        let severity = MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true,
        };

        let msgtype = MessageType {
            general: true,
            performance: true,
            validation: true,
        };

        DebugCallback::new(instance, severity, msgtype, |msg| {
            match msg.severity {
                MessageSeverity { error: true, .. } => {
                    log::error!("{:?}", msg.description)
                }
                MessageSeverity { warning: true, .. } => {
                    log::warn!("{:?}", msg.description)
                }
                MessageSeverity {
                    information: true, ..
                } => {
                    log::info!("{:?}", msg.description);
                }
                MessageSeverity { verbose: true, .. } => {
                    log::debug!("{:?}", msg.description);
                }
                _ => log::debug!("{:?}", msg.description),
            }
        })
        .ok()
    }

    fn create_instance() -> DynResult<Arc<Instance>> {
        if ENABLE_VALIDATION_LAYERS && !Self::check_debug_layers()? {
            log::warn!(
                "validation layers requested, but they were not all avialable!"
            )
        }

        let supported_extensions = InstanceExtensions::supported_by_core()?;
        let required_extensions = Self::required_extensions();
        log::info!(
            "supported extensions \n {}",
            format!("{:?}", supported_extensions)
                .as_str()
                .replace(",", "\n")
                .replace("[", "")
                .replace("]", "")
        );
        log::info!(
            "required extensions \n {}",
            format!("{:?}", required_extensions)
                .as_str()
                .replace(",", "\n")
                .replace("[", "")
                .replace("]", "")
        );

        let app_info = ApplicationInfo {
            application_name: Some("Vulkan Experiments".into()),
            application_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            engine_name: Some("no engine".into()),
            engine_version: None,
        };

        Ok(Instance::new(Some(&app_info), &required_extensions, None)?)
    }

    /**
     * Render the screen.
     */
    fn render(&self) {
        self.window.request_redraw();
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
