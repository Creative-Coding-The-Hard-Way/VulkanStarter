use std::error::Error;
use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::Surface;
use winit::window::Window;

mod queue_family_indices;

use queue_family_indices::QueueFamilyIndices;

type DynResult<T> = Result<T, Box<dyn Error>>;

/// Create a logical device and command queues
pub fn create_logical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
) -> DynResult<(Arc<Device>, Arc<Queue>, Arc<Queue>)> {
    let physical_device = pick_physical_device(surface, instance)?;

    let indices = QueueFamilyIndices::find(surface, &physical_device)?;
    let unique_indices = indices.unique_indices();

    let families = unique_indices
        .iter()
        .map(|index| physical_device.queue_families().nth(*index).unwrap())
        .map(|family| (family, 1.0f32));

    let (device, queues) = Device::new(
        physical_device,
        &Features::none(),
        &DeviceExtensions::none(),
        families,
    )?;

    let (graphics_queue, present_queue) = indices.take_queues(queues)?;

    Ok((device, graphics_queue, present_queue))
}

/// Take the first suitable physical device
fn pick_physical_device<'a>(
    surface: &Arc<Surface<Window>>,
    instance: &'a Arc<Instance>,
) -> Result<PhysicalDevice<'a>, String> {
    let devices: Vec<PhysicalDevice> =
        PhysicalDevice::enumerate(&instance).collect();

    let names: Vec<String> = devices
        .iter()
        .map(|properties| properties.name().to_owned())
        .collect();
    log::info!("available devices\n  {}", names.join("\n  "));

    devices
        .iter()
        .find(|device| is_device_suitable(&surface, &device))
        .cloned()
        .ok_or("unable to find a physical device".to_owned())
}

/// Find a device which suits the application's needs
fn is_device_suitable(
    surface: &Arc<Surface<Window>>,
    device: &PhysicalDevice,
) -> bool {
    match QueueFamilyIndices::find(surface, device) {
        Ok(_) => true,
        Err(error) => {
            log::warn!(
                "{:?} is not suitable because - {:?}",
                device.name(),
                error
            );
            false
        }
    }
}
