use bevy::{
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use noise::{Abs, BasicMulti, NoiseFn, Perlin};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        PanOrbitCameraPlugin,
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("automata"),
                    ..Default::default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    ))
    .add_systems(Startup, startup)
    .add_systems(Update, generate)
    .run();
}

const X: f32 = 160.;
const Y: f32 = X;
const C: f32 = 160.;
const SIZE: f32 = X / C;
const RULE: i32 = 110;

#[derive(Component)]
struct State(f32);

#[derive(Component)]
struct Storage(Vec<Vec<(Entity, State, Vec3)>>);

#[derive(Component)]
struct Row(Vec<(Entity, State, Vec3)>);

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera { ..default() },
            projection: OrthographicProjection {
                far: 1000.,
                near: -1000.,
                scaling_mode: ScalingMode::Fixed {
                    width: X,
                    height: Y,
                },
                ..default()
            },
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    let cell = Mesh2dHandle(meshes.add(Rectangle::new(SIZE, Y / C)));
    fn spawn_rule(i: usize) -> bool {
        i % 11 == 0
    }
    let mut cell_storage: Vec<Vec<(Entity, State, Vec3)>> = Vec::new();
    let mut row_storage: Vec<(Entity, State, Vec3)> = Vec::new();

    let noise = BasicMulti::<Perlin>::new(111);
    for i in 0..C as usize {
        let noise_val = noise.get([i as f64 / 200., 0.]) * 11.;
        dbg!(noise_val);
        let hue = noise_val as f32 * (360. * (i / 11) as f32 / (C / 11.));
        let material = if spawn_rule(i) {
            materials.add(Color::hsl(
                hue, //map_range(i as f32, 0., C as f32, 0., 1.),
                0.75, 0.75,
            ))
        } else {
            materials.add(Color::hsl(
                hue, //map_range(i as f32, 0., C as f32, 0., 1.),
                0.25, 0.25,
            ))
        };

        let state = if spawn_rule(i) { State(1.) } else { State(0.) };

        let transform = Transform::from_xyz(
            -X / 2. + (SIZE / 2.) + (i as f32 * SIZE),
            Y / 2. - (SIZE / 2.),
            0.,
        );

        let cell_entity = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: cell.clone(),
                    material,
                    transform,
                    ..default()
                },
                state,
            ))
            .id();

        let state = if spawn_rule(i) { State(1.) } else { State(0.) };
        row_storage.push((cell_entity, state, transform.translation));
    }

    cell_storage.push(row_storage);
    commands.spawn(Storage(cell_storage));
}

fn calc_state(a: i32, b: i32, c: i32) -> i32 {
    let ruleset = format!("{:b}", RULE);
    let mut chars: Vec<char> = ruleset.chars().collect();

    while chars.len() < 8 {
        chars.insert(0, '0')
    }

    let hood = a.to_string() + &b.to_string() + &c.to_string();
    let value = usize::from_str_radix(&hood, 2).unwrap();

    chars[7 - value].to_digit(2).unwrap() as i32
}

fn generate(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut storage: Query<&mut Storage>,
) {
    let mut row_storage: Vec<(Entity, State, Vec3)> = Vec::new();
    let mut cell_storage = storage.get_single_mut().unwrap();
    let last_row = cell_storage.0.last().unwrap();

    let mut i = 0;
    let noise = BasicMulti::<Perlin>::new(111);
    for cell in last_row {
        let p_cell;
        let n_cell;

        if i == 0 {
            p_cell = last_row.last().unwrap().1 .0;
            n_cell = last_row[i + 1].1 .0
        } else if i == last_row.len() - 1 {
            p_cell = last_row[i - 1].1 .0;
            n_cell = last_row[0].1 .0
        } else {
            p_cell = last_row[i - 1].1 .0;
            n_cell = last_row[i + 1].1 .0;
        }

        let n_state = calc_state(p_cell as i32, cell.1 .0 as i32, n_cell as i32);
        let noise_val = noise
            .get([i as f64 / 200., cell_storage.0.len() as f64 / 200.])
            .abs()
            * 11.;
        let hue = noise_val as f32 * (360. * (i / 11) as f32 / (C / 11.));
        let material = if n_state == 0 {
            materials.add(Color::hsl(
                hue, //map_range(i as f32, 0., C as f32, 0., 1.),
                0.75, 0.75,
            ))
        } else {
            materials.add(Color::hsl(
                hue, //map_range(i as f32, 0., C as f32, 0., 1.),
                0.25, 0.25,
            ))
        };
        let transform = Transform::from_xyz(cell.2.x, cell.2.y - SIZE, cell.2.z);
        let cell_entity = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(SIZE, SIZE))),
                    material,
                    transform,
                    ..default()
                },
                State(n_state as f32),
            ))
            .id();

        row_storage.push((cell_entity, State(n_state as f32), transform.translation));
        i += 1;
    }

    cell_storage.0.push(row_storage);
    if cell_storage.0.len() >= (Y / SIZE) as usize {
        cell_storage.0.remove(0);
    }
}
