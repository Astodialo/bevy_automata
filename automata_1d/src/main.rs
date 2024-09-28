use bevy::{
    prelude::*,
    render::camera::ScalingMode,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

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

const X: f32 = 40.;
const Y: f32 = X * 4.;
const C: f32 = 40.;
const SIZE: f32 = X / C;
const RULE: i32 = 110;
const COLORS: [Color; 2] = [Color::hsl(0.5, 0.75, 0.8), Color::hsl(0.5, 0.25, 0.2)];

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
        i == (C / 2.) as usize + 1
    }
    let mut cell_storage: Vec<Vec<(Entity, State, Vec3)>> = Vec::new();
    let mut row_storage: Vec<(Entity, State, Vec3)> = Vec::new();

    for i in 0..C as usize {
        let material = if spawn_rule(i) {
            materials.add(COLORS[1])
        } else {
            materials.add(COLORS[0])
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
        let material = if n_state == 0 {
            materials.add(COLORS[0])
        } else {
            materials.add(COLORS[1])
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

    cell_storage.0.push(row_storage)
}
