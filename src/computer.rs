use crate::rules::Rule;

pub struct ComputerFactory {
    shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) size_buffer: wgpu::Buffer,
    pub(crate) rule_buffer: wgpu::Buffer,
}

impl ComputerFactory {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("game-of-life.compute.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("compute_bind_group_layout"),
        });

        let size_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("size_buffer"),
            size: (2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rule_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rule_buffer"),
            size: (2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            shader,
            bind_group_layout,
            size_buffer,
            rule_buffer,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        device: &wgpu::Device,
        cells_width: u32,
        cells_height: u32,
        rule: &Rule,
        seed: u32,
        initial_density: u8,
        queue: &wgpu::Queue,
    ) -> Computer {
        use rand::prelude::{Rng, SeedableRng};
        use wgpu::util::DeviceExt;

        let size_array = [cells_width, cells_height];
        queue.write_buffer(&self.size_buffer, 0, bytemuck::cast_slice(&size_array));

        queue.write_buffer(
            &self.rule_buffer,
            0,
            bytemuck::cast_slice(&rule.rule_array()),
        );

        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(u64::from(seed));
        let mut cells_vec = vec![0_u32; cells_width as usize * cells_height as usize];
        let initial_density = f32::from(initial_density) * 0.01;
        for cell in cells_vec.iter_mut() {
            if rng.gen::<f32>() < initial_density {
                *cell = 1;
            }
        }

        let cells_buffer_usages = {
            #[cfg(not(test))]
            {
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX
            }
            #[cfg(test)]
            {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_SRC
            }
        };

        let cells_buffer_0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cells_vec),
            usage: cells_buffer_usages,
        });

        let cells_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (cells_vec.len() * std::mem::size_of::<u32>()) as u64,
            usage: cells_buffer_usages,
            mapped_at_creation: false,
        });

        let create_bind_group = |from_buffer, to_buffer, bind_group_name| {
            device.create_bind_group({
                &wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: from_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: to_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.size_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.rule_buffer,
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                    label: Some(bind_group_name),
                }
            })
        };

        let compute_bind_group_from_0_to_1 =
            create_bind_group(&cells_buffer_0, &cells_buffer_1, "compute_bind_group_0");
        let compute_bind_group_from_1_to_0 =
            create_bind_group(&cells_buffer_1, &cells_buffer_0, "compute_bind_group_1");

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&self.bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &self.shader,
            entry_point: "main",
            constants: &Default::default(),
        });

        Computer {
            cells_width,
            cells_height,
            compute_pipeline,
            currently_computed_is_0: true,
            compute_bind_group_from_0_to_1,
            compute_bind_group_from_1_to_0,
            cells_buffer_0,
            cells_buffer_1,
        }
    }
}

pub struct Computer {
    cells_width: u32,
    cells_height: u32,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_from_0_to_1: wgpu::BindGroup,
    compute_bind_group_from_1_to_0: wgpu::BindGroup,
    pub currently_computed_is_0: bool,
    pub cells_buffer_0: wgpu::Buffer,
    pub cells_buffer_1: wgpu::Buffer,
}

impl Computer {
    pub fn enqueue(&mut self, command_encoder: &mut wgpu::CommandEncoder) {
        let mut pass_encoder =
            command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass_encoder.set_pipeline(&self.compute_pipeline);

        pass_encoder.set_bind_group(
            0,
            if self.currently_computed_is_0 {
                &self.compute_bind_group_from_0_to_1
            } else {
                &self.compute_bind_group_from_1_to_0
            },
            &[],
        );

        self.currently_computed_is_0 = !self.currently_computed_is_0;

        let workgroup_width = 8;
        assert_eq!(self.cells_width % workgroup_width, 0);
        assert_eq!(self.cells_height % workgroup_width, 0);
        let workgroup_count_x = (self.cells_width + workgroup_width - 1) / workgroup_width;
        let workgroup_count_y = (self.cells_height + workgroup_width - 1) / workgroup_width;
        let workgroup_count_z = 1;
        pass_encoder.dispatch_workgroups(workgroup_count_x, workgroup_count_y, workgroup_count_z);
    }
}

