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
    let mut z = " ";
    let mut x = " ";
    let mut c = " ";
    let mut v = " ";
    let mut idx_ = String::from("");

    match idx.as_str() {
        "z" => {
            z = "x";
        },
        "x" => {
            x = "x";
        },
        "c" => {
            c = "x";
        },
        "v" => {
            v = "x";
        },
        _ => {
            idx_ = format!("{} ", idx).to_string();
        },
    }

    write!(
        stdout,
        "\r{}{}z[{}] x[{}] c[{}] v[{}]\r\n{}{}{}",
        termion::cursor::Up(1),
        termion::clear::CurrentLine,
        z,
        x,
        c,
        v,
        termion::clear::CurrentLine,
        idx_,
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
    let mut i = 0;
    let mut paths = Vec::new();
    let mut z: Option<&PathBuf> = None;
    let mut x: Option<&PathBuf> = None;
    let mut c: Option<&PathBuf> = None;
    let mut v: Option<&PathBuf> = None;

    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let file_path = file.path().to_path_buf();
            paths.push(file_path);
        }
    }

    if paths.len() == 0 {
        return;
    }

    // Audio backend
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();

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
                termion::event::Key::Char('z') => {
                    if z == None {
                        z = Some(&paths[i]);
                    }
                    play(z.unwrap(), &stream_handle, &mut stdout, &"z".to_string());
                },
                termion::event::Key::Char('Z') => {
                    z = Some(&paths[i]);
                    play(z.unwrap(), &stream_handle, &mut stdout, &"z".to_string());
                },
                termion::event::Key::Char('x') => {
                    if x == None {
                        x = Some(&paths[i]);
                    }
                    play(x.unwrap(), &stream_handle, &mut stdout, &"x".to_string());
                },
                termion::event::Key::Char('X') => {
                    x = Some(&paths[i]);
                    play(x.unwrap(), &stream_handle, &mut stdout, &"x".to_string());
                },
                termion::event::Key::Char('c') => {
                    if c == None {
                        c = Some(&paths[i]);
                    }
                    play(c.unwrap(), &stream_handle, &mut stdout, &"c".to_string());
                },
                termion::event::Key::Char('C') => {
                    c = Some(&paths[i]);
                    play(c.unwrap(), &stream_handle, &mut stdout, &"c".to_string());
                },
                termion::event::Key::Char('v') => {
                    if v == None {
                        v = Some(&paths[i]);
                    }
                    play(v.unwrap(), &stream_handle, &mut stdout, &"v".to_string());
                },
                termion::event::Key::Char('V') => {
                    v = Some(&paths[i]);
                    play(v.unwrap(), &stream_handle, &mut stdout, &"v".to_string());
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
        thread::sleep(time::Duration::from_micros(1000));
    }
    write!(stdout, "\r\n").unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide path to directory to traverse as first argument\n");
    } else {
        let path = &args[1];
        println!("Traversing {}\n", path);
        walk_and_play(path);
    }
}
