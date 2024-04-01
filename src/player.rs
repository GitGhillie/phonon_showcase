use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_fmod::prelude::AudioSource;
use bevy_fmod::prelude::*;
use bevy_fps_controller::controller::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_rapier3d::prelude::*;
use std::f32::consts::TAU;
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(FpsControllerPlugin)
            .init_state::<WalkingState>()
            .add_systems(Startup, setup_player)
            .add_systems(Update, change_walking_state)
            .add_systems(OnEnter(WalkingState::Walking), play_walking_sound)
            .add_systems(
                Update,
                play_walking_sound
                    .run_if(in_state(WalkingState::Walking))
                    .run_if(on_timer(Duration::from_millis(470))),
            )
            .add_systems(
                Update,
                play_walking_sound
                    .run_if(in_state(WalkingState::CrouchedWalking))
                    .run_if(on_timer(Duration::from_millis(700))),
            );
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum WalkingState {
    #[default]
    NotWalking,
    Walking,
    CrouchedWalking,
}

fn play_walking_sound(mut audio_sources: Query<&AudioSource>) {
    for audio_source in audio_sources.iter_mut() {
        println!("Playing walking sounds");
        audio_source.play();
    }
}

fn change_walking_state(
    mut next_state: ResMut<NextState<WalkingState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let walking_keys = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD];

    if keys.any_pressed(walking_keys) {
        if keys.pressed(KeyCode::ControlLeft) {
            next_state.set(WalkingState::CrouchedWalking);
        } else {
            next_state.set(WalkingState::Walking);
        }
    } else {
        next_state.set(WalkingState::NotWalking)
    }
}

fn setup_player(
    mut commands: Commands,
    studio: Res<FmodStudio>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
        .insert(Name::from("Player"))
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

    // Footstep sounds
    let event_description = studio
        .0
        .get_event("event:/Character/Player Footsteps")
        .unwrap();
    let child = commands
        .spawn(SpatialAudioBundle::new(event_description))
        .insert(PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.2, 0.2)),
            transform: Transform::from_xyz(0.0, 0.2, 0.0).with_scale(Vec3::splat(0.25)),
            ..default()
        })
        .id();

    commands.entity(logical_entity).add_child(child);
}
