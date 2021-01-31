use std::error::Error;
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::impl_vertex;
use vulkano::pipeline::{
    viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract,
};

type DynResult<T> = Result<T, Box<dyn Error>>;
type DynRenderPass = dyn RenderPassAbstract + Send + Sync;

#[derive(Default, Debug, Copy, Clone)]
pub struct Vertex {
    pub inPosition: [f32; 2],
    pub inColor: [f32; 4],
}

impl_vertex!(Vertex, inPosition, inColor);

impl Vertex {
    pub fn new(pos: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            inPosition: pos,
            inColor: color,
        }
    }
}

pub fn create_vertex_buffer(
    device: &Arc<Device>,
) -> Arc<dyn BufferAccess + Send + Sync> {
    CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        [
            Vertex::new([0.0, -0.5], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([0.5, 0.5], [1.0, 1.0, 1.0, 1.0]),
            Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0, 1.0]),
        ],
    )
    .expect("unable to create a vertex buffer")
}

pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    swapchain_extent: [u32; 2],
    render_pass: &Arc<DynRenderPass>,
) -> DynResult<Arc<dyn GraphicsPipelineAbstract + Send + Sync>> {
    mod vertex_shader {
        //
        vulkano_shaders::shader! {
            ty: "vertex",
            src: r#"
            #version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) in vec2 inPosition;
            layout(location = 1) in vec4 inColor;

            layout(location = 0) out vec4 vertColor;

            void main() {
                vertColor = inColor;
                gl_Position = vec4(inPosition, 0.0, 1.0);
            }
            "#
        }
    }

    mod fragment_shader {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: r#"
            #version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) in vec4 fragColor;
            layout(location = 0) out vec4 outColor;

            void main() {
               outColor = fragColor;
            }
            "#
        }
    }

    let vert = vertex_shader::Shader::load(device.clone())?;
    let frag = fragment_shader::Shader::load(device.clone())?;

    let dimensions = [swapchain_extent[0] as f32, swapchain_extent[1] as f32];
    let viewport = Viewport {
        dimensions,
        origin: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vert.main_entry_point(), ())
            .fragment_shader(frag.main_entry_point(), ())
            .viewports(vec![viewport])
            .depth_clamp(false)
            .polygon_mode_fill()
            .line_width(1.0)
            .cull_mode_disabled()
            .front_face_clockwise()
            .depth_write(false)
            .sample_shading_disabled()
            .blend_pass_through()
            .render_pass(
                Subpass::from(render_pass.clone(), 0)
                    .ok_or("could not create renderpass subpass!")?,
            )
            .build(device.clone())?,
    );

    Ok(pipeline)
}
