use super::*;
use gfx;

gfx_defines! {
    pipeline pipe {
        u_stereo: gfx::Global<i32> = "u_stereo",
        u_resolution_in: gfx::Global<[f32; 2]> = "u_resolution_in",
        u_resolution_out: gfx::Global<[f32; 2]> = "u_resolution_out",
        s_source: gfx::TextureSampler<[f32; 4]> = "s_color",
        s_deflection: gfx::TextureSampler<[f32; 4]> = "s_deflection",
        rt_color: gfx::RenderTarget<ColorFormat> = "rt_color",
        u_vis_type: gfx::Global<i32> = "u_vis_type",
        u_heat_scale: gfx::Global<f32> = "u_heat_scale",
    }
}

pub struct Display {
    pso: gfx::PipelineState<Resources, pipe::Meta>,
    pso_data: pipe::Data<Resources>,
}

impl Node for Display {
    fn new(window: &Window) -> Self {
        let mut factory = window.factory().borrow_mut();

        let pso = factory
            .create_pipeline_simple(
                &include_glsl!("mod.vert"),
                &include_glsl!("mod.frag"),
                pipe::new(),
            )
            .unwrap();

        let sampler = factory.create_sampler_linear();
        let (_, src, dst) = factory.create_render_target(1, 1).unwrap();

        // add deflection view
        let (_, srv, _): (
            _,
            _,
            gfx::handle::RenderTargetView<gfx_device_gl::Resources, [f32; 4]>,
        ) = factory.create_render_target(1, 1).unwrap();

        Display {
            pso,
            pso_data: pipe::Data {
                u_stereo: 0,
                u_resolution_in: [1.0, 1.0],
                u_resolution_out: [1.0, 1.0],
                s_source: (src, sampler.clone()),
                s_deflection: (srv, sampler.clone()),
                rt_color: dst,
                u_vis_type: 0,
                u_heat_scale: 1.0,
            },
        }
    }

    fn negociate_slots(&mut self, window: &Window, slots: NodeSlots) -> NodeSlots {
        let slots = slots.to_color_input(window).to_color_output(window);
        self.pso_data.u_resolution_in = slots.input_size_f32();
        self.pso_data.u_resolution_out = slots.output_size_f32();
        self.pso_data.s_source = slots.as_color_view();
        self.pso_data.s_deflection = slots.as_deflection_view();
        self.pso_data.rt_color = slots.as_color();
        slots
    }

    fn update_values(&mut self, _window: &Window, values: &ValueMap) {
        self.pso_data.u_stereo = if values
            .get("split_screen_switch")
            .unwrap_or(&Value::Bool(false))
            .as_bool()
            .unwrap_or(false)
        {
            1
        } else {
            0
        }
    }

    fn input(&mut self, _head: &Head, gaze: &Gaze, vis_param: &VisualizationParameters) -> Gaze {
        let ratio = [
            self.pso_data.u_resolution_out[0] / self.pso_data.u_resolution_in[0],
            self.pso_data.u_resolution_out[1] / self.pso_data.u_resolution_in[1],
        ];
        let offset = [
            0.5 * (ratio[0] - ratio[1]).max(0.0),
            0.5 * (ratio[1] - ratio[0]).max(0.0),
        ];
        let scale = [
            ratio[0] / ratio[0].min(ratio[1]),
            ratio[1] / ratio[0].min(ratio[1]),
        ];

        self.pso_data.u_vis_type = ((vis_param.vis_type) as u32) as i32;
        self.pso_data.u_heat_scale = vis_param.heat_scale;

        Gaze {
            x: scale[0] * gaze.x - offset[0],
            y: scale[1] * gaze.y - offset[1],
        }
    }

    fn render(&mut self, window: &Window) {
        let mut encoder = window.encoder().borrow_mut();

        if self.pso_data.u_stereo == 0 {
            encoder.draw(&gfx::Slice::from_vertex_count(6), &self.pso, &self.pso_data);
        } else {
            encoder.draw(
                &gfx::Slice::from_vertex_count(12),
                &self.pso,
                &self.pso_data,
            );
        }
    }
}
