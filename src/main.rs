// --------> Imports <---------
use bevy::{
    input::{keyboard::KeyboardInput, touch::TouchPhase},
    log::{Level, LogPlugin},
    prelude::*,
    window::{MonitorSelection, WindowMode},
    winit::WinitSettings,
};

// --------> Color Palette <---------
#[derive(Resource)]
struct ColorsPalette {
    player_color: Color,
    fly_color: Color,
    bullet_color: Color,
    background_color: Color,
}

// Initialize the palette
fn create_color_palette() -> ColorsPalette {
    ColorsPalette {
        player_color: Color::rgb(0.2, 0.6, 1.0),   // Custom Blue
        fly_color: Color::rgb(1.0, 0.0, 0.0),      // Custom Red
        bullet_color: Color::rgb(0.0, 1.0, 0.0),   // Custom Green
        background_color: Color::rgb(0.0, 0.0, 0.2), // Dark Background
    }
}

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
    color_palette: Res<ColorsPalette>, // Use the palette here
) {
    let move_delta = 10.0; // --------> Player movement speed <---------
    let mut move_offset = Vec3::ZERO;
    let mut shoot = false;

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

    if move_offset != Vec3::ZERO {
        player_position.0 += move_offset;

        for (mut transform, game_entity) in query.iter_mut() {
            if game_entity.entity_type == EntityType::Player {
                transform.translation += move_offset;
                println!("Player moved to position: {:?}", transform.translation);
            }
        }
    }

    if shoot {
        println!("Player shoots!");
        shoot_bullet(&mut commands, &player_position, &color_palette);
        let shoot_sound = asset_server.load("sounds/shooting.ogg");
        commands.spawn(AudioPlayer::new(shoot_sound));
    }
}

/// Shoots a bullet from the player's position
fn shoot_bullet(
    commands: &mut Commands,
    player_position: &ResMut<PlayerPosition>,
    color_palette: &ColorsPalette,
) {
    let bullet_starting_position = player_position.0 + Vec3::new(0.0, 50.0, 0.0);

    commands.spawn((
        Bullet,
        SpriteBundle {
            transform: Transform {
                translation: bullet_starting_position,
                scale: Vec3::splat(10.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: color_palette.bullet_color,
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

        if transform.translation.y > 800.0 {
            commands.entity(bullet_entity).despawn();
        }
    }
}

/// Detects collisions between bullets and other entities (like Fly or Player).
/// Removes the bullet and the target (Fly or Player) if a collision is detected.
/// Detects collisions between bullets and other entities and removes collided bullets and targets.
fn collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    target_query: Query<(Entity, &Transform, &GameEntity)>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (target_entity, target_transform, target) in target_query.iter() {
            // Check collision between bullet and target
            if is_colliding(&bullet_transform.translation, &target_transform.translation, 25.0) {
                println!(
                    "Collision detected! Bullet at {:?} hit {:?} at {:?}",
                    bullet_transform.translation, target.entity_type, target_transform.translation
                );

                // Ensure bullet and target entities are marked for removal
                commands.entity(bullet_entity).despawn(); // Remove the bullet
                commands.entity(target_entity).despawn(); // Remove the target

                println!("Removed bullet and target: {:?}", target.entity_type);

                // Once a collision is handled, break to avoid processing this bullet further
                break;
            }
        }
    }
}

/// Helper function to determine whether two entities are colliding.
/// `radius` defines the collision circle radius for simplicity.
fn is_colliding(pos1: &Vec3, pos2: &Vec3, radius: f32) -> bool {
    pos1.distance(*pos2) < radius
}

/// Despawns entities that leave the boundaries of the container
fn despawn_out_of_bounds_entities(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&OutlineContainer>)>,
) {
    for (entity, transform, outline_container) in query.iter() {
        // Define container boundaries (adjust these if container dimensions change)
        let container_width = 1200.0 / 2.0; // Half-width since position is relative to the center
        let container_height = 800.0 / 2.0; // Half-height since position is relative to the center

        // Optional check for a specific container entity
        if outline_container.is_some() {
            continue; // Skip the container itself
        }

        // Check if entity is outside the container boundaries
        let pos = transform.translation;
        if pos.x < -container_width
            || pos.x > container_width
            || pos.y < -container_height
            || pos.y > container_height
        {
            println!("Despawning entity outside bounds at position: {:?}", pos);
            commands.entity(entity).despawn();
        }
    }
}

/// Sets up the initial game scene (camera, player, boundary, etc.)
fn setup_scene(mut commands: Commands, color_palette: Res<ColorsPalette>) {
    commands.spawn(Camera2d);

    let container_width = 1200.0;
    let container_height = 800.0;

    spawn_outline_container(
        &mut commands,
        Vec3::new(0.0, 0.0, 0.0), // Positioned at origin (updated earlier)
        container_width,
        container_height,
    );

    add_game_entity(
        &mut commands,
        Vec3::new(0.0, -250.0, 0.0),
        EntityType::Player,
        &color_palette,
    );
    add_game_entity(
        &mut commands,
        Vec3::new(-300.0, 100.0, 0.0),
        EntityType::Fly,
        &color_palette,
    );
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
                color: Color::rgba(0.0, 0.0, 0.0, 0.2), // Semi-transparent container
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Adds a game entity (Player, Fly, etc.) at the given position
fn add_game_entity(
    commands: &mut Commands,
    position: Vec3,
    entity_type: EntityType,
    color_palette: &ColorsPalette,
) {
    let id = match entity_type {
        EntityType::Player => 1,
        EntityType::Fly => 2,
        EntityType::Bullet => 3,
    };

    let color = match entity_type {
        EntityType::Player => color_palette.player_color,
        EntityType::Fly => color_palette.fly_color,
        _ => Color::WHITE,
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
                scale: Vec3::splat(50.0),
                ..Default::default()
            },
            sprite: Sprite {
                color,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Plays background music
fn music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioPlayer::new(asset_server.load("sounds/galaga.ogg")));
}

/// Main function
fn main() {
    let mut app = App::new();
    let color_palette = create_color_palette();

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
        .insert_resource(WinitSettings::mobile())
        .insert_resource(PlayerPosition(Vec3::new(0.0, -250.0, 0.0)))
        .insert_resource(BulletSpeed(300.0))
        .insert_resource(color_palette) // Add palette to resources
        .add_systems(Startup, (setup_scene, music))
        .add_systems(Update, (handle_player_input, move_bullets, collision, despawn_out_of_bounds_entities)) // Added despawn system
        .run();
}