use bevy::{
    asset::HandleId,
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        texture::ImageSampler,
    },
};
use rand::Rng;

// the black is the walls

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Generating,
    NotGenerating,
    Flooding,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: CONWAYS_SCREEN_SIZE.x,
                        height: CONWAYS_SCREEN_SIZE.y,
                        title: "To do".to_string(),
                        resizable: false,
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSampler::nearest_descriptor(),
                }),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Generating)
                .with_system(swap_colours)
        )
        .add_system_set(
            SystemSet::on_update(AppState::NotGenerating)
                .with_system(click_to_flood)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Flooding)
                .with_system(flooding)
        )
        .add_state(AppState::Generating)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .run();
}

const CONWAYS_MAP_SIZE: Vec2 = Vec2::new(160.0, 160.0);
const CONWAYS_SCREEN_SIZE: Vec2 = Vec2::new(1000.0, 1000.0);
// the colours (first one is the walls (black), second is open space, third is water) RGBA
const CAVE_COLOURS: [[u8; 4]; 3] = [[200, 120, 70, 0], [200, 120, 70, 255], [100, 100, 254, 255]];

#[derive(Resource)]
struct GameOfLifeImage(HandleId);

fn flooding (
    mut images: ResMut<Assets<Image>>,
    mut app_state: ResMut<State<AppState>>,
    id: Res<GameOfLifeImage>,
) {

    let handle = Handle::weak(id.0);

    if let Some(image) = images.get_mut(&handle) {

        // makes a vector of all of the blue pixels
        let old_image: Vec<u8> = image
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| (i + 2) % 4 == 0)
            .map(|x| *x.1)
            .collect();

        let mut flooding = false;

        for (i, _) in old_image.iter().enumerate() {

            // i need to make it so that diagonal neighbors don't count
            let num_neighbors = count_neighbors(&old_image, i.try_into().unwrap(), CAVE_COLOURS[2][2], false);

            if num_neighbors > 0 && image.data[(i * 4) + 3] != CAVE_COLOURS[0][3] && image.data[(i * 4) + 2] != CAVE_COLOURS[2][2] {
                image.data[(i * 4)] = CAVE_COLOURS[2][0];
                image.data[(i * 4) + 1] = CAVE_COLOURS[2][1];
                image.data[(i * 4) + 2] = CAVE_COLOURS[2][2];
                image.data[(i * 4) + 3] = CAVE_COLOURS[2][3];

                flooding = true;
            }
        }

        if !flooding {
            app_state.set(AppState::NotGenerating).unwrap();
        }
    }
}

fn click_to_flood (
    mut images: ResMut<Assets<Image>>,
    id: Res<GameOfLifeImage>,
    buttons: Res<Input<MouseButton>>,
    mut app_state: ResMut<State<AppState>>,
    windows: Res<Windows>,
) {
    if buttons.just_pressed(MouseButton::Left) {

        let handle = Handle::weak(id.0);

        if let Some(image) = images.get_mut(&handle) {
            let window = windows.get_primary().unwrap();

            if let Some(mut position) = window.cursor_position() {
                position.y = CONWAYS_SCREEN_SIZE.y - position.y;

                let x = ((position.x / CONWAYS_SCREEN_SIZE.x) * CONWAYS_MAP_SIZE.x).trunc();
                let y = ((position.y / CONWAYS_SCREEN_SIZE.y) * CONWAYS_MAP_SIZE.y).trunc();
                let pixel_clicked = (x + (y * (CONWAYS_MAP_SIZE.x))) as usize;

                if image.data[(pixel_clicked * 4) + 3] == CAVE_COLOURS[1][3] {
                    image.data[(pixel_clicked * 4)] = CAVE_COLOURS[2][0];
                    image.data[(pixel_clicked * 4) + 1] = CAVE_COLOURS[2][1];
                    image.data[(pixel_clicked * 4) + 2] = CAVE_COLOURS[2][2];
                    image.data[(pixel_clicked * 4) + 3] = CAVE_COLOURS[2][3];

                    app_state.set(AppState::Flooding).unwrap();
                }
            }
        }
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: (CONWAYS_MAP_SIZE.x) as u32,
            height: (CONWAYS_MAP_SIZE.y) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &CAVE_COLOURS[1],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    for (i, x) in image.data.iter_mut().enumerate() {
        if (i + 1) % 4 == CAVE_COLOURS[0][3] as usize {

            let val = rand::thread_rng().gen_range(0..=13) < 6;
            if val {
                *x = CAVE_COLOURS[1][3];
            } else {
                *x = CAVE_COLOURS[0][3];
            }
        }
    }

    let image = images.add(image.clone());
    let id = image.id();

    commands.insert_resource(GameOfLifeImage(id));

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(CONWAYS_SCREEN_SIZE),
            ..default()
        },
        texture: image,
        ..default()
    });

    commands.spawn(Camera2dBundle::default());

}

