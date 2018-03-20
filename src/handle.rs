extern crate glutin;
extern crate servo as libservo;
use self::libservo::gl;
use self::libservo::Servo;
use self::glutin::{GlContext, GlWindow, WindowEvent as GlutinWindowEvent};
use self::libservo::servo_config::resource_files::set_resources_path;
use self::libservo::servo_config::opts;
use self::libservo::ipc_channel::ipc;
use self::libservo::servo_url::ServoUrl;
use self::libservo::compositing::windowing::WindowEvent;

use std::env;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use amethyst::prelude::World;
use amethyst::renderer::ScreenDimensions;
use amethyst::winit::EventsLoopProxy;

use super::ServoWindow;

pub struct ServoHandle {
    pub window: Rc<ServoWindow>,
    pub servo: Servo<ServoWindow>,
}

impl ServoHandle {
    pub fn update(&mut self) {
        self.servo.handle_events(vec![]);
    }

    pub fn forward_events(&mut self, events: Vec<GlutinWindowEvent>) {
        let events: Vec<WindowEvent> = events
            .iter()
            .map(|e| match e {
                &GlutinWindowEvent::Resized(x, y) => {
                    self.window.set_dimensions(x, y);
                    WindowEvent::Resize
                }
                _ => WindowEvent::Idle,
            })
            .collect();
        self.servo.handle_events(events);
    }
}

impl ServoHandle {
    pub fn start_servo(world: &World) -> ServoHandle {
        // Fetch gl context
        let gl = unsafe {
            let window = world.read_resource::<Arc<GlWindow>>();
            window
                .context()
                .make_current()
                .expect("Failed to make current");
            gl::GlFns::load_with(|s| window.context().get_proc_address(s) as *const _)
        };

        // Dimensions
        let screen_dimensions = world.read_resource::<ScreenDimensions>();

        // Fetch window
        let window = world.read_resource::<Arc<GlWindow>>();

        // Create renderer
        let renderer = Rc::new(ServoWindow {
            gl: gl,
            waker: world.read_resource::<EventsLoopProxy>().clone(),
            window: window.clone(),
            dimensions: Arc::new(Mutex::new((1024, 1024))),
            target_texture: Arc::new(Mutex::new(None)),
            buffers: Arc::new(Mutex::new(None)),
        });

        // Get resources
        let path = env::current_dir().unwrap().join("resources");
        /*let mut url = "file://".to_string();
        url.push_str(&path.to_str().unwrap());
        url.push_str("/index.html");*/
        set_resources_path(Some(path.to_str().unwrap().to_string()));
        let mut opts = opts::default_opts();
        opts.dump_display_list = true;
        opts::set_defaults(opts);

        // Start servo
        let mut servo = Servo::new(renderer.clone());

        // Launch servo
        //let url = ServoUrl::parse(&url).unwrap();
        let url = ServoUrl::parse("https://servo.org").unwrap();
        let (sender, receiver) = ipc::channel().unwrap();
        servo.handle_events(vec![WindowEvent::NewBrowser(url, sender)]);
        let id = receiver.recv().unwrap();
        servo.handle_events(vec![WindowEvent::SelectBrowser(id)]);

        ServoHandle {
            servo: servo,
            window: renderer.clone(),
        }
    }
}
