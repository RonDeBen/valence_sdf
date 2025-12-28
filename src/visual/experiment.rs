use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

use crate::visual::sdf::digits::{Digit, TransitionSpec};

pub struct ExperimentMaterialPlugin;

impl Plugin for ExperimentMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExperimentMaterial>::default())
            .init_resource::<DigitTimer>()
            .add_systems(Startup, setup_experiment_scene)
            .add_systems(Update, update_experiment_shader);
    }
}

/// Single flow instruction for the shader
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct ShaderFlow {
    pub from_seg: u32,
    pub to_seg: u32,
    pub share: f32,
    pub _padding: u32, // Align to 16 bytes
}

const MAX_FLOWS: usize = 32; // Maximum number of flows (should be enough for any digit transition)

/// Data passed to the experimental shader (uniform buffer with fixed-size flow array)
#[derive(ShaderType, Debug, Clone)]
pub struct ExperimentData {
    pub time: f32,
    pub from_digit: u32,
    pub to_digit: u32,
    pub transition_progress: f32,

    pub from_digit_mask: u32, // Bitmask of active segments in from_digit
    pub to_digit_mask: u32,   // Bitmask of active segments in to_digit
    pub flow_count: u32,      // How many flows are active
    pub _padding: u32,        // Align to 16 bytes
    
    pub flows: [ShaderFlow; MAX_FLOWS], // Fixed-size array of flows
}

impl Default for ExperimentData {
    fn default() -> Self {
        Self {
            time: 0.0,
            from_digit: 0,
            to_digit: 0,
            transition_progress: 0.0,
            from_digit_mask: 0,
            to_digit_mask: 0,
            flow_count: 0,
            _padding: 0,
            flows: [ShaderFlow {
                from_seg: 0,
                to_seg: 0,
                share: 0.0,
                _padding: 0,
            }; MAX_FLOWS],
        }
    }
}

/// Material for experimental shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct ExperimentMaterial {
    #[uniform(0)]
    pub data: ExperimentData,
}

impl Material for ExperimentMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/experiment.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Resource to store the handle to the experiment material
#[derive(Resource)]
pub struct ExperimentMaterialHandle(pub Handle<ExperimentMaterial>);

/// Timer for cycling through digits 0-9
#[derive(Resource)]
pub struct DigitTimer {
    pub current_digit: Digit,
    pub elapsed: f32,
    pub digit_duration: f32,
    pub transition_duration: f32,
}

impl Default for DigitTimer {
    fn default() -> Self {
        Self {
            current_digit: Digit::Zero,
            elapsed: 0.0,
            digit_duration: 2.0,
            transition_duration: 1.0,
        }
    }
}

impl DigitTimer {
    pub fn next_digit(&self) -> Digit {
        match self.current_digit {
            Digit::Zero => Digit::One,
            Digit::One => Digit::Two,
            Digit::Two => Digit::Three,
            Digit::Three => Digit::Four,
            Digit::Four => Digit::Five,
            Digit::Five => Digit::Six,
            Digit::Six => Digit::Seven,
            Digit::Seven => Digit::Eight,
            Digit::Eight => Digit::Nine,
            Digit::Nine => Digit::Zero,
        }
    }

    pub fn transition_progress(&self) -> Option<f32> {
        if self.elapsed < self.digit_duration {
            None
        } else {
            let transition_elapsed = self.elapsed - self.digit_duration;
            Some((transition_elapsed / self.transition_duration).min(1.0))
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed += delta;

        let total_cycle = self.digit_duration + self.transition_duration;
        if self.elapsed >= total_cycle {
            self.current_digit = self.next_digit();
            self.elapsed -= total_cycle;
        }
    }
}

fn setup_experiment_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExperimentMaterial>>,
) {
    info!("🧪 Setting up experimental scene...");

    let plane_size = 20.0;
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    let experiment_material = ExperimentMaterial::default();
    let material_handle = materials.add(experiment_material);
    commands.insert_resource(ExperimentMaterialHandle(material_handle.clone()));

    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    info!("✨ Experimental scene ready!");
}

fn update_experiment_shader(
    time: Res<Time>,
    mut digit_timer: ResMut<DigitTimer>,
    experiment_handle: Res<ExperimentMaterialHandle>,
    mut materials: ResMut<Assets<ExperimentMaterial>>,
) {
    digit_timer.tick(time.delta_secs());

    if let Some(material) = materials.get_mut(&experiment_handle.0) {
        let transition_progress = digit_timer.transition_progress().unwrap_or(0.0);

        let from_digit = digit_timer.current_digit;
        let to_digit = digit_timer.next_digit();

        // Compute the transition spec (flows) in Rust
        let transition_spec = TransitionSpec::compute_flows(from_digit, to_digit);

        // Convert flows to shader format and copy into fixed-size array
        let flow_count = transition_spec.flows.len().min(MAX_FLOWS);
        for (i, flow) in transition_spec.flows.iter().enumerate().take(MAX_FLOWS) {
            material.data.flows[i] = ShaderFlow {
                from_seg: flow.from as u32,
                to_seg: flow.to as u32,
                share: flow.share,
                _padding: 0,
            };
        }

        material.data.time = time.elapsed_secs();
        material.data.from_digit = from_digit as u32;
        material.data.to_digit = to_digit as u32;
        material.data.transition_progress = transition_progress;
        material.data.from_digit_mask = from_digit.mask() as u32;
        material.data.to_digit_mask = to_digit.mask() as u32;
        material.data.flow_count = flow_count as u32;
    }
}
