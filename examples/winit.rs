use std::{
    fmt::Debug,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
    time::Duration,
};

use channel_protocol::channel_protocol;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::KeyEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey, SmolStr},
    platform::windows::EventLoopBuilderExtWindows,
    window::{Window, WindowAttributes},
};

#[channel_protocol]
trait WinitInputProtocol {
    fn create_window(title: String, width: u32, height: u32);
    fn is_window_open() -> bool;
    fn close_window();
    fn set_title(title: String);
    fn resize(width: u32, height: u32);
    fn teardown();
}

#[channel_protocol]
trait WinitOutputProtocol {
    fn on_window_resized(width: u32, height: u32);
    fn on_key_event(key: KeyCode, is_pressed: bool);
    fn on_text(text: SmolStr);
    fn on_close_request();
}

struct WinitApp {
    window: Option<Window>,
    output_client: WinitOutputProtocolClient,
}

impl HandleWinitInputProtocol<&ActiveEventLoop> for WinitApp {
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: String,
        width: u32,
        height: u32,
    ) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_inner_size(PhysicalSize::new(width, height))
                    .with_title(title),
            )
            .unwrap();

        self.window = Some(window);
    }

    fn is_window_open(&mut self, _: &ActiveEventLoop) -> bool {
        todo!()
    }

    fn close_window(&mut self, _: &ActiveEventLoop) {
        self.window.take();
    }

    fn set_title(&mut self, _: &ActiveEventLoop, title: String) {
        if let Some(window) = &self.window {
            window.set_title(&title);
        }
    }

    fn resize(&mut self, _: &ActiveEventLoop, width: u32, height: u32) {
        if let Some(window) = &self.window {
            let _ = window.request_inner_size(PhysicalSize::new(width, height));
        }
    }

    fn teardown(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.exit();
    }
}

impl ApplicationHandler<WinitInputProtocolMessage> for WinitApp {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = None;
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: WinitInputProtocolMessage,
    ) {
        println!("{event:?}");
        self.dispatch(event_loop, event);
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::Resized(physical_size) => {
                self.output_client
                    .on_window_resized(physical_size.width, physical_size.height);
            }
            winit::event::WindowEvent::CloseRequested => {
                self.output_client.on_close_request();
            }
            winit::event::WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        text,
                        state,
                        ..
                    },
                ..
            } => {
                if let Some(text) = text {
                    self.output_client.on_text(text);
                }

                if let PhysicalKey::Code(key) = physical_key {
                    let is_pressed = matches!(state, winit::event::ElementState::Pressed);
                    self.output_client.on_key_event(key, is_pressed);
                }
            }
            _ => {}
        }
    }
}

fn launch_winit_on_other_thread() -> (
    JoinHandle<()>,
    WinitInputProtocolClient,
    Receiver<WinitOutputProtocolMessage>,
) {
    let (input_client, input_rx) = WinitInputProtocolClient::new();
    let (output_client, output_rx) = WinitOutputProtocolClient::new();

    let winit_thread = thread::spawn(|| {
        let event_loop = EventLoop::with_user_event()
            .with_any_thread(true)
            .build()
            .unwrap();

        let event_sender = event_loop.create_proxy();

        thread::spawn(move || {
            for input_event in input_rx {
                event_sender.send_event(input_event).unwrap();
            }
        });

        event_loop
            .run_app(&mut WinitApp {
                window: None,
                output_client,
            })
            .unwrap();
    });

    (winit_thread, input_client, output_rx)
}

fn main() {
    enum Mode {
        TitleEditing(String),
        Normal,
    }

    let (winit_thread, winit_client, output_rx) = launch_winit_on_other_thread();

    winit_client.create_window("Test window".into(), 600, 600);

    let mut width = 600;
    let mut height = 600;

    let mut mode = Mode::Normal;

    for message in output_rx {
        match message {
            WinitOutputProtocolMessage::OnWindowResized(OnWindowResizedParamMessage {
                width: new_width,
                height: new_height,
            }) => {
                width = new_width;
                height = new_height;
            }
            WinitOutputProtocolMessage::OnKeyEvent(OnKeyEventParamMessage { key, is_pressed }) => {
                if let Mode::TitleEditing(_) = mode {
                    // In title editing mode, ignore all key events except Enter
                    if key != KeyCode::Enter {
                        continue;
                    }
                }
                if is_pressed {
                    match key {
                        KeyCode::Escape => {
                            println!(
                                "Closing current window and respawning a new one 2 secs later"
                            );
                            winit_client.close_window();
                            let winit_client_clone = winit_client.clone();
                            thread::spawn(move || {
                                thread::sleep(Duration::from_secs(2));
                                winit_client_clone.create_window(
                                    "Respawed window".into(),
                                    600,
                                    600,
                                );
                            });
                        }
                        KeyCode::ArrowLeft => {
                            winit_client.resize(width - 100, height);
                        }
                        KeyCode::ArrowRight => {
                            winit_client.resize(width + 100, height);
                        }
                        KeyCode::ArrowUp => {
                            winit_client.resize(width, height - 100);
                        }
                        KeyCode::ArrowDown => {
                            winit_client.resize(width, height + 100);
                        }
                        KeyCode::Enter => match mode {
                            Mode::Normal => {
                                mode = Mode::TitleEditing(String::new());
                                winit_client.set_title("Editing title...".into());
                            }
                            Mode::TitleEditing(new_title) => {
                                mode = Mode::Normal;
                                winit_client.set_title(new_title);
                            }
                        },
                        _ => {}
                    }
                }
            }
            WinitOutputProtocolMessage::OnCloseRequest => {
                winit_client.teardown();
            }
            WinitOutputProtocolMessage::OnText(OnTextParamMessage { text }) => {
                if let Mode::TitleEditing(ref mut current_title) = mode {
                    current_title.push_str(&text);
                    winit_client.set_title(current_title.clone());
                }
            }
        }
    }

    winit_thread.join().unwrap();
}
