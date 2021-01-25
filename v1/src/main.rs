mod fighting_game;

use portia::{
    gui::{AssetCommand, RenderCommand, RenderQueue},
    input::{Input, Window},
    GameImpl, GfxSettings, JobScheduler, SystemMessage,
};

fn main() {
    portia::Game::<FinalDestination>::run("FinalDest");
}

const DRAW_GLTF: bool = true;

pub struct FinalDestination {
    model_rot_degrees: f32,
    eye: [f32; 3],
    target: [f32; 3],
    rollback_game: fighting_game::RollbackGame,
    input_poller: fighting_game::input_poller::InputPoller,
    player: u8,
}

//const TEST_GLTF: &'static str = "wizard_revised.glb";
const TEST_GLTF: &'static str = "models/monkey/monkey_2.glb";

//const TEST_GLTF: &'static str = "models/waterbottle/WaterBottle.glb";

//const TEST_GLTF: &'static str = "models/cube/cube.gltf";
//const TEST_GLTF: &'static str = "models/sphere/sphere.gltf";
//const TEST_GLTF: &'static str = "models/FlightHelmet/FlightHelmet.gltf";

impl GameImpl for FinalDestination {
    fn default(queue: &mut RenderQueue) -> Self {
        queue.push(RenderCommand::Asset(AssetCommand::LoadObj {
            file: "monkey.obj",
            max_instances: 200,
        }));

        if DRAW_GLTF {
            queue.push(RenderCommand::Asset(AssetCommand::LoadGltf {
                file: TEST_GLTF,
            }));
        }

        queue.push(RenderCommand::CameraUpdate {
            target: [0.0, 0.0, 0.0],
            eye: [0.0, 1.0, 2.0],
            orthographic: false,
        });

        let frame_delay = 5;
        let game_version = "0.0.1";
        let rollback_modes = vec![];

        let mut rollback_game =
            fighting_game::RollbackGame::new(game_version, frame_delay, rollback_modes);
        let player = rollback_game.add_player();

        Self {
            input_poller: fighting_game::input_poller::InputPoller::new(),
            model_rot_degrees: 0.0,
            eye: [0.0, 0.0, 10.0],
            target: [0.0, 0.0, 0.0],
            rollback_game,
            player,
        }
    }

    fn sim_hz() -> u32 {
        60
    }

    fn gfx_settings() -> GfxSettings {
        let physical_resolution = (1920, 1080);
        let render_resolution = (640, 360);
        // let render_resolution = (640 / 2, 360 / 2); // super low res test
        let render_resolution = physical_resolution; // Testing for now.

        GfxSettings {
            physical_resolution,
            render_resolution,
            fps: 144,
        }
    }

    fn update(&mut self, inputs: &Vec<Input>) -> Vec<SystemMessage> {
        for input in inputs {
            match input {
                Input::Window(window_event) => match window_event {
                    Window::DroppedFile(path) => loop {
                        println!("dropped: {:?}", path);
                    },
                    _ => {}
                },
                Input::AssetLoaded { file } => {
                    println!("Asset {:?} loaded!", file);
                }
                Input::Mouse(mouse) => match mouse {
                    portia::input::Mouse::Clicked { x, y, button } => {}
                    portia::input::Mouse::Released { x, y, button } => {}
                    portia::input::Mouse::Moved { x, y } => {
                        let (max_x, max_y) = Self::gfx_settings().physical_resolution;
                        let (max_x, max_y) = (max_x as f32, max_y as f32);

                        let x = *x as f32;
                        let y = *y as f32;

                        let x = x / max_x - 1.0;
                        let y = y / max_y - 1.0;
                    }
                },
                _ => {}
            }
        }

        let player0_input = self.input_poller.poll(inputs);

        self.rollback_game
            .register_local_input(self.player, player0_input);

        let events = self.rollback_game.tick();
        for event in events {
            match event {
                networking::rollback::RollbackEvent::Disconnected => {
                    unimplemented!("TODO: how to handle disconnects?");
                }
            }
        }

        self.model_rot_degrees += 1.;

        vec![]
    }

    fn render(&self, queue: &mut RenderQueue) {
        use cgmath::Rotation3;
        /*
        // Cube 2
        let scale = [0.25, 0.25, 0.25];
        let position = [-1.0, 0.0, 0.0];
        let rotation = cgmath::Quaternion::from_axis_angle(
            cgmath::Vector3::unit_z(),
            cgmath::Deg(-self.model_rot_degrees),
        ) * cgmath::Quaternion::from_axis_angle(
            cgmath::Vector3::unit_y(),
            cgmath::Deg(self.model_rot_degrees * 0.5),
        );

        queue.push(RenderCommand::ModelDraw {
            file: "monkey.obj",
            position: position.into(),
            scale: scale.into(),
            rotation,
        });
        */

        // Players
        for character in self.rollback_game.state().characters.iter() {
            let scale = [0.5, 0.5, 0.5];

            let position = character.position;
            let rotation = cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_z(),
                cgmath::Deg(-self.model_rot_degrees * 0.5),
            ) * cgmath::Quaternion::from_axis_angle(
                cgmath::Vector3::unit_y(),
                cgmath::Deg(-self.model_rot_degrees * 0.5),
            );

            queue.push(RenderCommand::ModelDraw {
                file: "monkey.obj",
                position: position.into(),
                scale: scale.into(),
                rotation,
            });

            // Aabbs for debugging
            {
                let mut z = character.position[2];
                let z_inc = 0.01;

                // pushbox
                let color = [0.0, 1.0, 0.0, 0.8];
                draw_aabbs(character.position, z, &character.push_boxes, color, queue);
                z += z_inc;
                // hitbox
                let color = [1.0, 0.0, 0.0, 0.8];
                draw_aabbs(character.position, z, &character.hit_boxes, color, queue);
                z += z_inc;

                // hurtbox
                let color = [0.0, 0.0, 1.0, 0.8];
                draw_aabbs(character.position, z, &character.hurt_boxes, color, queue);
                z += z_inc;

                // grabbox
                let color = [0.5, 0.5, 0.5, 0.8];
                draw_aabbs(character.position, z, &character.grab_boxes, color, queue);
            }
        }

        // Draw stage
        {
            let state = &self.rollback_game.state();
            let z = 0.0;
            let color = [0.9, 0.9, 0.9, 0.9];
            draw_aabbs(state.stage_position, z, &state.stage_aabbs, color, queue);
        }

        if DRAW_GLTF {
            queue.push(RenderCommand::GltfDraw { file: TEST_GLTF });
        }

        // Camera
        queue.push(RenderCommand::CameraUpdate {
            eye: self.eye,
            target: self.target,
            orthographic: false,
        });
    }
}

fn draw_aabbs(
    position: [f32; 3],
    z: f32,
    aabbs: &Vec<fighting_game::Aabb>,
    color: [f32; 4],
    queue: &mut RenderQueue,
) {
    for aabb in aabbs {
        let min = {
            let min = aabb.min;
            let mut min = [min[0], min[1], 2.];
            min[0] += position[0];
            min[1] += position[1];
            min[2] += position[2];
            min
        };

        let max = {
            let max = aabb.max;
            let mut max = [max[0], max[1], 2.];
            max[0] += position[0];
            max[1] += position[1];
            max[2] += position[2];
            max
        };

        let debug_shape = RenderCommand::DebugRectangle { min, max, z, color };
        queue.push(debug_shape);
    }
}
