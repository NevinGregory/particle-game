use bevy::{
    prelude::*,
    input::common_conditions::*,
    window::PrimaryWindow,
};

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.

const WIDTH: usize = 128;
const HEIGHT: usize = 128;

const CELL_SIZE: f32 = 4.; //Scale for cells

const COLORS: [Color; 5] = [Color::srgb(0.67, 0.88, 0.91), //Air
                            Color::srgb(1.0, 0.906, 0.702), //Sand
                            Color::srgb(0.0, 0.0, 1.0), //Water
                            Color::srgb(0.5, 0.5, 0.5), // Rock
                            Color::srgb(0.2, 0.2, 0.2) // Smoke
];

#[derive(Resource, Default)]
struct ParticleType(i32);

#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

#[derive(Resource)]
struct MyCellArray([i32; WIDTH * HEIGHT]);

//#[derive(Resource)]
//struct MyWorldTex(Image);

#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins((
                DefaultPlugins, 
        ))
        .insert_resource(ClearColor(COLORS[3]))
        .insert_resource(MyWorldCoords(Vec2::new(0.0, 0.0)))
        .insert_resource(MyCellArray([0; WIDTH * HEIGHT]))
        .insert_resource(ParticleType(1))
        .add_systems(Startup, setup)
        // Add our gameplay simulation systems to the fixed timestep schedule
        // which runs at 64 Hz by default
        //.add_systems(
        //    FixedUpdate,
        //    (
        //        //apply_velocity,
        //    )
        //        // `chain`ing systems together runs them in order
        //        .chain(),
        //)
        .add_systems(Update, 
            (
                select_type,
                my_cursor_system
                    .run_if(input_pressed(MouseButton::Left)),
                update_array,
                draw_array,
            ).chain(),
        )
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Cell {
    val: i32,
    index: usize,
}


// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    myarray: Res<MyCellArray>,
) {
    // Camera
    commands.spawn((Camera2d, MainCamera));

    for row in 0..HEIGHT {
        for column in 0..WIDTH {

            let val = myarray.0[column + HEIGHT * row];
            //println!("{}, {} corresponds to {}", column, row, column + HEIGHT * row);
            commands.spawn((
                Sprite {
                    color: COLORS[0],
                    ..default()
                },
                Transform {
                    translation: Vec3::new(column as f32 * CELL_SIZE - WIDTH as f32 / 2. * CELL_SIZE, row as f32 * CELL_SIZE - HEIGHT as f32 / 2. * CELL_SIZE, 0.0),
                    scale: Vec3::new(CELL_SIZE, CELL_SIZE, 1.0),
                    ..default()
                },
                Cell {
                    val: val as i32,
                    index: column + HEIGHT * row,
                },
            ));
        }
    }
}

