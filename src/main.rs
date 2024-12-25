use bevy::{
    color::palettes::basic::*,
    input::{gestures::RotationGesture, touch::TouchPhase},
    log::{Level, LogPlugin},
    prelude::*,
    window::{AppLifecycle, WindowMode},
    winit::WinitSettings,
};

#[derive(Debug, Clone, PartialEq)]
enum EntityBlockType {
    Player,
    Fly,
    Bullet,
}

#[derive(Component)]
struct ButtonComponent {
    text: String,
}

#[derive(Component)]
struct EntityBlock {
    id: u32,
    position: Vec3,
    entity_type: EntityBlockType,
}

#[derive(Resource)]
struct EntityBlockPosition(Vec3);

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
        .insert_resource(WinitSettings::mobile())
        .insert_resource(EntityBlockPosition(Vec3::new(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (button_handler, handle_lifetime))
        .add_systems(Update, entity_block)
        .run();
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d);
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

    spawn_entity_block(&mut commands, Vec3::new(0.0, 0.0, 0.0), EntityBlockType::Player);

    spawn_entity_block(&mut commands, Vec3::new(-300.0, 100.0, 0.0), EntityBlockType::Fly);
    spawn_entity_block(&mut commands, Vec3::new(300.0, -100.0, 0.0), EntityBlockType::Bullet);
}

fn spawn_button(commands: &mut Commands, text: String, position: Vec2) {
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

fn button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut entity_block_position: ResMut<EntityBlockPosition>,
    mut entity_query: Query<(Entity, &mut Transform), With<EntityBlock>>,
    mut commands: Commands,
) {
    for (interaction, mut color, button_component) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BLUE.into();
                println!("{} was pressed!", button_component.text);

                if button_component.text == "Left" {
                    move_entity_block_left(
                        &mut entity_block_position,
                        &mut entity_query,
                        &mut commands,
                    );
                } else if button_component.text == "Right" {
                    move_entity_block_right(
                        &mut entity_block_position,
                        &mut entity_query,
                        &mut commands,
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

fn move_entity_block_left(
    entity_block_position: &mut ResMut<EntityBlockPosition>,
    entity_query: &mut Query<(Entity, &mut Transform), With<EntityBlock>>,
    commands: &mut Commands,
) {
    for (entity, _) in entity_query.iter() {
        commands.entity(entity).despawn();
    }

    let new_position = entity_block_position.0 - Vec3::new(30.0, 0.0, 0.0);
    entity_block_position.0 = new_position;

    println!("EntityBlock moved left to position: {:?}", new_position);

    spawn_entity_block(commands, new_position, EntityBlockType::Player);
}

fn move_entity_block_right(
    entity_block_position: &mut ResMut<EntityBlockPosition>,
    entity_query: &mut Query<(Entity, &mut Transform), With<EntityBlock>>,
    commands: &mut Commands,
) {
    for (entity, _) in entity_query.iter() {
        commands.entity(entity).despawn();
    }

    let new_position = entity_block_position.0 + Vec3::new(30.0, 0.0, 0.0);
    entity_block_position.0 = new_position;

    println!("EntityBlock moved right to position: {:?}", new_position);

    spawn_entity_block(commands, new_position, EntityBlockType::Player);
}

fn spawn_entity_block(commands: &mut Commands, position: Vec3, entity_type: EntityBlockType) {
    let id = match entity_type {
        EntityBlockType::Player => 1,
        EntityBlockType::Fly => 2,
        EntityBlockType::Bullet => 3,
    };

    commands.spawn((
        EntityBlock {
            id,
            position,
            entity_type,
        },
        Mesh2d(Handle::default()),
        Transform {
            translation: position,
            scale: Vec3::splat(50.),
            ..Default::default()
        },
    ));
}

fn entity_block(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    entity_block_position: Res<EntityBlockPosition>,
) {
    let position = entity_block_position.0;

    commands.spawn((
        EntityBlock {
            id: 1,
            position,
            entity_type: EntityBlockType::Player,
        },
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform {
            translation: position,
            scale: Vec3::splat(50.),
            ..Default::default()
        },
    ));

    println!("Spawned EntityBlock at position: {:?}", position);
}

fn handle_lifetime(mut lifecycle_events: EventReader<AppLifecycle>) {
    for event in lifecycle_events.read() {
        match event {
            AppLifecycle::Idle | AppLifecycle::WillSuspend | AppLifecycle::WillResume => {}
            _ => {}
        }
    }
}
