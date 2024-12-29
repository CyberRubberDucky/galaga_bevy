use bevy::{
    input::keyboard::KeyboardInput,
    prelude::*,
};
use crate::{ColorsPalette, PlayerPosition, EntityType, GameEntity};

/// Handles player input (keyboard events)
pub fn handle_player_input(
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
        crate::Bullet,
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