fn swap_colours(
    mut images: ResMut<Assets<Image>>,
    id: Res<GameOfLifeImage>,
    mut app_state: ResMut<State<AppState>>,
    keys: Res<Input<KeyCode>>,
) {

    // if !keys.just_pressed(KeyCode::Space) {
    //     return;   
    // }

    let handle = Handle::weak(id.0);

    if let Some(image) = images.get_mut(&handle) {
        let mut new_image: Vec<u8> = vec![0; image.data.len()];

        let old_image: Vec<u8> = image
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| (i + 1) % 4 == 0)
            .map(|x| *x.1)
            .collect();

        for (i, x) in old_image.iter().enumerate() {
            let num_neighbors = count_neighbors(&old_image, i.try_into().unwrap(), CAVE_COLOURS[1][3], true);

            if *x == CAVE_COLOURS[1][3] {
                if num_neighbors < 3 {
                    new_image[i] = CAVE_COLOURS[0][3]
                } else {
                    new_image[i] = CAVE_COLOURS[1][3]
                }
            } else {
                if num_neighbors > 4 {
                    new_image[i] = CAVE_COLOURS[1][3]
                } else {
                    new_image[i] = CAVE_COLOURS[0][3]
                }
            }

            // This is maze generation
            // if num_neighbors == 3 {
            //     new_image[i] = CAVE_COLOURS[1][3];
            // } else if (1..=5).contains(&num_neighbors) {
            //     new_image[i] = *x
            // } else {
            //     new_image[i] = CAVE_COLOURS[0][3]
            // }
        }

        let mut equivalent = true;

        for (i, x) in image.data.iter_mut().enumerate() {
            if (i + 1) % 4 == 0 {
                let y = new_image[((i + 1) / 4) - 1];
                if !*x == y {
                    *x = y;
                    equivalent = false;
                }
            }
        }

        if equivalent || keys.just_pressed(KeyCode::Space) {
            println!("I am in your caves");
            app_state.set(AppState::NotGenerating).unwrap()
        }
    }
}

// pixels with that colour are counted as live neighbors
fn count_neighbors(image: &Vec<u8>, pixel: i32, colour: u8, diagonals: bool) -> i32 {
    let mut total = 0;

    for x in (0..3).map(|x| x - 1) {
        for y in (0..3).map(|y| y - 1) {
            if (x == y && x == 0) || (!diagonals && (x != 0 && y != 0)) {
                continue;
            } else {

                let index = x + (y * CONWAYS_MAP_SIZE.x as i32);

                if pixel + index < 0
                    || (pixel % CONWAYS_MAP_SIZE.x as i32 == 0 && x < 0)
                    || ((pixel + 1) % CONWAYS_MAP_SIZE.x as i32 == 0 && x > 0)
                {
                    continue;
                }

                let index = (pixel + index) as usize;

                if index >= image.len() {
                    continue;
                } else {
                    let number = image[index];

                    if number == colour {
                        total += 1;
                    }
                }
            }
        }
    }
    total
}