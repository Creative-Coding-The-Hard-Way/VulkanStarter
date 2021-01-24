use log;
use std::error::Error;
use std::sync::Arc;
use vulkano::instance::{
    ApplicationInfo, Instance, InstanceExtensions, Version,
};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct Application {
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

        Ok(Self {
            instance,
            event_loop: Option::Some(event_loop),
            window,
        })
    }

    fn create_instance() -> Result<Arc<Instance>, Box<dyn Error>> {
        let supported_extensions = InstanceExtensions::supported_by_core()?;
        let required_extensions = vulkano_win::required_extensions();
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
