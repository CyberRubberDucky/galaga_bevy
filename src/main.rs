// --------> Imports <---------
use bevy::{
    color::palettes::basic::*,
    input::{touch::TouchPhase, keyboard::KeyboardInput},
    log::{Level, LogPlugin},
    prelude::*,
    window::{WindowMode, MonitorSelection},
    winit::WinitSettings,
};

// --------> Variables <---------

// Add constants, static variables, or global settings here if needed
// Example:
// const SCREEN_WIDTH: f32 = 800.0;

// --------> Structs <---------

#[derive(Debug, Clone, PartialEq)]
enum EntityType {
    Player,
    Fly,
    Bullet,
}

#[derive(Component)]
struct GameEntity {
    id: u32,
    position: Vec3,
    entity_type: EntityType,
}

#[derive(Resource)]
struct PlayerPosition(Vec3);

#[derive(Resource)]
struct BulletSpeed(f32);

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct OutlineContainer {
    width: f32,
    height: f32,
}

// --------> Functions <---------

/// Handles player input (keyboard events)
fn handle_player_input(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut player_position: ResMut<PlayerPosition>,
    mut query: Query<(&mut Transform, &GameEntity), With<GameEntity>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let move_delta = 10.0; // --------> Player movement speed <---------
    let mut move_offset = Vec3::ZERO;
    let mut shoot = false;

    // --------> Collect Input Events <---------
    for event in keyboard_input_events.read() {
        if let key_code = event.key_code {
            match key_code {
                KeyCode::ArrowLeft => {
                    move_offset += Vec3::new(-move_delta, 0.0, 0.0); // Move left
                }
                KeyCode::ArrowRight => {
                    move_offset += Vec3::new(move_delta, 0.0, 0.0); // Move right
                }
                KeyCode::Space => {
                    shoot = true;
                }
                _ => {}
            }
        }
    }

    // --------> Apply Movement Logic <---------
    if move_offset != Vec3::ZERO {
        player_position.0 += move_offset;

        // Update the player's transform
        for (mut transform, game_entity) in query.iter_mut() {
            if game_entity.entity_type == EntityType::Player {
                transform.translation += move_offset;
                println!("Player moved to position: {:?}", transform.translation);
            }
        }
    }

    // --------> Shooting Logic <---------
    if shoot {
        println!("Player shoots!");

        shoot_bullet(&mut commands, &player_position);

        // Load and play the shooting sound
        let shoot_sound = asset_server.load("sounds/shooting.ogg");
        commands.spawn(AudioPlayer::new(shoot_sound));
    }
}

