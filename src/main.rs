use bevy::{
    color::palettes::basic::*,
    input::{gestures::RotationGesture, touch::TouchPhase},
    log::{Level, LogPlugin},
    prelude::*,
    window::{AppLifecycle, WindowMode},
    winit::WinitSettings,
};

#[derive(Debug, Clone, PartialEq)]
enum EntityType {
    Player,
    Fly,
    Bullet,
}

#[derive(Component)]
struct ButtonComponent {
    text: String,
}

#[derive(Component)]
struct GameEntity {
    id: u32,
    position: Vec3,
    entity_type: EntityType,
}

#[derive(Resource)]
struct PlayerPosition(Vec3);

#[derive(Component)]
struct OutlineContainer {
    width: f32,
    height: f32,
}

fn main() {
    let mut app = App::new();
    // -------------------------------------------
    // Setup Plugins
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
        // -------------------------------------------
        // Insert Resources
        .insert_resource(WinitSettings::mobile())
        .insert_resource(PlayerPosition(Vec3::new(0.0, 0.0, 0.0)))
        // -------------------------------------------
        // Add Systems
        .add_systems(Startup, (setup_scene, music))
        .add_systems(Update, button_handler)
        .run();
}

fn setup_scene(mut commands: Commands) {
    // -------------------------------------------
    // Spawn Camera
    commands.spawn(Camera2d);

    // -------------------------------------------
    // Spawn UI Buttons
    let button_width = 100.0;
    let spacing = 20.0;
    let screen_width = 1500.0;

    let total_width = button_width * 3.0 + spacing * 2.0;
    let start_position = (screen_width - total_width) / 2.0;

    spawn_button(&mut commands, "Left".to_string(), Vec2::new(start_position, 50.0));
    spawn_button(
        &mut commands,
        "Shoot".to_string(),
        Vec2::new(start_position + button_width + spacing, 50.0),
    );
    spawn_button(
        &mut commands,
        "Right".to_string(),
        Vec2::new(start_position + 2.0 * (button_width + spacing), 50.0),
    );

    // -------------------------------------------
    // Spawn Outline Container (Boundary)
    let container_width = 1200.0;
    let container_height = 800.0;
    spawn_outline_container(&mut commands, Vec3::new(0.0, 100.0, 0.0), container_width, container_height);

    // -------------------------------------------
    // Spawn Game Entities
    add_game_entity(&mut commands, Vec3::new(0.0, 0.0, 0.0), EntityType::Player);
    add_game_entity(&mut commands, Vec3::new(-300.0, 100.0, 0.0), EntityType::Fly);
    add_game_entity(&mut commands, Vec3::new(300.0, -100.0, 0.0), EntityType::Bullet);
}

fn spawn_button(commands: &mut Commands, text: String, position: Vec2) {
    // -------------------------------------------
    // Spawn UI Button with Text
    commands
        .spawn((
            ButtonComponent { text: text.clone() },
            Button,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: Val::Px(100.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                bottom: Val::Px(position.y),
                ..default()
            },
        ))
        .with_child((
            Text::new(text),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor::BLACK,
            TextLayout::new_with_justify(JustifyText::Center),
        ));
}

fn spawn_outline_container(commands: &mut Commands, position: Vec3, width: f32, height: f32) {
    // -------------------------------------------
    // Spawn a rectangular boundary container
    commands.spawn((
        OutlineContainer { width, height },
        SpriteBundle {
            transform: Transform {
                translation: position,
                scale: Vec3::new(width, height, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.2), // Semi-transparent for the container
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut player_position: ResMut<PlayerPosition>,
    mut game_entity_query: Query<(Entity, &mut Transform, &GameEntity), Without<Camera>>,
    mut commands: Commands,
) {
    // -------------------------------------------
    // Handle Button Presses and Movements
    for (interaction, mut color, button_component) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BLUE.into();
                println!("{} was pressed!", button_component.text);

                if button_component.text == "Left" {
                    move_player_entity(
                        &mut player_position,
                        &mut game_entity_query,
                        &mut commands,
                        Vec3::new(-30.0, 0.0, 0.0),
                    );
                } else if button_component.text == "Right" {
                    move_player_entity(
                        &mut player_position,
                        &mut game_entity_query,
                        &mut commands,
                        Vec3::new(30.0, 0.0, 0.0),
                    );
                }
            }
            Interaction::Hovered => {
                *color = GRAY.into();
            }
            Interaction::None => {
                *color = WHITE.into();
            }
        }
    }
}

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

fn music(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioPlayer::new(
        asset_server.load("sound/galaga.ogg"),
    ));
}

fn add_game_entity(commands: &mut Commands, position: Vec3, entity_type: EntityType) {
    // -------------------------------------------
    // Add Game Entities (Player, Fly, Bullet)
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
                scale: Vec3::splat(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn remove_game_entity(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}
