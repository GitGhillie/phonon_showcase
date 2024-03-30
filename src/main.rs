use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fmod::prelude::AudioSource;
use bevy_fmod::prelude::*;
use bevy_fmod_phonon::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_perf_ui::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            FmodPlugin {
                audio_banks_paths: &[
                    "./assets/audio/demo_project/Build/Desktop/Master.bank",
                    "./assets/audio/demo_project/Build/Desktop/Master.strings.bank",
                    "./assets/audio/demo_project/Build/Desktop/Music.bank",
                ],
                plugin_paths: Some(&["./phonon_fmod.dll"]),
            },
            PhononPlugin,
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FpsControllerPlugin)
        .add_plugins(HookPlugin)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, (setup_scene, setup_player))
        .add_systems(PostStartup, play_music)
        .run();
}

fn setup_player(mut commands: Commands) {
    let logical_entity = commands
        .spawn((
            Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            ActiveEvents::COLLISION_EVENTS,
            Velocity::zero(),
            RigidBody::Dynamic,
            Sleeping::disabled(),
            LockedAxes::ROTATION_LOCKED,
            AdditionalMassProperties::Mass(1.0),
            GravityScale(0.0),
            Ccd { enabled: true }, // Prevent clipping when going fast
            TransformBundle::from_transform(Transform::from_xyz(0.0, 3.0, 0.0)),
            LogicalPlayer,
            FpsControllerInput {
                pitch: -TAU / 12.0,
                yaw: TAU * 5.0 / 8.0,
                ..default()
            },
            FpsController { ..default() },
        ))
        .insert(CameraConfig {
            height_offset: 0.0,
            radius_scale: 0.75,
        })
        .id();

    commands
        .spawn((
            Camera3dBundle {
                projection: Projection::from(PerspectiveProjection {
                    fov: 80.0 * TAU / 360.0,
                    ..default()
                }),
                ..default()
            },
            RenderPlayer { logical_entity },
        ))
        .insert(SpatialListenerBundle::default());
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    studio: Res<FmodStudio>,
) {
    commands.spawn(PerfUiCompleteBundle::default());

    // Audio sources
    let event_description = studio.0.get_event("event:/Music/Radio Station").unwrap();

    commands
        .spawn(SpatialAudioBundle::new(event_description))
        .insert(PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 1.5, 20.0).with_scale(Vec3::splat(0.25)),
            ..default()
        });

    // Load blockout
    commands.spawn((
        Name::from("Blockout"),
        HookedSceneBundle {
            scene: SceneBundle {
                scene: asset_server.load("level/blockout.glb#Scene0"),
                ..default()
            },
            hook: SceneHook::new(|entity, cmds| {
                cmds.insert(NeedsAudioMesh::default());
            }),
        },
        AsyncSceneCollider::default(),
    ));

    // Load detail
    commands.spawn((
        Name::from("Detail"),
        SceneBundle {
            scene: asset_server.load("level/detail.glb#Scene0"),
            ..default()
        },
    ));
}

fn play_music(mut audio_sources: Query<&AudioSource>) {
    for audio_source in audio_sources.iter_mut() {
        audio_source.play();
    }
}
