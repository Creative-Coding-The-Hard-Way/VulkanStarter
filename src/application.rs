use std::error::Error;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct Application {
    event_loop: Option<EventLoop<()>>,
    window: Window,
}

impl Application {
    pub fn initialize() -> Result<Application, Box<dyn Error>> {
        let event_loop: EventLoop<()> = EventLoop::new();
        let window: Window = WindowBuilder::new()
            .with_title("vulkan experiments")
            .with_resizable(false)
            .with_decorations(true)
            .with_visible(true)
            .with_inner_size(LogicalSize::new(1366, 768))
            .build(&event_loop)?;

        Ok(Application {
            event_loop: Option::Some(event_loop),
            window,
        })
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
