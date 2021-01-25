use std::marker::PhantomData;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

pub mod file_system;
mod gfx;
pub use gfx::GfxSettings;
pub mod gui;
pub mod input;
pub mod math;
pub mod time;
use gfx::GfxRenderer;
use gui::RenderQueue;

pub use job_scheduler::JobScheduler;

pub trait GameImpl: Send {
    fn default(queue: &mut gui::RenderQueue) -> Self;
    fn sim_hz() -> u32;
    fn gfx_settings() -> GfxSettings;
    fn update(&mut self, input: &Vec<input::Input>) -> Vec<SystemMessage>;
    fn render(&self, queue: &mut gui::RenderQueue);
}

pub enum SystemMessage {
    UpdateRenderFps(u32),
}

pub struct Game<TApplication>
where
    TApplication: GameImpl + 'static,
{
    phantom: PhantomData<TApplication>,
}

#[derive(Debug)]
pub struct Event {
    time: time::Duration,
    input: input::Input,
}

/// A queue for engine driven events.
pub struct EventQueue {
    events: Vec<Event>,
    start_time: time::Instant,
}

impl EventQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            start_time: time::Clock::now(),
        }
    }

    fn reset_start_time(&mut self) {
        self.start_time = time::Clock::now();
    }

    /// Returns the duration since initialization.
    pub fn duration(&self) -> time::Duration {
        time::Clock::now() - self.start_time
    }

    /// Pushes a new event onto the queue.
    pub fn push(&mut self, input: input::Input) {
        self.events.push(Event {
            time: self.duration(),
            input,
        });
    }

    /// Pops the top item off the EventQueue.
    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop()
    }

    /// Peeks the top item on the EventQueue.
    pub fn peek(&self) -> Option<&Event> {
        if self.events.is_empty() {
            return None;
        }

        return Some(&self.events[0]);
    }

    /// Returns whether the event queue is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

const EVENT_CAPACITY: usize = 200;
const RENDER_CAPACITY: usize = 1000;

impl<TApplication> Game<TApplication>
where
    TApplication: GameImpl + 'static,
{
    pub fn run(title: &'static str) {
        // Initialize events
        let mut event_queue = EventQueue::new(EVENT_CAPACITY);
        let mut render_queue = RenderQueue::new(RENDER_CAPACITY);

        // Create game state, render loop and scheduler
        let game = TApplication::default(&mut render_queue);
        let (event_loop, window, mut gfx_state) = gfx::setup(title, TApplication::gfx_settings());
        gfx_state.update(&render_queue, &mut event_queue);

        const MAX_THREADS: u32 = 4; // TODO: make this configurable by build?
        let mut job_scheduler = JobScheduler::new(MAX_THREADS);

        let mut input_handler = gfx::InputHandler::new();

        let (input_sender, input_receiver): (Sender<SimJobMessage>, Receiver<SimJobMessage>) =
            mpsc::channel();

        let (render_command_sender, render_command_receiver): (
            Sender<Vec<gui::RenderCommand>>,
            Receiver<Vec<gui::RenderCommand>>,
        ) = mpsc::channel();

        let (render_behavior_sender, render_behavior_receiver) = mpsc::channel();

        // Move the sim execution to a separate thread
        {
            let sim = SimExecutor::new(
                TApplication::sim_hz(),
                input_receiver,
                game,
                render_command_sender,
                event_queue.duration(),
            );

            let job_inbox = job_scheduler.inbox();

            job_scheduler.queue(move || sim_job(sim, job_inbox));
        }

        // Move the render execution to a separate thread
        let (render_job_sender, render_job_receiver) = mpsc::channel();

        {
            let render_executor = RenderExecutor::new(
                gfx_state,
                render_job_receiver,
                render_command_receiver,
                render_behavior_receiver,
                render_queue,
                input_sender.clone(),
            );

            let job_inbox = job_scheduler.inbox();

            job_scheduler.queue(move || render_job(render_executor, job_inbox));
        }

        // Start the main loop
        let mut shutting_down = false;
        event_loop.run(move |event, _, control_flow| {
            // Register inputs + render
            match input_handler.handle_events(event, control_flow, &window, &render_behavior_sender)
            {
                Some(input) => {
                    // If going to exit, signal a shutdown.
                    if input == input::Input::ApplicationExit {
                        shutting_down = true;
                        input_sender.send(SimJobMessage::Shutdown).unwrap();
                        render_job_sender.send(RenderJobMessage::Shutdown).unwrap();
                    }

                    // Otherwise log the event
                    event_queue.push(input);
                }
                None => {}
            }

            // Send inputs from event queue to simulation
            if event_queue.is_empty() == false {
                let mut inputs_to_send = vec![];

                while let Some(event) = event_queue.pop() {
                    inputs_to_send.push(event);
                }

                match input_sender.send(SimJobMessage::Input(inputs_to_send)) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }

            // Process jobs
            job_scheduler.process();

            // Check if shutting down
            if shutting_down {
                // Wait for jobs to complete...
                while job_scheduler.process() > 0 {}

                // Kill the application.
                return;
            }
        });
    }
}

