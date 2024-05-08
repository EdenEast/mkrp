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
                for item in s.split(',') {
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
            .map(Duration::from_millis)
            .unwrap_or(Duration::ZERO);
        let total_session = session.events.len();

        let total_duration =
            (session.total_time * total_iterations) + (delay * (total_iterations - 1));
        let total_formatted_duration = FormattedDuration(total_duration);
        let session_duration = session.total_time;
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
                    simulate(&event.event)
                        .unwrap_or_else(|_| panic!("failed to simulate {:#?}", event));
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
            if rt.try_recv().is_ok() {
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

                let cur_total_duration = now.saturating_duration_since(initial_start);
                let cur_total_eta = total_duration.saturating_sub(cur_total_duration);

                let cur_session_duration = now.saturating_duration_since(session_start);
                let cur_session_eta = session_duration.saturating_sub(cur_session_duration);

                tpb.set_message(format!(
                    "({:>5}/{:>5}) [{} / {}] ({})",
                    current_total,
                    total_iterations,
                    FormattedDuration(cur_total_duration),
                    total_formatted_duration,
                    FormattedDuration(cur_total_eta)
                ));
                spb.set_message(format!(
                    "({:>5}/{:>5}) [{} / {}] ({})",
                    current_event,
                    total_session,
                    FormattedDuration(now.saturating_duration_since(session_start)),
                    session_formatted_dutation,
                    FormattedDuration(cur_session_eta)
                ));
            }
        }

        tpb.finish_and_clear();
        spb.finish_and_clear();
        mp.clear();

        println!("bye");

        Ok(())
    }
}
