use std::{
    hint::spin_loop,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{bounded, tick, unbounded, Receiver};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use rdev::{listen, simulate, EventType};

use crate::{
    cli::{Play, Run},
    keys::{Key, KeyState},
    session::Session,
};

impl Run for Play {
    fn run(self) -> eyre::Result<()> {
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

        let total_time = (session.total_time * total_iterations) + (delay * (total_iterations - 1));

        // Dump information on play
        println!("Session run time: {}", HumanDuration(session.total_time));
        println!("Total run time:   {}", HumanDuration(total_time));

        let style =
            ProgressStyle::with_template("{prefix} {wide_bar} ({pos:^5}/{len:^5}) {percent:>3}%")
                .unwrap();
        let mp = MultiProgress::new();
        let tpb = mp.add(ProgressBar::new(total_iterations as u64));
        let spb = mp.add(ProgressBar::new(session.events.len() as u64));
        tpb.set_prefix("Total  ");
        spb.set_prefix("Session");
        tpb.set_style(style.clone());
        spb.set_style(style);

        let (tt, rt) = unbounded();

        let tt_input = tt.clone();
        let listener = thread::spawn(move || {
            let tt = tt_input;
            let mut keystate = KeyState::default();
            listen(move |event| match event.event_type {
                EventType::KeyPress(k) => {
                    keystate.set_pressed(k.into());
                    if keystate.is_state_held(stop_state) {
                        tt.send(true)
                            .unwrap_or_else(|_| println!("Could not send terminate event"));
                    }
                }
                EventType::KeyRelease(k) => {
                    keystate.set_released(k.into());
                }
                _ => {}
            })
            .expect("Could not listen");
        });

        let mut has_terminated = false;
        tpb.tick();
        spb.tick();
        for i in 0..total_iterations {
            spb.set_position(0);
            for event in &session.events {
                if let Ok(msg) = rt.try_recv() {
                    println!("term msg recv");
                    has_terminated = true;
                    break;
                }

                spin_sleep::sleep(event.delay);
                simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));

                spb.inc(1);
            }

            if has_terminated {
                break;
            }

            // No need if we are at the final iteration
            if i < total_iterations - 1 {
                spin_sleep::sleep(delay);
            }

            tpb.inc(1);
        }

        tpb.finish_and_clear();
        spb.finish_and_clear();
        mp.clear();

        // let (tt_exec, rt_exec) = (tt.clone(), rt.clone());
        // let executor = thread::spawn(move || {
        //     let (tt, rt) = (tt_exec, rt_exec);
        //     println!("executor started");
        //     for i in 0..total_iterations {
        //         // println!("Iterations {}", i);
        //         // for event in &session.events {
        //             if let Ok(msg) = rt.try_recv() {
        //                 println!("term msg recv in executor thread");
        //                 tt.send(true)
        //                     .unwrap_or_else(|_| println!("Could not send terminate event"));
        //                 return;
        //             }
        //             spin_sleep::sleep(event.delay);
        //             simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
        //         }
        //
        //         // No need if we are at the final iteration
        //         if i < total_iterations - 1 {
        //             spin_sleep::sleep(delay);
        //         }
        //     }
        //
        //     tt.send(true)
        //         .expect("failed to send terminate signal from exec thread");
        //     println!("executor thread finished");
        // });

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