fn render_job<Renderer>(
    mut renderer: RenderExecutor<Renderer>,
    job_inbox: Sender<job_scheduler::JobFunction>,
) where
    Renderer: gfx::GfxRenderer + Send + 'static,
{
    // Check if there's new content to render
    if renderer.execute() {
        // Queue up next execution.
        let queue = job_inbox.clone();

        queue
            .send(Box::new(move || render_job(renderer, job_inbox)))
            .unwrap();
    }
}

enum RenderJobMessage {
    Shutdown,
}

struct RenderExecutor<Renderer>
where
    Renderer: gfx::GfxRenderer,
{
    renderer: Renderer,
    render_job_inbox: Receiver<RenderJobMessage>,
    render_command_receiver: Receiver<Vec<gui::RenderCommand>>,
    render_behavior_receiver: Receiver<gfx::GfxMsgs>,
    render_queue: RenderQueue,
    event_queue: EventQueue,
    sim_sender: Sender<SimJobMessage>,
}

impl<Renderer> RenderExecutor<Renderer>
where
    Renderer: gfx::GfxRenderer,
{
    fn new(
        renderer: Renderer,
        render_job_inbox: Receiver<RenderJobMessage>,
        render_command_receiver: Receiver<Vec<gui::RenderCommand>>,
        render_behavior_receiver: Receiver<gfx::GfxMsgs>,
        render_queue: RenderQueue,
        sim_sender: Sender<SimJobMessage>,
    ) -> Self {
        Self {
            renderer,
            render_job_inbox,
            render_command_receiver,
            render_behavior_receiver,
            render_queue,
            event_queue: EventQueue::new(EVENT_CAPACITY),
            sim_sender,
        }
    }

    fn execute(&mut self) -> bool {
        // Check for shutdowns
        for event in self.render_job_inbox.try_recv() {
            match event {
                RenderJobMessage::Shutdown => {
                    return false;
                }
            }
        }

        let mut should_render = false;
        for render_msg in self.render_behavior_receiver.try_recv() {
            match render_msg {
                gfx::GfxMsgs::Render => {
                    should_render = true;
                }
                gfx::GfxMsgs::Resize { width, height } => {
                    self.renderer.resize(width, height);
                    break;
                }
            }
        }

        let mut updated_render_state = false;
        for (i, render_commands) in self
            .render_command_receiver
            .try_recv()
            .iter_mut()
            .enumerate()
        {
            self.render_queue.clear();
            updated_render_state = true;

            while let Some(render_command) = render_commands.pop() {
                self.render_queue.push(render_command);
            }
        }

        if updated_render_state {
            self.renderer
                .update(&self.render_queue, &mut self.event_queue);

            // Drain event queue and send back to main thread
            let mut events = vec![];
            while let Some(event) = self.event_queue.pop() {
                events.push(event);
            }

            if events.is_empty() == false {
                match self.sim_sender.send(SimJobMessage::Input(events)) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }

        if should_render {
            self.renderer.render();
        }

        return true;
    }
}

/// Reoccuring job for the game simulation.
fn sim_job<TApplication>(
    mut simulation: SimExecutor<TApplication>,
    job_inbox: Sender<job_scheduler::JobFunction>,
) where
    TApplication: GameImpl + 'static,
{
    if simulation.execute() {
        let queue = job_inbox.clone();

        queue
            .send(Box::new(move || sim_job(simulation, job_inbox)))
            .unwrap();
    }
}

/// Enumeration for channel that can be sent inputs.
enum SimJobMessage {
    Shutdown,
    Input(Vec<Event>),
}

struct SimExecutor<TApplication>
where
    TApplication: GameImpl + 'static,
{
    physics_frame_time: time::Duration,
    physics_accumulated_time: time::Duration,
    frame_accumulated_time: time::Duration,
    frame_stopwatch: time::Clock,
    input_receiver: Receiver<SimJobMessage>,
    frame_event_queue: Vec<input::Input>,
    event_queue: Vec<Event>,
    game: TApplication,
    render_sender: Sender<Vec<gui::RenderCommand>>,
    render_queue: RenderQueue,
}

impl<TApplication> SimExecutor<TApplication>
where
    TApplication: GameImpl + 'static,
{
    fn new(
        sim_hz: u32,
        input_receiver: Receiver<SimJobMessage>,
        game: TApplication,
        render_sender: Sender<Vec<gui::RenderCommand>>,
        frame_time_offset: time::Duration,
    ) -> Self {
        // Init time settings. Physics runs at a set hz, but is divorced from actual time in
        // the event that the renderer takes too long.
        // Loop is based on https://gafferongames.com/post/fix_your_timestep/
        let event_queue = Vec::with_capacity(EVENT_CAPACITY);

        let physics_frame_time = time::Duration::from_secs_f32(1.0) / sim_hz;
        let physics_accumulated_time = frame_time_offset;
        let frame_accumulated_time = time::Duration::from_secs_f32(0.0);

        let frame_event_queue = Vec::with_capacity(EVENT_CAPACITY);

        let render_queue = RenderQueue::new(RENDER_CAPACITY);
        let frame_stopwatch = time::Clock::new();

        Self {
            physics_frame_time,
            physics_accumulated_time,
            frame_accumulated_time,
            frame_stopwatch,
            input_receiver,
            frame_event_queue,
            event_queue,
            game,
            render_sender,
            render_queue,
        }
    }

    fn execute(&mut self) -> bool {
        // Increment the frame time
        self.frame_accumulated_time += self.frame_stopwatch.stop_watch();

        // Log received events
        for event in self.input_receiver.try_recv() {
            match event {
                SimJobMessage::Shutdown => {
                    // TODO: ensure that this is the proper way to shut down.
                    return false;
                }
                SimJobMessage::Input(events) => {
                    for event in events {
                        self.event_queue.push(event);
                    }
                }
            }
        }

        // Execute the sim until it's caught up
        let mut new_render_state = false;
        while self.frame_accumulated_time >= self.physics_frame_time {
            self.frame_accumulated_time -= self.physics_frame_time;
            new_render_state = true;
            self.physics_accumulated_time += self.physics_frame_time;

            // Sort input, ensuring one's that have a lower time stamp are first.
            self.event_queue
                .sort_by(|a, b| b.time.partial_cmp(&a.time).unwrap());

            // Parse all input that is relevant to the frame
            while let Some(event) = self.event_queue.pop() {
                self.frame_event_queue.push(event.input);
            }

            // Now trigger update
            let system_messages = self.game.update(&self.frame_event_queue);
            self.frame_event_queue.clear();
            for msg in &system_messages {
                match msg {
                    SystemMessage::UpdateRenderFps(new_fps) => {
                        println!("todo: update system fps to {:?}", new_fps);
                    }
                }
            }
        }

        // If there's a new render state, grab the latest sim state and update the gfx pipeline.
        if new_render_state {
            self.render_queue.clear();
            self.game.render(&mut self.render_queue);

            match self.render_sender.send(self.render_queue.drain()) {
                Ok(_) => {}
                Err(e) => {
                    println!(
                        "Received an error sending render commands: '{:?}'. Exiting.",
                        e
                    );
                    return false;
                }
            }
        }

        return true;
    }
}