/// Shoots a bullet from the player's position
fn shoot_bullet(commands: &mut Commands, player_position: &ResMut<PlayerPosition>) {
    let bullet_starting_position = player_position.0 + Vec3::new(0.0, 50.0, 0.0); // --------> Start bullet above player <---------

    // Spawn a bullet entity
    commands.spawn((
        Bullet,
        SpriteBundle {
            transform: Transform {
                translation: bullet_starting_position,
                scale: Vec3::splat(10.0), // --------> Make the bullet small <---------
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::WHITE, // --------> Bullet color <---------
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Moves bullets and despawns them if they exit the screen
fn move_bullets(
    mut bullet_query: Query<(&mut Transform, Entity), With<Bullet>>,
    bullet_speed: Res<BulletSpeed>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta_time = time.delta().as_secs_f32();

    for (mut transform, bullet_entity) in bullet_query.iter_mut() {
        transform.translation.y += bullet_speed.0 * delta_time;

        // --------> Despawn bullets if they go off-screen <---------
        if transform.translation.y > 800.0 {
            commands.entity(bullet_entity).despawn();
        }
    }
}

/// Sets up the initial game scene (camera, player, boundary, etc.)
fn setup_scene(mut commands: Commands) {
    // --------> Spawn Camera <---------
    commands.spawn(Camera2d);

    // --------> Spawn Outline Container (Boundary) <---------
    let container_width = 1200.0;
    let container_height = 800.0;
    spawn_outline_container(&mut commands, Vec3::new(0.0, 100.0, 0.0), container_width, container_height);

    // --------> Spawn Game Entities (Player and Fly) <---------
    add_game_entity(&mut commands, Vec3::new(0.0, -250.0, 0.0), EntityType::Player); // Place player lower
    add_game_entity(&mut commands, Vec3::new(-300.0, 100.0, 0.0), EntityType::Fly);
}

/// Spawns the visible boundary container
fn spawn_outline_container(commands: &mut Commands, position: Vec3, width: f32, height: f32) {
    commands.spawn((
        OutlineContainer { width, height },
        SpriteBundle {
            transform: Transform {
                translation: position,
                scale: Vec3::new(width, height, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.2), // --------> Semi-transparent container <---------
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Adds a game entity (Player, Fly, etc.) at the given position
fn add_game_entity(commands: &mut Commands, position: Vec3, entity_type: EntityType) {
    let id = match entity_type {
        EntityType::Player => 1,
        EntityType::Fly => 2,
        EntityType::Bullet => 3,
    };

    commands.spawn((
        GameEntity {
            id,
            position,
            entity_type,
        },
        SpriteBundle {
            transform: Transform {
                translation: position,
                scale: Vec3::splat(50.0), // --------> Scale for each entity <---------
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Plays background music
fn music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioPlayer::new(
        asset_server.load("sounds/galaga.ogg"),
    ));
}

/// Attempts to update the player's entity movement with screen boundary checks
fn move_player_entity(
    player_position: &mut ResMut<PlayerPosition>,
    game_entity_query: &mut Query<(Entity, &mut Transform, &GameEntity), Without<Camera>>,
    commands: &mut Commands,
    offset: Vec3,
) {
    let container_width = 1200.0;
    let container_height = 800.0;

    let entity_width = 20.0;
    let entity_height = 20.0;

    for (entity, _, game_entity) in game_entity_query.iter_mut() {
        if let EntityType::Player = game_entity.entity_type {
            commands.entity(entity).despawn();

            let new_position = player_position.0 + offset;

            // Boundary checks
            let left_edge = new_position.x - entity_width / 2.0;
            let right_edge = new_position.x + entity_width / 2.0;

            let top_edge = new_position.y + entity_height / 2.0;
            let bottom_edge = new_position.y - entity_height / 2.0;

            let clamped_left = left_edge.clamp(-container_width / 2.0, container_width / 2.0);
            let clamped_right = right_edge.clamp(-container_width / 2.0, container_width / 2.0);
            let clamped_top = top_edge.clamp(-container_height / 2.0, container_height / 2.0);
            let clamped_bottom = bottom_edge.clamp(-container_height / 2.0, container_height / 2.0);

            let clamped_position = Vec3::new(
                (clamped_left + clamped_right) / 2.0,
                (clamped_bottom + clamped_top) / 2.0,
                new_position.z,
            );

            player_position.0 = clamped_position;

            println!("Player Entity moved to clamped position: {:?}", clamped_position);

            add_game_entity(commands, clamped_position, EntityType::Player);
            break;
        }
    }
}

// --------> Main Function <---------
fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: Level::DEBUG,
                filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    recognize_rotation_gesture: true,
                    ..default()
                }),
                ..default()
            }),
    )
        // --------> Insert Resources <---------
        .insert_resource(WinitSettings::mobile())
        .insert_resource(PlayerPosition(Vec3::new(0.0, -250.0, 0.0))) // Player's start position
        .insert_resource(BulletSpeed(300.0)) // How fast the bullet moves up
        // --------> Add Systems <---------
        .add_systems(Startup, (setup_scene, music))
        .add_systems(Update, (handle_player_input, move_bullets))
        .run();
}

