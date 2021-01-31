use std::error::Error;
use std::sync::Arc;
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::{
    vertex::BufferlessDefinition, viewport::Viewport, GraphicsPipeline,
};

type DynResult<T> = Result<T, Box<dyn Error>>;
type DynRenderPass = dyn RenderPassAbstract + Send + Sync;

// concrete type is required because we're using the BufferlessDefinition
pub type GraphicsPipelineComplete = GraphicsPipeline<
    BufferlessDefinition,
    Box<dyn PipelineLayoutAbstract + Send + Sync>,
    Arc<dyn RenderPassAbstract + Send + Sync>,
>;

pub fn create_graphics_pipeline(
    device: &Arc<Device>,
    swapchain_extent: [u32; 2],
    render_pass: &Arc<DynRenderPass>,
) -> DynResult<Arc<GraphicsPipelineComplete>> {
    mod vertex_shader {
        //
        vulkano_shaders::shader! {
            ty: "vertex",
            src: r#"
            #version 450
            #extension GL_ARB_separate_shader_objects : enable

            layout(location = 0) out vec4 vertColor;

            vec2 positions[] = vec2[] (
                vec2(0.0, -0.5),
                vec2(0.5, 0.5),
                vec2(-0.5, 0.5)
            );

            vec3 colors[] = vec3[] (
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0)
            );

            void main() {
                vertColor = vec4(colors[gl_VertexIndex], 1.0);
                gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
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
            .vertex_input(BufferlessDefinition {})
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
