use libservo::{gl, Servo};
use glutin::{GlContext, GlWindow};
use libservo::servo_config::resource_files::set_resources_path;
use libservo::servo_config::opts;
use libservo::ipc_channel::ipc;
use libservo::servo_url::ServoUrl;
use libservo::msg::constellation_msg::TopLevelBrowsingContextId;
use libservo::compositing::windowing::WindowEvent;

use std::env;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use amethyst::winit::EventsLoopProxy;
use amethyst::ecs::{Component, VecStorage};

use super::ServoWindow;

pub struct ServoHandle {
    pub servo: Servo<ServoWindow>,
    pub window: Rc<ServoWindow>,
    pub id: TopLevelBrowsingContextId,
}

/// FIXME: YOU'RE GOING TO KILL SOMEONE
unsafe impl Sync for ServoHandle {}
unsafe impl Send for ServoHandle {}

impl Component for ServoHandle {
    type Storage = VecStorage<ServoHandle>;
}

impl ServoHandle {
    pub fn update(&mut self) {
        self.servo.handle_events(vec![]);
    }

    pub fn navigate(&mut self, url: &str) -> Result<(), String> {
        match ServoUrl::parse(url) {
            Ok(url) => {
                self.servo
                    .handle_events(vec![WindowEvent::LoadUrl(self.id, url)]);
                Ok(())
            }
            Err(e) => Err(format!("Failed to parse URL: {}", e)),
        }
    }
}

impl ServoHandle {
    pub fn start_servo(window: &Arc<GlWindow>, events: &EventsLoopProxy, url: &str) -> ServoHandle {
        // Fetch gl context
        let gl = unsafe {
            window
                .context()
                .make_current()
                .expect("Failed to make current");
            gl::GlFns::load_with(|s| window.context().get_proc_address(s) as *const _)
        };

        // Create renderer
        let renderer = Rc::new(ServoWindow {
            gl: gl,
            waker: events.clone(),
            window: window.clone(),
            dimensions: Arc::new(Mutex::new((1024, 1024))),
            target_texture: Arc::new(Mutex::new(None)),
            buffers: Arc::new(Mutex::new(None)),
        });

        // Get resources
        let path = env::current_dir().unwrap().join("resources");
        set_resources_path(Some(path.to_str().unwrap().to_string()));
        let mut opts = opts::default_opts();
        opts.dump_display_list = true;
        opts::set_defaults(opts);

        // Start servo
        let mut servo = Servo::new(renderer.clone());

        // Launch servo
        let url = ServoUrl::parse(&url).unwrap();
        let (sender, receiver) = ipc::channel().unwrap();
        servo.handle_events(vec![WindowEvent::NewBrowser(url, sender)]);
        let id = receiver.recv().unwrap();
        servo.handle_events(vec![WindowEvent::SelectBrowser(id)]);

        ServoHandle {
            window: renderer.clone(),
            servo: servo,
            id: id,
        }
    }
}
