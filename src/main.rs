use std::fs::File;
use std::io;
use std::io::{Write, BufReader, Stdout};
use std::path::PathBuf;
use std::{thread, time};
use jwalk::WalkDir;
use rodio::{Source, OutputStreamHandle};
use std::env;
use termion;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

fn play(path: &PathBuf, stream_handle: &OutputStreamHandle, stdout: &mut RawTerminal<Stdout>, idx: &String) {
    write!(
        stdout,
        "\r{}{} {}",
        termion::clear::CurrentLine,
        idx,
        path.display().to_string(),
        ).unwrap();
    stdout.lock().flush().unwrap();
    if let Ok(sound_file) = File::open(path.display().to_string()) {
        if let Ok(source) = rodio::Decoder::new(BufReader::new(sound_file)) {
            stream_handle.play_raw(source.convert_samples()).ok();
        }
    }
}

fn make_idx(i: usize, max: usize) -> String {
    format!("[{}/{}]", i+1, max)
}

fn walk_and_play(path: &String) {
    // Audio backend
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();

    let mut paths = Vec::new();

    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let file_path = file.path().to_path_buf();
            paths.push(file_path);
        }
    }

    let mut i = 0;
    if paths.len() == 0 {
        return;
    }

    play(&paths[i], &stream_handle, &mut stdout, &make_idx(i, paths.len()));

    'outer: loop {
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                termion::event::Key::Char('j') | termion::event::Key::Char('i') => {
                    if i == paths.len() - 1 {
                        i = 0;
                    } else {
                        i += 1;
                    }
                    play(&paths[i], &stream_handle, &mut stdout, &make_idx(i, paths.len()));
                },
                termion::event::Key::Char('k') | termion::event::Key::Char('o') => {
                    if i == 0 {
                        i = paths.len() - 1;
                    } else {
                        i -= 1;
                    }
                    play(&paths[i], &stream_handle, &mut stdout, &make_idx(i, paths.len()));
                },
                termion::event::Key::Char('q') => {
                    break 'outer;
                },
                _ => {
                    play(&paths[i], &stream_handle, &mut stdout, &make_idx(i, paths.len()));
                    stdout.lock().flush().unwrap();
                }
            }
        }
        thread::sleep(time::Duration::from_millis(1));
    }
    write!(stdout, "\r\n").unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide path to directory to traverse as first argument\n");
    } else {
        let path = &args[1];
        println!("Traversing {}", path);
        walk_and_play(path);
    }
}
