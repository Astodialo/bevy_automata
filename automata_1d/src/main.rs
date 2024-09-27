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

const X: f32 = 920.;
const C: f32 = 92.;
const SIZE: f32 = X / C;
const RULE: i32 = 110;
const COLORS: [Color; 2] = [Color::hsl(0.5, 0.75, 0.8), Color::hsl(0.5, 0.25, 0.2)];

#[derive(Component)]
struct State(f32);

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
                    height: X,
                },
                ..default()
            },
            ..default()
        },
        PanOrbitCamera::default(),
    ));

    let cell = Mesh2dHandle(meshes.add(Rectangle::new(SIZE, SIZE)));
    fn spawn_rule(i: usize) -> bool {
        i == (C / 2.) as usize
    }

    for i in 0..C as usize {
        let material = if spawn_rule(i) {
            materials.add(COLORS[1])
        } else {
            materials.add(COLORS[0])
        };

        let state = if spawn_rule(i) { State(1.) } else { State(0.) };

        let transform = Transform::from_xyz(
            -X / 2. + (SIZE / 2.) + (i as f32 * SIZE),
            X / 2. - (SIZE / 2.),
            0.,
        );

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: cell.clone(),
                material,
                transform,
                ..default()
            },
            state,
        ));
    }
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
    cells: Query<(&Transform, &State)>,
) {
    for (i, cell) in cells.iter().enumerate() {
        let mut p_cell = 0;
        let mut n_cell = 0;

        if i == 0 {
            p_cell = cells.iter().last().unwrap().1 .0 as i32;
        } else if i == cells.iter().len() - 1 {
            n_cell = cells.iter().collect::<Vec<(&Transform, &State)>>()[0].1 .0 as i32;
        } else {
            p_cell = cells.iter().collect::<Vec<(&Transform, &State)>>()[i - 1]
                .1
                 .0 as i32;
            n_cell = cells.iter().collect::<Vec<(&Transform, &State)>>()[i + 1]
                .1
                 .0 as i32;
        }

        let n_state = calc_state(p_cell, cell.1 .0 as i32, n_cell);
        let material = if n_state == 0 {
            materials.add(COLORS[0])
        } else {
            materials.add(COLORS[1])
        };
        let transform = Transform::from_xyz(
            cell.0.translation.x,
            cell.0.translation.y - SIZE,
            cell.0.translation.z,
        );

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(SIZE, SIZE))),
                material,
                transform,
                ..default()
            },
            State(n_state as f32),
        ));
    }
}
