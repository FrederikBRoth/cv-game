use log::warn;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::core::state::State;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

pub enum UserEvent {
    StateEvent(State),
    ScrollPosition { x: f64, y: f64 },
}
// #[derive(Default)]
pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
    state: Option<State>,
    last_time: web_time::Instant,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<UserEvent>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
            last_time: web_time::Instant::now(),
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        // Create window object
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(proxy) = self.proxy.take() {
                let proxy_clone = proxy.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let state = State::new(window).await;
                    assert!(proxy_clone
                        .send_event(
                            UserEvent::StateEvent(state) // .expect("Unable to create canvas!!!")
                        )
                        .is_ok())
                });

                setup_scroll_listener(proxy);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = pollster::block_on(State::new(window.clone()));
            self.state = Some(state);
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: UserEvent) {
        match event {
            UserEvent::StateEvent(mut state) => {
                #[cfg(target_arch = "wasm32")]
                {
                    state.window.request_redraw();
                    state.resize(state.window.inner_size());
                }
                self.state = Some(state);
                warn!("test")
            }
            #[cfg(not(target_arch = "wasm32"))]
            _ => {}
            #[cfg(target_arch = "wasm32")]
            UserEvent::ScrollPosition { x, y } => {
                if let Some(state) = &mut self.state {
                    state.update_scroll(y as i64);
                }
            }
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        state.input(&event);
        // println!("{event:?}");
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = self.last_time.elapsed();
                self.last_time = web_time::Instant::now();
                state.update(dt);
                state.render().unwrap();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            state.update_scroll(state.scroll_y - y as i64 * 50);
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
}
#[cfg(target_arch = "wasm32")]
fn setup_scroll_listener(proxy: winit::event_loop::EventLoopProxy<UserEvent>) {
    let window = wgpu::web_sys::window().unwrap();
    let window_clone = window.clone();

    let closure = Closure::<dyn FnMut(_)>::new(move |_event: wgpu::web_sys::Event| {
        let x = window_clone.scroll_x().unwrap_or(0.0);
        let y = window_clone.scroll_y().unwrap_or(0.0);

        // Send a custom event with the scroll data to your app
        let _ = proxy.send_event(UserEvent::ScrollPosition { x, y });
    });

    window
        .add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget();
}
pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
