use std::error::Error;
use std::sync::Arc;
use vulkano::buffer::BufferAccess;
use vulkano::command_buffer::{
    AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState,
};
use vulkano::format::ClearValue;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::swapchain::acquire_next_image;
use vulkano::sync::GpuFuture;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::display::Display;

mod triangle_pipeline;

type DynResult<T> = Result<T, Box<dyn Error>>;

pub struct Application {
    // vulkan display resources
    display: Display,

    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

    // command buffers
    command_buffers: Vec<Arc<AutoCommandBuffer>>,

    // vertex buffers
    triangle_vertices: Arc<dyn BufferAccess + Send + Sync>,
}

impl Application {
    pub fn initialize() -> DynResult<Self> {
        let display = Display::create()?;

        let pipeline = triangle_pipeline::create_graphics_pipeline(
            &display.device,
            display.swapchain.dimensions(),
            &display.render_pass,
        )?;

        let triangle_vertices =
            triangle_pipeline::create_vertex_buffer(&display.device);

        let mut app = Self {
            display,
            pipeline,
            command_buffers: vec![],
            triangle_vertices,
        };
        app.build_command_buffers();
        Ok(app)
    }

    fn build_command_buffers(&mut self) {
        let family = self.display.graphics_queue.family();
        self.command_buffers = self
            .display
            .framebuffer_images
            .iter()
            .map(|framebuffer_image| {
                let mut builder =
                    AutoCommandBufferBuilder::primary_simultaneous_use(
                        self.display.device.clone(),
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
                        vec![self.triangle_vertices.clone()],
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
    fn render(&mut self) {
        let (image_index, suboptimal, acquire_swapchain_future) =
            acquire_next_image(self.display.swapchain.clone(), None).unwrap();

        let command_buffer = self.command_buffers[image_index].clone();

        let future = acquire_swapchain_future
            .then_execute(self.display.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.display.present_queue.clone(),
                self.display.swapchain.clone(),
                image_index,
            )
            .then_signal_fence_and_flush()
            .unwrap();

        // wait for the frame to finish
        future.wait(None).unwrap();

        if suboptimal {
            self.rebuild_swapchain_resources();
        }
    }

    /// Rebuild the swapchain and command buffers
    fn rebuild_swapchain_resources(&mut self) {
        log::debug!("rebuilding swapchain resources");
        self.display.rebuild_swapchain();
        self.pipeline = triangle_pipeline::create_graphics_pipeline(
            &self.display.device,
            self.display.swapchain.dimensions(),
            &self.display.render_pass,
        )
        .expect("unable to rebuild the triangle pipeline");
        self.build_command_buffers();
    }

    /**
     * Main application loop for this window. Blocks the thread until the
     * window is closed.
     */
    pub fn main_loop(mut self) {
        let event_loop = self.display.event_loop.take().unwrap();

        // render once before showing the window so it's not garbage
        self.render();
        self.display.surface.window().set_visible(true);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.rebuild_swapchain_resources();
                }

                Event::MainEventsCleared => {
                    // redraw here
                    self.render();
                    self.display.surface.window().request_redraw();
                }

                _ => (),
            }
        });
    }
}
