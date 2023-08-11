use std::{thread::JoinHandle, time::Duration};

use crossbeam_channel::{select, tick, unbounded, Receiver, Sender};
use device_query::{device_state, DeviceQuery, DeviceState, MouseState};
use eframe::egui;
use rdev::{Event as RdEvent, ListenError};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}

fn make_listener_thread(tx: Sender<RdEvent>) -> JoinHandle<()> {
    let handle = std::thread::spawn(move || {
        rdev::listen(move |event| {
            tx.send(event)
                .unwrap_or_else(|e| println!("Could not send event {:?}", e));
        })
        .expect("Could not listen");
    });

    handle
}

fn make_executor_thead(rx: Receiver<RdEvent>) -> JoinHandle<()> {
    let handle = std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            rdev::simulate(&event.event_type).expect(&format!("failed to simulate {:#?}", event));
        }
    });

    handle
}

struct App {
    device_state: DeviceState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            device_state: DeviceState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mouse_state = self.device_state.get_mouse();
        let (x, y) = mouse_state.coords;
        let pointer = ctx.input(|i| i.pointer.hover_pos().unwrap_or_default());
        let (lx, ly) = (pointer.x, pointer.y);
        let mut record_string = String::new();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Record");
                ui.text_edit_singleline(&mut record_string);
            });
            // ui.horizontal(|ui| ui.label(format!("global coords x: {}, y: {}", x, y)));
            // ui.horizontal(|ui| ui.label(format!("local coords x: {}, y: {}", lx, ly)));
        });
        ctx.request_repaint();
    }
}
