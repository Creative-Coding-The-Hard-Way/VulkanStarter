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

pub struct Application {
    debug_callback: Option<DebugCallback>,
    instance: Arc<Instance>,
    event_loop: Option<EventLoop<()>>,
    window: Window,
}

impl Application {
    pub fn initialize() -> Result<Self, Box<dyn Error>> {
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
        Self::pick_physical_device(&instance);

        Ok(Self {
            debug_callback,
            instance,
            event_loop: Option::Some(event_loop),
            window,
        })
    }

    fn pick_physical_device(instance: &Arc<Instance>) {
        let names: Vec<String> = PhysicalDevice::enumerate(&instance)
            .map(|device| device.name().to_owned())
            .collect();
        log::info!("available devices\n  {}", names.join("\n  "));
    }

    fn check_debug_layers() -> Result<bool, Box<dyn Error>> {
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

    fn create_instance() -> Result<Arc<Instance>, Box<dyn Error>> {
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
