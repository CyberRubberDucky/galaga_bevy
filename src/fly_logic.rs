use bevy::prelude::*;
use crate::{ColorsPalette, EntityType, GameEntity};

/// Spawns a single fly at the given position
pub fn spawn_fly(
    commands: &mut Commands,
    position: Vec3,
    color_palette: &Res<ColorsPalette>,
) {
    commands.spawn((
        GameEntity {
            id: 2, // Unique ID for Fly
            position,
            entity_type: EntityType::Fly,
        },
        SpriteBundle {
            transform: Transform {
                translation: position,
                scale: Vec3::splat(50.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: color_palette.fly_color,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

/// Spawns multiple flies at predefined positions
pub fn spawn_three_flies(
    commands: &mut Commands,
    base_position: Vec3,
    color_palette: &Res<ColorsPalette>,
) {
    let offsets = vec![
        Vec3::new(0.0, 0.0, 0.0), // First fly at base_position
        Vec3::new(100.0, 50.0, 0.0), // Second fly slightly offset
        Vec3::new(-100.0, -50.0, 0.0), // Third fly slightly offset
    ];

    for offset in offsets {
        let position = base_position + offset;
        spawn_fly(commands, position, color_palette);
    }
}

/// System that manages fly spawning
pub fn fly_spawner_system(mut commands: Commands, color_palette: Res<ColorsPalette>) {
    spawn_three_flies(&mut commands, Vec3::new(0.0, 200.0, 0.0), &color_palette);
}
