mod sound_loader;

use crossterm::cursor::{Hide, Show};
use crossterm::event::KeyCode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::{event, terminal, ExecutableCommand};
use invaders::frame;
use invaders::frame::Drawable;
use invaders::invaders::Invaders;
use invaders::player::Player;
use invaders::render::render;
use rusty_audio::Audio;
use sound_loader::load_sounds;
use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the audio system
    let mut audio = Audio::new();

    // Load sound effects from the specified directory
    load_sounds(&mut audio, "./sounds")?;

    // Play the startup sound to indicate the program has started
    audio.play("startup");

    // Initialise a terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = std::thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    'gameloop: loop {
        //curr frame init
        let mut curr_frame = frame::new_frame();
        let delta = instant.elapsed();
        instant = Instant::now();

        // input handling
        while event::poll(Duration::default())? {
            if let event::Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    _ => {}
                }
            }
        }

        // updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hit(&mut invaders) {
            audio.play("explode");
        }

        // render
        player.draw(&mut curr_frame);
        invaders.draw(&mut curr_frame);
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame).unwrap();

        // sleep
        sleep(Duration::from_millis(1));
        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    // Block execution until all sounds have finished playing
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(EnterAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
