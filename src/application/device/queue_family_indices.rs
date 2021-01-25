use std::error::Error;
use std::sync::Arc;
use vulkano::device::{Queue, QueuesIter};
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::Surface;
use winit::window::Window;

type DynResult<T> = Result<T, Box<dyn Error>>;

pub struct QueueFamilyIndices {
    graphics_family: usize,
    present_family: usize,
}

impl QueueFamilyIndices {
    /// Find the queue family indices for the given device
    pub fn find(
        surface: &Arc<Surface<Window>>,
        device: &PhysicalDevice,
    ) -> DynResult<Self> {
        let mut graphics = None;
        let mut present = None;

        for (i, family) in device.queue_families().enumerate() {
            if family.supports_graphics() {
                graphics = Some(i);
            }

            if surface.is_supported(family)? {
                present = Some(i);
            }
            if graphics.is_some() && present.is_some() {
                break;
            }
        }

        let indices = graphics
            .zip(present)
            .map(|(graphics_family, present_family)| Self {
                graphics_family,
                present_family,
            })
            .ok_or("unable to find all required queue families for device")?;

        Ok(indices)
    }

    /// Return the set of unique queue family indices
    pub fn unique_indices(&self) -> Vec<usize> {
        if self.is_same_queue() {
            vec![self.graphics_family]
        } else {
            vec![self.graphics_family, self.present_family]
        }
    }

    /// get the graphics and present queues based on the index order returned
    /// by unique_indices
    pub fn take_queues(
        &self,
        mut queues: QueuesIter,
    ) -> DynResult<(Arc<Queue>, Arc<Queue>)> {
        let graphics_queue =
            queues.next().ok_or("no graphics queue available")?;

        let present_queue = if self.is_same_queue() {
            graphics_queue.clone()
        } else {
            queues.next().ok_or("no present queue available")?
        };

        Ok((graphics_queue, present_queue))
    }

    fn is_same_queue(&self) -> bool {
        self.graphics_family == self.present_family
    }
}
