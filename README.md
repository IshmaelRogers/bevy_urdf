# bevy_urdf

[![Crates.io](https://img.shields.io/crates/v/bevy_urdf.svg)](https://crates.io/crates/bevy_urdf)
[![Crates.io](https://img.shields.io/crates/d/bevy_urdf.svg)](https://crates.io/crates/bevy_urdf)

A Bevy plugin for importing robots from URDF files and running physics simulations. Ground vehicles use Rapier physics, while drones are simulated using integrated dynamics models.

## Features

- Import URDF robot descriptions into Bevy
- Support for STL and OBJ mesh formats
- Rapier physics integration for ground vehicles
- Custom dynamics simulation for drone vehicles
- Event-driven sensor reading and motor control
- Multiple robot type support (ground vehicles, quadrupeds, drones)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_urdf = "0.3.0"  # Replace with actual version
```

## Quick Start

### 1. Add Plugins

Add the necessary plugins to your Bevy app:

```rust
use bevy::prelude::*;
use bevy_urdf::{UrdfPlugin, StlPlugin, ObjPlugin, CameraControlPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            UrdfPlugin,
            StlPlugin,
            ObjPlugin,
            CameraControlPlugin,
        ))
        .run();
}
```

### 2. Load Robot

Load a robot in your startup systems:

```rust
use bevy_urdf::{LoadRobot, RobotLoaded};

fn setup(mut load_robot_events: EventWriter<LoadRobot>) {
    load_robot_events.send(LoadRobot {
        urdf_path: "robots/flamingo_edu/urdf/Edu_v4.urdf".to_string(),
        mesh_dir: "assets/robots/flamingo_edu/urdf".to_string(),
    });
}
```

### 3. Spawn Robot

Subscribe to the loaded event and spawn the robot:

```rust
use bevy_urdf::{RobotLoaded, SpawnRobot, RobotType};

fn start_simulation(
    mut robot_loaded_events: EventReader<RobotLoaded>,
    mut spawn_robot_events: EventWriter<SpawnRobot>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for event in robot_loaded_events.read() {
        spawn_robot_events.send(SpawnRobot {
            handle: event.handle.clone(),
            mesh_dir: event.mesh_dir.clone(),
            robot_type: RobotType::NotDrone,
        });
        next_state.set(AppState::Simulation);
    }
}
```

## API Reference

### Events

The plugin uses an event-driven architecture for robot control and sensor feedback:

#### Sensor Reading

```rust
#[derive(Event)]
pub struct SensorsRead {
    pub handle: Handle<UrdfAsset>,
    pub transforms: Vec<Transform>,
    pub joint_angles: Vec<f32>,
}
```

#### Motor Control (Ground Robots)

For ground-based robots and vehicles:

```rust
#[derive(Event)]
pub struct ControlMotors {
    pub handle: Handle<UrdfAsset>,
    pub velocities: Vec<f32>,
}
```

#### Thrust Control (Drones)

For aerial vehicles and drones:

```rust
#[derive(Event)]
pub struct ControlThrusts {
    pub handle: Handle<UrdfAsset>,
    pub thrusts: Vec<f32>,
}
```

#### Thruster Control (UUVs)

For underwater vehicles:

```rust
#[derive(Event)]
pub struct ControlThrusters {
    pub handle: Handle<UrdfAsset>,
    pub thrusts: Vec<f32>,
}
```

#### Camera Rotation

Control the viewing angle of any active camera:

```rust
#[derive(Event)]
pub struct RotateCamera {
    pub delta_yaw: f32,
    pub delta_pitch: f32,
}
```

### Robot Types

- `RobotType::NotDrone` - Ground vehicles, quadrupeds, manipulators
- `RobotType::Drone` - Aerial vehicles with thrust-based control
- `RobotType::Uuv` - Underwater vehicles with thruster control

## Examples

Run the included examples to see the plugin in action:

```bash
# Drone simulation
cargo run --example quadrotor --release

# Quadruped robot simulation
cargo run --example quadruped --release

# Underwater vehicle simulation with map terrain
cargo run --example uuv --release
```

When using `MapTerrainPlugin` (for example in the `uuv` example), the Bevy `png`
feature must be enabled so that PNG tile images can be loaded correctly.

### Bathymetry Tiles and Starting Location

`MapTerrainPlugin` works with any map tile service following the `{z}/{x}/{y}`
pattern. To visualize underwater terrain, supply a bathymetry tile source and
set the reference coordinates. The snippet below centers the map near
`41.61906° N, 71.20932° W` using public GEBCO tiles:

```rust
MapTerrainPlugin::new(MapConfig {
    reference_lat: 41.61906,
    reference_lon: -71.20932,
    zoom_levels: vec![(16, 1), (15, 2)],
    tile_source_url: "https://tiles.gebco.net/tiles/{z}/{x}/{y}.png".to_string(),
    heightmap_source_url: None,
    height_scale: 1.0,
    cache_dir: "assets/tiles".to_string(),
    z_layer: 0.0,
});
```

Replace the URL with your preferred bathymetry tile server. See the
[GEBCO gridded bathymetry data](https://www.gebco.net/data-products/gridded-bathymetry-data)
for details.

## URDF Requirements

Before using URDF files with this plugin:

1. **Mesh Paths**: Ensure all mesh file references use relative paths
2. **Unsupported Elements**: Remove or replace `package://` URIs
3. **Gazebo Elements**: Gazebo-specific nodes are not supported and should be removed

Example of supported mesh reference:
```xml
<!-- Good: relative path -->
<mesh filename="meshes/base_link.stl"/>

<!-- Not supported: package URI -->
<mesh filename="package://robot_description/meshes/base_link.stl"/>
```

## Known Limitations

- Package URIs (`package://`) are not supported
- Gazebo-specific URDF elements are ignored
- Drone rotor animations are not yet implemented
- Limited to STL and OBJ mesh formats

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