fn update_array(
    mut myarray: ResMut<MyCellArray>,
) {
    for row in 0..HEIGHT {
        let mut column = 0;
        while column < WIDTH {
            let val = myarray.0[column + HEIGHT * row];
            match val {
                1 => { // Sand
                    if row > 0 {
                        if myarray.0[column + HEIGHT * (row - 1)] == 0 || myarray.0[column + HEIGHT * (row - 1)] == 2 {
                            //Cell below is empty
                            myarray.0[column + HEIGHT * row] = myarray.0[column + HEIGHT * (row - 1)];
                            myarray.0[column + HEIGHT * (row - 1)] = val;
                        } else if column > 0 && (myarray.0[(column - 1) + HEIGHT * (row - 1)] == 0 || column > 0 && myarray.0[(column - 1) + HEIGHT * (row - 1)] == 2) {
                            //Cell down and left is empty
                            myarray.0[column + HEIGHT * row] = myarray.0[(column - 1) + HEIGHT * (row - 1)];
                            myarray.0[(column - 1) + HEIGHT * (row - 1)] = val;
                        } else if column < WIDTH - 1 && (myarray.0[(column + 1) + HEIGHT * (row - 1)] == 0 || column < WIDTH - 1 && myarray.0[(column + 1) + HEIGHT * (row - 1)] == 2) {
                            //Cell down and right is empty
                            myarray.0[column + HEIGHT * row] = myarray.0[(column + 1) + HEIGHT * (row - 1)];
                            myarray.0[(column + 1) + HEIGHT * (row - 1)] = val;
                        }
                    }
                }
                2 => { // Water
                    if row > 0  && myarray.0[column + HEIGHT * (row - 1)] == 0 {
                        //Cell below is empty
                        myarray.0[column + HEIGHT * (row - 1)] = val;
                        myarray.0[column + HEIGHT * row] = 0;
                    } else if row > 0  && column > 0 && myarray.0[(column - 1) + HEIGHT * (row - 1)] == 0 {
                        //Cell down and left is empty
                        myarray.0[(column - 1) + HEIGHT * (row - 1)] = val;
                        myarray.0[column + HEIGHT * row] = 0;
                    } else if row > 0  && column < WIDTH - 1 && myarray.0[(column + 1) + HEIGHT * (row - 1)] == 0 {
                        //Cell down and right is empty
                        myarray.0[(column + 1) + HEIGHT * (row - 1)] = val;
                        myarray.0[column + HEIGHT * row] = 0;
                    } else if column > 0 && myarray.0[(column - 1) + HEIGHT * row] == 0 {
                        //Cell right is empty
                        //println!("Flowing Left!");
                        myarray.0[(column - 1) + HEIGHT * row] = val;
                        myarray.0[column + HEIGHT * row] = 0;
                    } else if column < WIDTH - 1 && myarray.0[(column + 1) + HEIGHT * row] == 0 {
                        //Cell right is empty
                        //println!("Flowing right!");
                        myarray.0[(column + 1) + HEIGHT * row] = val;
                        myarray.0[column + HEIGHT * row] = 0;
                        column += 1;
                    }
                }
                _ => {}
            };
            column += 1;
        }
    }
    for row in (0..WIDTH).rev() {
        let mut column = 0;
        while column < WIDTH {
            let val = myarray.0[column + HEIGHT * row];
            match val {
                4 => { // Smoke
                    if row < HEIGHT-1 {
                        if myarray.0[column + HEIGHT * (row + 1)] == 0 {
                            //Cell above is empty
                            myarray.0[column + HEIGHT * row] = 0;
                            myarray.0[column + HEIGHT * (row + 1)] = val;
                        } else if column > 0 && myarray.0[(column - 1) + HEIGHT * (row + 1)] == 0 {
                            //Cell up and left is empty
                            myarray.0[column + HEIGHT * row] = 0;
                            myarray.0[(column - 1) + HEIGHT * (row + 1)] = val;
                        } else if column < WIDTH - 1 && myarray.0[(column + 1) + HEIGHT * (row + 1)] == 0 {
                            //Cell up and right is empty
                            myarray.0[column + HEIGHT * row] = 0;
                            myarray.0[(column + 1) + HEIGHT * (row + 1)] = val;
                        } else if column > 0 && myarray.0[(column - 1) + HEIGHT * row] == 0 {
                            //Cell right is empty
                            myarray.0[(column - 1) + HEIGHT * row] = val;
                            myarray.0[column + HEIGHT * row] = 0;
                        } else if column < WIDTH - 1 && myarray.0[(column + 1) + HEIGHT * row] == 0 {
                            //Cell right is empty
                            myarray.0[(column + 1) + HEIGHT * row] = val;
                            myarray.0[column + HEIGHT * row] = 0;
                            column += 1;
                        }
                    }
                }
                _ => {}
            }
            column += 1;
        }
    }
}

fn draw_array(
    myarray: Res<MyCellArray>,
    mut cell_query: Query<(&mut Cell, &mut Sprite), With<Cell>>,
) {
    for (mut cell, mut sprite) in &mut cell_query{
        cell.val = myarray.0[cell.index];
        sprite.color = COLORS[cell.val as usize];
    }
}

//fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
//    for (mut transform, velocity) in &mut query {
//        transform.translation.x += velocity.x * time.delta_secs();
//        transform.translation.y += velocity.y * time.delta_secs();
//    }
//}

fn select_type(
    keys: Res<ButtonInput<KeyCode>>,
    mut particle_type: ResMut<ParticleType>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        particle_type.0 = 1;
    } else if keys.just_pressed(KeyCode::Digit2) {
        particle_type.0 = 2;
    } else if keys.just_pressed(KeyCode::Digit3) {
        particle_type.0 = 3;
    } else if keys.just_pressed(KeyCode::Digit4) {
        particle_type.0 = 4;
    }
}

fn my_cursor_system(
    mut mycoords: ResMut<MyWorldCoords>,
    mut myarray: ResMut<MyCellArray>,
    particle_type: Res<ParticleType>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| Some(camera.viewport_to_world(camera_transform, cursor)))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        mycoords.0 = world_position;
        let r =  (world_position.y / CELL_SIZE as f32).floor() as i32 + HEIGHT as i32/2 + 1;
        let c = (world_position.x / CELL_SIZE as f32).floor() as i32 + WIDTH as i32/ 2 + 1;

        if (r < HEIGHT as i32 && r >= 0) && (c < WIDTH as i32 && c >= 0) {
            myarray.0[coords_to_index(r, c)] = particle_type.0;
            myarray.0[coords_to_index(r + 1, c)] = particle_type.0;
            myarray.0[coords_to_index(r - 1, c)] = particle_type.0;
            myarray.0[coords_to_index(r, c + 1)] = particle_type.0;
            myarray.0[coords_to_index(r, c - 1)] = particle_type.0;
            myarray.0[coords_to_index(r + 1, c - 1)] = particle_type.0;
            myarray.0[coords_to_index(r - 1, c - 1)] = particle_type.0;
            myarray.0[coords_to_index(r + 1, c + 1)] = particle_type.0;
            myarray.0[coords_to_index(r - 1, c + 1)] = particle_type.0;
            //eprintln!("World coords: {}/{}", world_position.x.floor(), world_position.y.floor());
            //eprintln!("Converted coords: {}/{}, {}", r, c, c + HEIGHT as i32 * r);
        }
    }
}

fn coords_to_index(
    r: i32,
    c: i32,
) -> usize {
    c as usize + HEIGHT * r as usize
}

