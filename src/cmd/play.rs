use std::{
    thread,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use crossbeam_channel::{bounded, tick, unbounded, Receiver, Sender};
// use indicatif::MultiProgress;
use rdev::{listen, simulate, EventType};

use crate::{
    cli::Play,
    keys::{Key, KeyState},
    session::Session,
};

use super::Run;

#[derive(Debug, Clone, Copy)]
enum FinishState {
    Finished,
    Aborted,
}

#[async_trait]
impl Run for Play {
    async fn run(self) -> eyre::Result<()> {
        let session = Session::from_file(self.output).unwrap();

        let stop_state = match self.stop_key {
            Some(s) => {
                let mut state = KeyState::default();
                for item in s.split(",") {
                    let key = Key::from_str(item)
                        .ok_or(eyre::eyre!("Unknown key '{}' for stop key", item))?;
                    state.set_pressed(key);
                }
                state
            }
            None => KeyState::with_pressed(&[Key::Escape]),
        };

        let total_iterations = self.iterations.unwrap_or(1);
        let delay = self
            .delay
            .map(|s| Duration::from_millis(s))
            .unwrap_or(Duration::ZERO);

        let (tt, rt) = bounded(1);
        let input_task = tokio::task::spawn(input_task(tt.clone(), stop_state));
        let executor_task = tokio::task::spawn(executor_task(tt, session, delay, total_iterations));

        let ticker: Receiver<Instant> = tick(std::time::Duration::from_millis(500));

        loop {
            crossbeam_channel::select! {
                recv(rt) -> finish_state=> {
                    match finish_state {
                        Ok(FinishState::Finished) => {
                            input_task.abort();
                            println!("finished");
                            break;
                        }
                        _ =>  {
                            input_task.abort();
                            executor_task.abort();
                            println!("aborted");
                            break;
                        }
                    }
                },
                recv(ticker) -> _ => {
                    println!("ticking");
                }
            };
        }

        println!("out of loop");

        // let (tt_ui, rt_ui) = (tt.clone(), rt.clone());
        // let ui = thread::spawn(move || {
        //     let (tt, rt) = (tt_ui, rt_ui);
        //     println!("ui thread started");
        //
        //     let ticker: Receiver<Instant> = tick(std::time::Duration::from_millis(500));
        //
        //     loop {
        //         crossbeam_channel::select! {
        //             recv(rt) -> _ => {
        //                 println!("term msg recv in ui thread");
        //                 tt.send(true)
        //                     .unwrap_or_else(|_| println!("Could not send terminate event"));
        //                 return;
        //             },
        //             recv(ticker) -> _ => {
        //
        //             }
        //         };
        //     }
        // });
        // ui.join();

        // executor.join();

        Ok(())
    }
}

async fn input_task(tx: Sender<FinishState>, stop_state: KeyState) -> eyre::Result<()> {
    let mut keystate = KeyState::default();
    listen(move |event| match event.event_type {
        EventType::KeyPress(k) => {
            keystate.set_pressed(k.into());
            if keystate.is_state_held(stop_state) {
                tx.send(FinishState::Aborted)
                    .unwrap_or_else(|_| println!("Could not send terminate event"));
                return;
            }
        }
        EventType::KeyRelease(k) => {
            keystate.set_released(k.into());
        }
        _ => {}
    })
    .expect("Could not listen");
    Ok(())
}

async fn executor_task(
    tx: Sender<FinishState>,
    session: Session,
    delay: Duration,
    total_iterations: u32,
) -> eyre::Result<()> {
    for i in 0..total_iterations {
        for event in &session.events {
            spin_sleep::sleep(event.delay);
            simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
        }
    }

    tx.send(FinishState::Finished)
        .unwrap_or_else(|_| println!("Could not send terminate event"));

    Ok(())
}
