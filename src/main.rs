use std::time::Duration;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
        pipeline::{CullMode, PipelineDescriptor, RasterizationStateDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

const SIZE: f32 = 100.;

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
struct MyMaterialWithVertexColorSupport {}

const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in float Vertex_X;
layout(location = 0) out float v_x;
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_x = Vertex_X;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in float v_x;

vec3 rainbow(float x)
{
    /*
        Target colors
        =============

        L  x   color
        0  0.0 vec4(1.0, 0.0, 0.0, 1.0);
        1  0.2 vec4(1.0, 0.5, 0.0, 1.0);
        2  0.4 vec4(1.0, 1.0, 0.0, 1.0);
        3  0.6 vec4(0.0, 0.5, 0.0, 1.0);
        4  0.8 vec4(0.0, 0.0, 1.0, 1.0);
        5  1.0 vec4(0.5, 0.0, 0.5, 1.0);
    */

    float level = floor(x * 6.0);
    float r = float(level <= 2.0) + float(level > 4.0) * 0.5;
    float g = max(1.0 - abs(level - 2.0) * 0.5, 0.0);
    float b = (1.0 - (level - 4.0) * 0.5) * float(level >= 4.0);
    return vec3(r, g, b);
}

void main() {
    o_Target = vec4(rainbow(v_x), 1.0);
}
"#;

type Vertice = ([f32; 3], [f32; 3], [f32; 2]);

fn vec2_to_array_3(vec: Vec2) -> [f32; 3] {
    [vec.x, vec.y, 0.0]
}

fn modify_mesh(mesh: &mut Mesh, vertices: &[Vertice], indices: Vec<u16>) {
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
}

fn make_mesh(vertices: &[Vertice], indices: Vec<u16>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    modify_mesh(&mut mesh, vertices, indices);
    mesh
}

fn make_player_mesh(size: f32) -> Mesh {
    let indices = vec![0, 2, 1, 2, 0, 3];
    let vertices = &[
        ([-size / 2., -size / 2., 0.], [0., 0., 1.], [0., 0.]),
        ([-size / 2., size / 2., 0.0], [0., 0., 1.], [0., 0.]),
        ([size / 2., size / 2., 0.0], [0., 0., 1.], [0., 0.]),
        ([size / 2., -size / 2., 0.0], [0., 0., 1.], [0., 0.]),
    ];
    make_mesh(vertices, indices)
}

struct MousePos(Vec2);

struct TailTimer(Timer);

#[derive(Default)]
struct State {
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

#[derive(Default, Clone, Copy, Debug)]
struct TailNode {
    pos: Vec2,
    velocity: Vec2,
}

const TAIL_LEN: usize = 32;

struct Player {
    size: f32,
    tail: [TailNode; TAIL_LEN],
}

struct Tail {
    player: Option<Entity>,
}

impl Player {
    pub fn push_tail_node(&mut self, pos: Vec2) {
        let mut velocity = pos - self.tail[0].pos;
        if pos.distance_squared(self.tail[0].pos) < 2. {
            velocity = self.tail[0].velocity;
        }
        let new_node = TailNode { pos, velocity };
        for i in (1..TAIL_LEN).rev() {
            self.tail[i] = self.tail[i - 1];
        }
        self.tail[0] = new_node;
    }

    #[allow(dead_code)]
    pub fn make_debug_tail(&mut self, pos: Vec2) {
        let scale = 200.;
        self.tail[0] = TailNode {
            pos,
            velocity: Vec2::new(1. * scale, 0.),
        };
        self.tail[1] = TailNode {
            pos: pos + Vec2::new(-1. * scale, 0. * scale),
            velocity: Vec2::new(1., -1.),
        };
        self.tail[2] = TailNode {
            pos: pos + Vec2::new(-2. * scale, 1. * scale),
            velocity: Vec2::new(-1.0, 0.),
        };
        self.tail[3] = TailNode {
            pos: pos + Vec2::new(-2. * scale, 2. * scale),
            velocity: Vec2::new(-1.0, 0.),
        };
    }
}

fn setup(
    commands: &mut Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut materials: ResMut<Assets<MyMaterialWithVertexColorSupport>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let white = color_materials.add(Color::rgb(1.0, 1.0, 1.0).into());
    commands.spawn(Camera2dBundle::default());
    let player = Player {
        size: SIZE,
        tail: [TailNode::default(); TAIL_LEN],
    };

    let mut pipeline_setting = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    });

    pipeline_setting
        .rasterization_state
        .replace(RasterizationStateDescriptor {
            cull_mode: CullMode::None,
            ..Default::default()
        });

    let pipeline_handle = pipelines.add(pipeline_setting);

    let player_entity = commands
        .spawn(SpriteBundle {
            mesh: meshes.add(make_player_mesh(SIZE)),
            material: white,
            sprite: Sprite {
                size: Vec2::new(1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(player)
        .current_entity();

    render_graph.add_system_node(
        "my_material_with_vertex_color_support",
        AssetRenderResourcesNode::<MyMaterialWithVertexColorSupport>::new(true),
    );

    render_graph
        .add_node_edge(
            "my_material_with_vertex_color_support",
            base::node::MAIN_PASS,
        )
        .unwrap();

    let material = materials.add(MyMaterialWithVertexColorSupport {});

    commands
        .spawn(MeshBundle {
            mesh: meshes.add(make_mesh(&[], vec![])),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(material)
        .with(Tail {
            player: player_entity,
        });
}

fn move_system(mouse_pos: Res<MousePos>, mut query: Query<&mut Transform, With<Player>>) {
    for mut trans in query.iter_mut() {
        trans.translation.x = mouse_pos.0.x;
        trans.translation.y = mouse_pos.0.y;
    }
}

fn tail_gen_system(
    time: Res<Time>,
    mut tail_timer: ResMut<TailTimer>,
    mut query: Query<(&Transform, &mut Player)>,
) {
    tail_timer.0.tick(time.delta_seconds());
    if !tail_timer.0.finished() {
        return;
    }
    for (trans, mut player) in query.iter_mut() {
        let pos = Vec2::new(trans.translation.x, trans.translation.y);
        player.push_tail_node(pos);
        // player.make_debug_tail(pos);
    }
}

fn tail_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, &Tail)>,
    query_a: Query<(&Player, &Transform)>,
) {
    for (mesh_handle, tail) in query.iter_mut() {
        if let Some(player_entity) = tail.player {
            if let Ok(player) = query_a.get_component::<Player>(player_entity) {
                let mut mesh = meshes.get_mut(mesh_handle).unwrap();
                make_tail_mesh(&mut mesh, player);
            } else {
                println!("not Player for this entity");
            }
        } else {
            println!("not player for this tail");
        }
    }
}

fn make_tail_indices() -> Vec<u16> {
    let mut triangles = vec![];
    for i in 0..TAIL_LEN - 1 {
        triangles.push((i, i + 1, 2 * i + TAIL_LEN));
        triangles.push((i + 1, 2 * i + TAIL_LEN, 2 * i + TAIL_LEN + 1));
    }
    for i in 1..TAIL_LEN - 1 {
        triangles.push((i, 2 * i + TAIL_LEN - 1, 2 * i + TAIL_LEN));
    }
    triangles
        .into_iter()
        .flat_map(|(a, b, c)| vec![a as u16, b as u16, c as u16])
        .collect()
}

fn get_normal(velocity: Vec2) -> Vec2 {
    // anti-clock 90 deg
    let mut normal = Vec2::new(velocity.y, -velocity.x).normalize();
    if normal.is_nan() {
        normal.x = 0.0;
        normal.y = 0.0;
    }
    normal
}

fn make_tail_mesh(mesh: &mut Mesh, player: &Player) {
    let mut main_tail = [Vec2::zero(); TAIL_LEN];
    for (i, node) in player.tail.iter().enumerate() {
        main_tail[i] = node.pos;
    }
    let mut sub_tail = [Vec2::zero(); (TAIL_LEN - 1) * 2];
    for i in 0..player.tail.len() {
        let normal = get_normal(player.tail[i].velocity);
        if i == 0 {
            sub_tail[0] = main_tail[0] + normal * SIZE;
        } else if i < player.tail.len() - 1 {
            let normal_last = get_normal(player.tail[i - 1].velocity);
            sub_tail[2 * i - 1] = main_tail[i] + normal_last * SIZE;
            sub_tail[2 * i] = main_tail[i] + normal * SIZE;
        } else {
            sub_tail[2 * i - 1] = main_tail[i] + normal * SIZE;
        }
    }

    let mut vertices = [([0.; 3], [0., 0., 1.], [0.; 2]); (TAIL_LEN - 1) * 4 - (TAIL_LEN - 2)];
    let indices = make_tail_indices();
    let mut colors = vec![0.; vertices.len()];
    for i in 0..main_tail.len() {
        vertices[i].0 = vec2_to_array_3(main_tail[i]);
        colors[i] = 1.0;
    }
    for i in 0..sub_tail.len() {
        vertices[i + TAIL_LEN].0 = vec2_to_array_3(sub_tail[i]);
        colors[i + TAIL_LEN] = 0.0;
    }
    modify_mesh(mesh, &vertices, indices);

    mesh.set_attribute("Vertex_X", VertexAttributeValues::from(colors));
}

#[bevy_main]
fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_asset::<MyMaterialWithVertexColorSupport>()
        .add_resource(MousePos(Vec2::new(0.0, 0.0)))
        .add_resource(TailTimer(Timer::new(Duration::from_millis(10u64), true)))
        .add_startup_system(setup.system())
        .add_system(mouse_movement_updating_system.system())
        .add_system(move_system.system())
        .add_system(tail_gen_system.system())
        .add_system(tail_system.system())
        .run();
}
