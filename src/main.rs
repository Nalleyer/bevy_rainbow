use bevy::{prelude::*, render::mesh::Indices, render::pipeline::PrimitiveTopology};

fn make_mesh(size: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let indices = vec![0, 1, 2, 2, 3, 0];
    let vertices = &[
        ([0., 0., 0.], [0., 0., 1.], [0., 0.]),
        ([0., size, 0.0], [0., 0., 1.], [0., 1.]),
        ([size, size, 0.0], [0., 0., 1.], [1., 1.]),
        ([size, 0., 0.0], [0., 0., 1.], [0., 0.]),
    ];
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

struct MousePos(Vec2);

#[derive(Default)]
struct State {
    // Set up from example
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

fn mouse_movement_updating_system(
    mut mouse_pos: ResMut<MousePos>,
    windows: Res<Windows>,
    mut state: Local<State>,
    cursor_moved_events: Res<Events<CursorMoved>>,
) {
    let window = windows.get_primary().unwrap();
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        mouse_pos.0.x = event.position.x - window.width() / 2.;
        mouse_pos.0.y = event.position.y - window.height() / 2.;
    }
}

// #[derive(Bundle)]
struct Player {
    size: f32,
}

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let white = materials.add(Color::rgb(1.0, 1.0, 1.0).into());
    commands.spawn(Camera2dBundle::default());
    let player = Player { size: 100. };
    commands
        .spawn(SpriteBundle {
            mesh: meshes.add(make_mesh(player.size)),
            material: white,
            sprite: Sprite {
                size: Vec2::new(1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(player);
}

fn move_system(mouse_pos: Res<MousePos>, mut query: Query<(&mut Transform, &Player)>) {
    debug!("{:?}", query.iter_mut().count());
    for (mut trans, player) in query.iter_mut() {
        trans.translation.x = mouse_pos.0.x - player.size / 2.;
        trans.translation.y = mouse_pos.0.y - player.size / 2.;
        debug!("({},{})", trans.translation.x, trans.translation.y);
    }
}

#[bevy_main]
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(MousePos(Vec2::new(0.0, 0.0)))
        .add_startup_system(setup.system())
        .add_system(mouse_movement_updating_system.system())
        .add_system(move_system.system())
        .run();
}
