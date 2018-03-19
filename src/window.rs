use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use libservo::compositing::compositor_thread::EventLoopWaker;
use libservo::{gl, BrowserId};
use libservo::compositing::windowing::{AnimationState, WindowMethods};
use glutin::GlWindow;
use libservo::euclid::{ Length, TypedPoint2D, TypedScale, TypedSize2D};
use libservo::webrender_api::{DeviceUintRect, DeviceUintSize};
use libservo::servo_geometry::DeviceIndependentPixel;
use libservo::style_traits::cursor::CursorKind;
use libservo::style_traits::DevicePixel;
use libservo::script_traits::LoadData;
use libservo::net_traits::net_error_list::NetError;
use libservo::servo_url::ServoUrl;
use libservo::msg::constellation_msg::{self, Key};
use libservo::ipc_channel::ipc::IpcSender;
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

    pub fn setup_framebuffer(&self) -> Result<(), u32> {
        let (width, height) = self.get_dimensions();
        let texture = self.get_target().unwrap();

        // Bind texture
        self.gl.bind_texture(gl::TEXTURE_2D, texture.into());

        // Create FBO
        let frame_buffer = self.gl.gen_framebuffers(1)[0];
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, frame_buffer);

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
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_buffer,
        );

        // Bind texture to framebuffer
        self.gl.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture.into(),
            0,
        );
        match self.gl.check_frame_buffer_status(gl::FRAMEBUFFER) {
            gl::FRAMEBUFFER_COMPLETE => match self.buffers.lock() {
                Ok(mut fb) => {
                    self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
                    self.gl.bind_renderbuffer(gl::RENDERBUFFER, 0);
                    self.gl.bind_texture(gl::TEXTURE_2D, 0);
                    *fb = Some((frame_buffer, depth_buffer));
                    Ok(())
                }
                Err(_) => {
                    self.gl.delete_framebuffers(&[frame_buffer]);
                    self.gl.delete_renderbuffers(&[depth_buffer]);
                    Err(0)
                }
            },
            e => {
                self.gl.delete_framebuffers(&[frame_buffer]);
                self.gl.delete_renderbuffers(&[depth_buffer]);
                Err(e)
            }
        }
    }

    pub fn enable_fb(&self) -> Result<(), ()> {
        match self.buffers.lock() {
            Ok(guard) => match *guard {
                Some((framebuffer, renderbuffer)) => {
                    self.gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
                    self.gl.bind_renderbuffer(gl::RENDERBUFFER, renderbuffer);
                    self.gl.draw_buffers(&[gl::COLOR_ATTACHMENT0]);
                    Ok(())
                }
                None => Err(()),
            },
            Err(_) => Err(()),
        }
    }

    pub fn disable_fb(&self) {
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
        self.gl.bind_renderbuffer(gl::RENDERBUFFER, 0);
        self.gl.draw_buffers(&[gl::BACK_LEFT]);
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

    fn framebuffer_size(&self) -> DeviceUintSize {
        let scale_factor = self.window.hidpi_factor() as u32;
        // TODO(ajeffrey): can this fail?
        let (width, height) = self.window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        DeviceUintSize::new(width, height) * scale_factor
    }

    fn window_rect(&self) -> DeviceUintRect {
        let size = self.framebuffer_size();
        let origin = TypedPoint2D::zero();
        DeviceUintRect::new(origin, size)
    }

    fn client_window(&self, _id: BrowserId) -> (TypedSize2D<u32, DevicePixel>, TypedPoint2D<i32, DevicePixel>) {
        // TODO(ajeffrey): can this fail?
        let (width, height) = self.window
            .get_outer_size()
            .expect("Failed to get window outer size.");
        let size = TypedSize2D::new(width, height);
        // TODO(ajeffrey): can this fail?
        let (x, y) = self.window
            .get_position()
            .expect("Failed to get window position.");
        let origin = TypedPoint2D::new(x as i32, y as i32);
        (size, origin)
    }

    fn screen_size(&self, _: BrowserId) -> TypedSize2D<u32, DevicePixel> {
        let dimensions = self.get_dimensions();
        let size = TypedSize2D::new(dimensions.0.into(), dimensions.1.into());
        size
    }

    fn screen_avail_size(&self, _: BrowserId) -> TypedSize2D<u32, DevicePixel> {
        let dimensions = self.get_dimensions();
        let size = TypedSize2D::new(dimensions.0.into(), dimensions.1.into());
        size
    }

    fn set_animation_state(&self, _state: AnimationState) {}

    fn set_inner_size(&self, _: BrowserId, _size: TypedSize2D<u32, DevicePixel>) {}

    fn set_position(&self, _: BrowserId, _point: TypedPoint2D<i32, DevicePixel>) {}

    fn set_fullscreen_state(&self, _: BrowserId, _state: bool) {}

    fn prepare_for_composite(&self, width: Length<u32, DevicePixel>, height: Length<u32, DevicePixel>) -> bool {
        println!("{:?} by {:?}", width, height);
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
        self.disable_fb();
        println!("Unbound framebuffer");
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        Box::new(WinitEventLoopWaker {
            waker: self.waker.clone(),
        })
    }

    fn set_page_title(&self, _: BrowserId, _title: Option<String>) {}

    fn status(&self, _: BrowserId, _status: Option<String>) {}

    fn load_start(&self, _: BrowserId) {}

    fn load_end(&self, _: BrowserId) {}

    fn history_changed(&self, _: BrowserId, _history: Vec<LoadData>, _current: usize) {}

    fn load_error(&self, _: BrowserId, _: NetError, _: String) {}

    fn head_parsed(&self, _: BrowserId) {}

    /// Has no effect on Android.
    fn set_cursor(&self, _cursor: CursorKind) {}

    fn set_favicon(&self, _: BrowserId, _: ServoUrl) {}

    /// Helper function to handle keyboard events.
    fn handle_key(
        &self,
        _: Option<BrowserId>,
        _ch: Option<char>,
        _key: Key,
        _mods: constellation_msg::KeyModifiers,
    ) {
    }

    fn allow_navigation(&self, _: BrowserId, _: ServoUrl, _response_chan: IpcSender<bool>) {}

    fn supports_clipboard(&self) -> bool {
        true
    }

    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        TypedScale::new(self.window.hidpi_factor())
    }

    fn handle_panic(&self, _: BrowserId, _reason: String, _backtrace: Option<String>) {
        // Nothing to do here yet. The crash has already been reported on the console.
    }
}
