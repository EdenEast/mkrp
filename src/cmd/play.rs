use std::{
    hint::spin_loop,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{bounded, select, tick, unbounded, Receiver};
use indicatif::{FormattedDuration, MultiProgress, ProgressBar, ProgressStyle};
use rdev::{listen, simulate, EventType};

use crate::{
    cli::{Play, Run},
    keys::{Key, KeyState},
    session::Session,
};

enum UiEvent {
    Iteration(u32),
    Event(u32),
    Aborted,
    Completed,
}

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
        let total_session = session.events.len();

        let total_duration =
            (session.total_time * total_iterations) + (delay * (total_iterations - 1));
        let total_formatted_duration = FormattedDuration(total_duration);
        let session_formatted_dutation = FormattedDuration(session.total_time);

        // Terminate channel
        let (tt, rt) = unbounded();
        // Execution channel
        let (tx, rx) = unbounded();

        let tt_input = tt.clone();
        let listener = thread::spawn(move || {
            let tt = tt_input;
            let mut keystate = KeyState::default();
            listen(move |event| match event.event_type {
                EventType::KeyPress(k) => {
                    keystate.set_pressed(k.into());
                    if keystate.is_state_held(stop_state) {
                        println!("sending terminate state");
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

        let executor = thread::spawn(move || {
            for current_iteration in 0..total_iterations {
                for (i, event) in session.events.iter().enumerate() {
                    spin_sleep::sleep(event.delay);
                    simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
                    tx.send(UiEvent::Event(i as u32 + 1))
                        .expect("failed to send event iteration to main ui thread");
                }

                if current_iteration < total_iterations - 1 {
                    spin_sleep::sleep(delay);
                }

                tx.send(UiEvent::Iteration(current_iteration + 1))
                    .expect("failed to send iteration event to main ui thread");
            }

            tx.send(UiEvent::Completed)
                .expect("failed to send Completed event to ui thread");
        });

        let mut current_iteration = 1;
        let mut current_event = 1;
        let zero_duration = FormattedDuration(Duration::ZERO);

        // Progressbar setup
        let style =
            ProgressStyle::with_template("{prefix} {wide_bar} {msg} {percent:>3}%").unwrap();
        let mp = MultiProgress::new();
        let tpb = mp.add(
            ProgressBar::new(total_duration.as_secs())
                .with_style(style.clone())
                .with_prefix("Total  ")
                .with_message(format!("{} / {}", zero_duration, total_formatted_duration)),
        );
        let spb = mp.add(
            ProgressBar::new(session.total_time.as_secs())
                .with_style(style.clone())
                .with_prefix("Session")
                .with_message(format!(
                    "{} / {}",
                    zero_duration, session_formatted_dutation
                )),
        );

        let ticker = tick(Duration::from_secs(1));
        let initial_start = Instant::now();
        let mut session_start = Instant::now();
        let total_formatted_duration = FormattedDuration(total_duration);
        let mut current_total = 1;
        let mut current_event = 1;

        loop {
            if let Ok(_) = rt.try_recv() {
                break;
            }

            if let Ok(event) = rx.try_recv() {
                match event {
                    UiEvent::Iteration(n) => {
                        current_total = n + 1;
                        spb.set_position(0);
                        session_start = Instant::now();
                    }
                    UiEvent::Event(n) => {
                        current_event = n + 1;
                    }
                    UiEvent::Aborted => break,
                    UiEvent::Completed => break,
                }
            }

            if let Ok(tick) = ticker.try_recv() {
                let now = Instant::now();
                tpb.inc(1);
                spb.inc(1);

                tpb.set_message(format!(
                    "({:>5}/{:>5}) {} / {}",
                    current_total,
                    total_iterations,
                    FormattedDuration(now.saturating_duration_since(initial_start)),
                    total_formatted_duration
                ));
                spb.set_message(format!(
                    "({:>5}/{:>5}) {} / {}",
                    current_event,
                    total_session,
                    FormattedDuration(now.saturating_duration_since(session_start)),
                    session_formatted_dutation
                ));
            }
        }

        tpb.finish_and_clear();
        spb.finish_and_clear();
        mp.clear();

        println!("bye");

        // let mut has_terminated = false;
        // for i in 0..total_iterations {
        //     spb.set_position(0);
        //     for event in &session.events {
        //         if let Ok(msg) = rt.try_recv() {
        //             println!("term msg recv");
        //             has_terminated = true;
        //             break;
        //         }
        //
        //         spin_sleep::sleep(event.delay);
        //         simulate(&event.event).expect(&format!("failed to simulate {:#?}", event));
        //
        //         spb.inc(1);
        //     }
        //
        //     if has_terminated {
        //         break;
        //     }
        //
        //     // No need if we are at the final iteration
        //     if i < total_iterations - 1 {
        //         spin_sleep::sleep(delay);
        //     }
        //
        //     tpb.inc(1);
        // }
        //
        // let ticker = tick(Duration::from_millis(500));
        //
        // tpb.finish_and_clear();
        // spb.finish_and_clear();
        // mp.clear();

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
