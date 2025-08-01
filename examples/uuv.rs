use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::{
    color::palettes::css::WHITE, input::common_conditions::input_toggle_active, prelude::*,
};
use bevy_flycam::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_stl::StlPlugin;
use bevy_urdf::events::{ControlThrusters, LoadRobot, RapierOption, RobotLoaded, SpawnRobot};
use bevy_urdf::plugin::{RobotType, UrdfPlugin};
use bevy_urdf::urdf_asset_loader::UrdfAsset;
use bevy_urdf::{CameraControlPlugin, RotateCamera};
use bevy_urdf::{MapConfig, MapTerrainPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            StlPlugin,
            ObjPlugin,
            FlyCameraPlugin {
                spawn_camera: true,
                grab_cursor_on_startup: true,
            },
            RapierPhysicsPlugin::<NoUserData>::default(),
            UrdfPlugin::default().with_default_system_setup(true),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            MapTerrainPlugin::new(MapConfig {
                // Reference point for Newport Harbor (approx.)
                reference_lat: 41.61906,
                reference_lon: -71.20932,
                // Highest zoom first; radius in tiles around the UUV
                zoom_levels: vec![(16, 1), (15, 2)],
                tile_source_url: "https://tiles.gebco.net/tiles/{z}/{x}/{y}.png".to_string(),
                heightmap_source_url: None,
                height_scale: 1.0,
                cache_dir: "assets/tiles".to_string(),
                z_layer: 0.0,
            }),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
            CameraControlPlugin,
        ))
        .init_state::<AppState>()
        .insert_resource(MovementSettings {
            move_speed: Vec3::ONE * 3.0,
        })
        .insert_resource(MouseSettings {
            invert_horizontal: false,
            invert_vertical: false,
            mouse_sensitivity: 0.00012,
            lock_cursor_to_middle: false,
        })
        .insert_resource(ClearColor(Color::linear_rgb(1.0, 1.0, 1.0)))
        .insert_resource(UrdfRobotHandle(None))
        .add_systems(Startup, setup)
        .add_systems(Update, control_thrusters)
        .add_systems(Update, camera_angle_input)
        .add_systems(Update, start_simulation.run_if(in_state(AppState::Loading)))
        .run();
}

#[derive(Resource)]
struct UrdfRobotHandle(Option<Handle<UrdfAsset>>);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Simulation,
}

fn start_simulation(
    mut commands: Commands,
    mut er_robot_loaded: EventReader<RobotLoaded>,
    mut ew_spawn_robot: EventWriter<SpawnRobot>,
    mut state: ResMut<NextState<AppState>>,
) {
    for event in er_robot_loaded.read() {
        ew_spawn_robot.write(SpawnRobot {
            handle: event.handle.clone(),
            mesh_dir: event.mesh_dir.clone(),
            parent_entity: None,
            robot_type: RobotType::Uuv,
            drone_descriptor: None,
            uuv_descriptor: event.uuv_descriptor.clone(),
        });
        state.set(AppState::Simulation);
        commands.insert_resource(UrdfRobotHandle(Some(event.handle.clone())));
    }
}

fn control_thrusters(
    robot_handle: Res<UrdfRobotHandle>,
    mut ew_control_thrusters: EventWriter<ControlThrusters>,
) {
    if let Some(handle) = robot_handle.0.clone() {
        ew_control_thrusters.write(ControlThrusters {
            handle,
            thrusts: vec![1.0, 1.0],
        });
    }
}

#[allow(deprecated)]
fn setup(mut commands: Commands, mut ew_load_robot: EventWriter<LoadRobot>) {
    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 300.0,
        ..default()
    });

    ew_load_robot.send(LoadRobot {
        robot_type: RobotType::Uuv,
        urdf_path: "uuvs/simple_uuv.urdf".to_string(),
        mesh_dir: "assets/uuvs/".to_string(),
        rapier_options: RapierOption {
            interaction_groups: None,
            translation_shift: Some(Vec3::new(0.0, 1.0, 0.0)),
            create_colliders_from_visual_shapes: false,
            create_colliders_from_collision_shapes: true,
        },
        marker: None,
        drone_descriptor: None,
        uuv_descriptor: None,
    });
}

fn camera_angle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ew_rotate: EventWriter<RotateCamera>,
) {
    let mut delta_yaw = 0.0;
    let mut delta_pitch = 0.0;
    let step = 0.05;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        delta_yaw += step;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        delta_yaw -= step;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        delta_pitch += step;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        delta_pitch -= step;
    }
    if delta_yaw != 0.0 || delta_pitch != 0.0 {
        ew_rotate.send(RotateCamera {
            delta_yaw,
            delta_pitch,
        });
    }
}
