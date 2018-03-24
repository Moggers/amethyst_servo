use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use libservo::compositing::compositor_thread::EventLoopWaker;
use libservo::gl;
use libservo::compositing::windowing::{AnimationState, EmbedderCoordinates, WindowMethods};
use glutin::GlWindow;
use libservo::euclid::{Length, TypedPoint2D, TypedRect, TypedScale, TypedSize2D};
use libservo::style_traits::DevicePixel;
use amethyst::winit::EventsLoopProxy;
use amethyst::renderer::Texture;
use gfx_device_gl::NewTexture;

pub struct ServoWindow {
    pub waker: EventsLoopProxy,
    pub gl: Rc<gl::Gl>,
    pub window: Arc<GlWindow>,
    // Needs interior mutability, so that resize event can mutate it
    pub dimensions: Arc<Mutex<(u32, u32)>>,
    pub target_texture: Arc<Mutex<Option<u32>>>,
    pub buffers: Arc<Mutex<Option<(u32, u32)>>>,
}

impl ServoWindow where {
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self.dimensions.lock() {
            Ok(d) => {
                let d = &d.clone();
                (d.0.clone(), d.1.clone())
            }
            Err(e) => {
                eprintln!("ERROR: Dimension lock for Servo implementation was poisoned, servo UI is not guaranteed to scale correctly and may cause race conditions.");
                let d = e.get_ref();
                (d.0.clone(), d.1.clone())
            }
        }
    }

    pub fn set_dimensions(&self, width: u32, height: u32) {
        match self.dimensions.lock() {
            Ok(ref mut dimensions) => {
                dimensions.0 = width;
                dimensions.1 = height;
            }
            Err(_) => {
                eprintln!("ERROR: Dimension lock for Servo implementation was poisoned, servo UI is not guaranteed to scale correctly and may cause race conditions.");
            }
        }
    }

    pub fn set_target(&self, targ: &Texture) {
        extern crate gfx_device_gl;
        let targ = targ.raw().deref().resource();
        match self.target_texture.lock() {
            Ok(ref mut target) => {
                let mut target = target.deref_mut();
                match targ {
                    &NewTexture::Texture(t) => {
                        *target = Some(t);
                    }
                    _ => {}
                }
            }
            Err(_) => {
                eprintln!("ERROR: Target texture lock poisoned.");
            }
        }
    }

    pub fn remove_target(&self) -> Result<(), String> {
        match self.target_texture.lock() {
            Ok(ref mut target) => {
                let mut target = target.deref_mut();
                *target = None;
                Ok(())
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    pub fn has_target(&self) -> Result<bool, String> {
        match self.target_texture.lock() {
            Ok(target) => match *target {
                Some(_) => Ok(true),
                None => Ok(false),
            },
            Err(_) => {
                eprintln!("ERROR: Target texture lock poisoned.");
                Err("Lock poisoned".into())
            }
        }
    }
    pub fn get_target(&self) -> Option<u32> {
        match self.target_texture.lock() {
            Ok(ref target) => target.deref().clone(),
            Err(ref e) => {
                eprintln!("ERROR: Target texture lock poisoned..");
                e.get_ref().deref().clone()
            }
        }
    }

    pub fn setup_framebuffer(&self, target: &Texture) -> Result<(), u32> {
        if let (Ok(option_texture), Ok(option_buffers)) =
            (self.target_texture.lock(), self.buffers.lock())
        {
            if let (Some(texture), Some((framebuffer, renderbuffer))) =
                (*option_texture, *option_buffers)
            {
                self.gl.delete_textures(&[texture]);
                self.gl.delete_framebuffers(&[framebuffer]);
                self.gl.delete_renderbuffers(&[renderbuffer]);
            }
        }
        self.set_target(target);
        // Fetch required width and height
        let (width, height) = self.get_dimensions();

        // Create FBO
        let frame_buffer = self.gl.gen_framebuffers(1)[0];
        self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, frame_buffer);

        // Texture
        let texture = self.get_target().unwrap();
        self.gl.bind_texture(gl::TEXTURE_2D, texture);
        self.gl.framebuffer_texture_2d(
            gl::DRAW_FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture.into(),
            0,
        );

        // Create depth buffer
        let depth_buffer = self.gl.gen_renderbuffers(1)[0];
        self.gl.bind_renderbuffer(gl::RENDERBUFFER, depth_buffer);
        self.gl.renderbuffer_storage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT,
            width as i32,
            height as i32,
        );
        // Bind depth buffer to FBO
        self.gl.framebuffer_renderbuffer(
            gl::DRAW_FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_buffer,
        );

        // Cleanup
        match self.gl.check_frame_buffer_status(gl::DRAW_FRAMEBUFFER) {
            gl::FRAMEBUFFER_COMPLETE => match self.buffers.lock() {
                Ok(mut fb) => {
                    self.gl
                        .insert_event_marker_ext(&"Finished setting up servo FBO resources");
                    *fb = Some((frame_buffer, depth_buffer));
                    Ok(())
                }
                Err(_) => {
                    self.gl.delete_framebuffers(&[frame_buffer]);
                    self.gl.delete_renderbuffers(&[depth_buffer]);
                    self.gl
                        .insert_event_marker_ext(&"Failed setting up servo FBO resources");
                    Err(0)
                }
            },
            e => {
                self.gl.delete_framebuffers(&[frame_buffer]);
                self.gl.delete_renderbuffers(&[depth_buffer]);
                self.gl
                    .insert_event_marker_ext(&"Failed setting up servo FBO resources");
                Err(e)
            }
        }
    }

    pub fn enable_fb(&self) -> Result<(), ()> {
        match self.buffers.lock() {
            Ok(guard) => match *guard {
                Some((framebuffer, _renderbuffer)) => {
                    self.gl
                        .insert_event_marker_ext(&"Binding FBO target for servo");
                    self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, framebuffer);
                    self.gl.draw_buffers(&[gl::COLOR_ATTACHMENT0]);
                    self.gl.disable(gl::CULL_FACE);
                    self.gl.depth_func(gl::LESS);

                    Ok(())
                }
                None => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

struct WinitEventLoopWaker {
    waker: EventsLoopProxy,
}

impl EventLoopWaker for WinitEventLoopWaker {
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(Self {
            waker: self.waker.clone(),
        })
    }
    fn wake(&self) {
        self.waker.wakeup().unwrap();
    }
}

impl WindowMethods for ServoWindow {
    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn set_animation_state(&self, _state: AnimationState) {}

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let coords = self.get_dimensions();
        EmbedderCoordinates {
            viewport: TypedRect::new(
                TypedPoint2D::new(0, 0),
                TypedSize2D::new(coords.0, coords.1),
            ),
            framebuffer: TypedSize2D::new(coords.0, coords.1),
            hidpi_factor: TypedScale::new(1.),
            screen: TypedSize2D::new(coords.0, coords.1),
            screen_avail: TypedSize2D::new(coords.0, coords.1),
            window: (
                TypedSize2D::new(coords.0, coords.1),
                TypedPoint2D::new(0, 0),
            ),
        }
    }

    fn prepare_for_composite(
        &self,
        _width: Length<u32, DevicePixel>,
        _height: Length<u32, DevicePixel>,
    ) -> bool {
        match self.enable_fb() {
            Ok(()) => {
                println!("Successfully bound framebuffer");
                true
            }
            Err(()) => {
                println!("Failed to enable framebuffer");
                false
            }
        }
    }

    fn present(&self) {
        println!("Unbound framebuffer");
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        Box::new(WinitEventLoopWaker {
            waker: self.waker.clone(),
        })
    }

    fn supports_clipboard(&self) -> bool {
        true
    }
}
