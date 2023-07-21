use std::{sync::mpsc::channel, thread};

use rdev::{listen, simulate, EventType};

use crate::{
    cli::{Play, Run},
    event::RawEvent,
    keys::Key,
    session::Session,
};

impl Run for Play {
    fn run(self) -> eyre::Result<()> {
        let session = Session::from_file(self.output).unwrap();

        let stop_key = Key::F9;
        let check_stop_key: rdev::Key = stop_key.into();

        let (tx, rx) = channel();
        let _listener = thread::spawn(move || {
            listen(move |event| {
                if let EventType::KeyPress(key) = event.event_type {
                    if check_stop_key == key {
                        tx.send(RawEvent::Terminate)
                            .unwrap_or_else(|e| println!("Could not send event {:?}", e));
                        return;
                    }
                }
            })
        });

        let total_iterations = self.iterations.unwrap_or(1);

        let mut has_terminated = false;
        for i in 0..total_iterations {
            for event in &session.events {
                if let Ok(msg) = rx.try_recv() {
                    has_terminated = true;
                    break;
                }
                spin_sleep::sleep(event.delay);
                simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
            }

            if has_terminated {
                break;
            }
        }

        Ok(())
    }
}
