mod player;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_fmod::prelude::AudioSource;
use bevy_fmod::prelude::*;
use bevy_fmod_phonon::prelude::*;
use bevy_rapier3d::prelude::AsyncSceneCollider;

use bevy_scene_hook::{HookPlugin, HookedSceneBundle, SceneHook};

use crate::player::PlayerPlugin;
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
                    "./assets/audio/demo_project/Build/Desktop/SFX.bank",
                ],
                plugin_paths: Some(&["./phonon_fmod.dll"]),
            },
            PhononPlugin,
        ))
        .add_plugins(PlayerPlugin)
        .add_plugins(HookPlugin)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup_scene)
        .add_systems(PostStartup, play_music)
        .add_systems(Update, enable_shadows)
        .run();
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
            hook: SceneHook::new(|_entity, cmds| {
                cmds.insert(NeedsAudioMesh::default());
            }),
        },
        AsyncSceneCollider::default(),
    ));

    // Load detail
    commands.spawn((
        Name::from("Detail"),
        HookedSceneBundle {
            scene: SceneBundle {
                scene: asset_server.load("level/detail.glb#Scene0"),
                ..default()
            },
            hook: SceneHook::new(|entity, cmds| {
                if let Some(name) = entity.get::<Name>() {
                    if name.as_str().contains("Collider") {
                        cmds.insert(NeedsAudioMesh::default())
                            .insert(Visibility::Hidden);
                    }
                }
            }),
        },
    ));
}

fn play_music(mut audio_sources: Query<&AudioSource>) {
    for audio_source in audio_sources.iter_mut() {
        audio_source.play();
    }
}

fn enable_shadows(mut added_lights: Query<&mut DirectionalLight, Added<DirectionalLight>>) {
    for mut dir_light in &mut added_lights {
        dir_light.shadows_enabled = true;
    }
}
