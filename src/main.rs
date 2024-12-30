use jwalk::WalkDir;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use rodio::Sink;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read, Stdout, Write};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process::ExitCode;
use termion::raw::{IntoRawMode, RawTerminal};

const NUM_SINKS: usize = 16;
const STDIN_CTRL_C: u8 = 3;
const STDIN_CTRL_D: u8 = 4;

struct ProgrammedKey {
    key: String,
    idx: String,
    path: Option<PathBuf>,
}

struct State {
    i: usize,
    paths: Vec<PathBuf>,
    z: ProgrammedKey,
    x: ProgrammedKey,
    c: ProgrammedKey,
    v: ProgrammedKey,
    sink_idx: usize,
    sinks: Vec<Sink>,
    stdout: RawTerminal<Stdout>,
}

fn resolve_key(key: termion::event::Key, state: &mut State) -> bool {
    match key {
        termion::event::Key::Char('j') | termion::event::Key::Char('i') => {
            if state.i == state.paths.len() - 1 {
                state.i = 0;
            } else {
                state.i += 1;
            }
            handle_key_press(
                Some(state.paths[state.i].clone()),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &make_idx(state.i, state.paths.len()),
                None,
            );
        }
        termion::event::Key::Char('k') | termion::event::Key::Char('o') => {
            if state.i == 0 {
                state.i = state.paths.len() - 1;
            } else {
                state.i -= 1;
            }
            handle_key_press(
                Some(state.paths[state.i].clone()),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &make_idx(state.i, state.paths.len()),
                None,
            );
        }
        termion::event::Key::Char('z') => match &state.z {
            z if !z.path.is_none() => {
                handle_key_press(
                    z.path.clone(),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &"".to_string(),
                    Some(&z),
                );
            }
            _ => {
                handle_key_press(
                    Some(state.paths[state.i].clone()),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &make_idx(state.i, state.paths.len()),
                    Some(&state.z),
                );
            }
        },
        termion::event::Key::Char('Z') => {
            state.z.key = "z".to_string();
            state.z.idx = make_idx(state.i, state.paths.len());
            state.z.path = Some(state.paths[state.i].clone());
            handle_key_press(
                state.z.path.clone(),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &"".to_string(),
                Some(&state.z),
            );
        }
        termion::event::Key::Char('x') => match &state.x {
            x if !x.path.is_none() => {
                handle_key_press(
                    x.path.clone(),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &"".to_string(),
                    Some(&x),
                );
            }
            _ => {
                handle_key_press(
                    Some(state.paths[state.i].clone()),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &make_idx(state.i, state.paths.len()),
                    Some(&state.x),
                );
            }
        },
        termion::event::Key::Char('X') => {
            state.x.key = "x".to_string();
            state.x.idx = make_idx(state.i, state.paths.len());
            state.x.path = Some(state.paths[state.i].clone());
            handle_key_press(
                state.x.path.clone(),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &"".to_string(),
                Some(&state.x),
            );
        }
        termion::event::Key::Char('c') => match &state.c {
            c if !c.path.is_none() => {
                handle_key_press(
                    c.path.clone(),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &"".to_string(),
                    Some(&c),
                );
            }
            _ => {
                handle_key_press(
                    Some(state.paths[state.i].clone()),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &make_idx(state.i, state.paths.len()),
                    Some(&state.c),
                );
            }
        },
        termion::event::Key::Char('C') => {
            state.c.key = "c".to_string();
            state.c.idx = make_idx(state.i, state.paths.len());
            state.c.path = Some(state.paths[state.i].clone());
            handle_key_press(
                state.c.path.clone(),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &"".to_string(),
                Some(&state.c),
            );
        }
        termion::event::Key::Char('v') => match &state.v {
            v if !v.path.is_none() => {
                handle_key_press(
                    v.path.clone(),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &"".to_string(),
                    Some(&v),
                );
            }
            _ => {
                handle_key_press(
                    Some(state.paths[state.i].clone()),
                    &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                    &mut state.stdout,
                    &make_idx(state.i, state.paths.len()),
                    Some(&state.v),
                );
            }
        },
        termion::event::Key::Char('V') => {
            state.v.key = "v".to_string();
            state.v.idx = make_idx(state.i, state.paths.len());
            state.v.path = Some(state.paths[state.i].clone());
            handle_key_press(
                state.v.path.clone(),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &"".to_string(),
                Some(&state.v),
            );
        }
        termion::event::Key::Char('s') => {
            stop_all(&state.sinks);
        }
        termion::event::Key::Char('q') => {
            return true;
        }
        termion::event::Key::Ctrl('c') => {
            return true;
        }
        _ => {
            handle_key_press(
                Some(state.paths[state.i].clone()),
                &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
                &mut state.stdout,
                &make_idx(state.i, state.paths.len()),
                None,
            );
            state.stdout.lock().flush().unwrap();
        }
    }

    return false;
}

fn handle_key_press(
    path: Option<PathBuf>,
    sink: &Sink,
    stdout: &mut RawTerminal<Stdout>,
    idx: &String,
    programmed_key: Option<&ProgrammedKey>,
) {
    let mut z = " ";
    let mut x = " ";
    let mut c = " ";
    let mut v = " ";
    let mut idx_ = String::from("");
    let mut noplay = false;

    match programmed_key {
        Some(programmed_key) => match programmed_key.key.as_str() {
            "z" => {
                z = "x";
                idx_ = format!("{} ", programmed_key.idx).to_string();
            }
            "z?" => {
                z = "?";
                noplay = true;
                idx_ = format!("{} ", idx).to_string();
            }
            "x" => {
                x = "x";
                idx_ = format!("{} ", programmed_key.idx).to_string();
            }
            "x?" => {
                x = "?";
                noplay = true;
                idx_ = format!("{} ", idx).to_string();
            }
            "c" => {
                c = "x";
                idx_ = format!("{} ", programmed_key.idx).to_string();
            }
            "c?" => {
                c = "?";
                noplay = true;
                idx_ = format!("{} ", idx).to_string();
            }
            "v" => {
                v = "x";
                idx_ = format!("{} ", programmed_key.idx).to_string();
            }
            "v?" => {
                v = "?";
                noplay = true;
                idx_ = format!("{} ", idx).to_string();
            }
            _ => (),
        },
        _ => {
            idx_ = format!("{} ", idx).to_string();
        }
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
        path.clone().unwrap().display().to_string(),
    )
    .unwrap();
    stdout.lock().flush().unwrap();

    if noplay {
        return;
    }

    // Stop whatever may be already playing
    sink.stop();

    if let Ok(sound_file) = File::open(path.clone().unwrap().display().to_string()) {
        if let Ok(source) = rodio::Decoder::new(BufReader::new(sound_file)) {
            sink.append(source);
        }
    }
}

fn make_idx(i: usize, max: usize) -> String {
    format!("[{}/{}]", i + 1, max)
}

fn postincrement(x: &mut usize, max: usize) -> usize {
    let ret = *x;
    if *x < (max - 1) {
        *x += 1;
    } else {
        *x = 0;
    }
    ret
}

fn stop_all(sinks: &Vec<Sink>) {
    for sink in sinks {
        sink.stop();
    }
}

fn walk_and_play(path: &String) {
    let mut state: State = State {
        i: 0,
        paths: Vec::new(),
        z: ProgrammedKey {
            key: "z?".to_string(),
            idx: "".to_string(),
            path: None,
        },
        x: ProgrammedKey {
            key: "x?".to_string(),
            idx: "".to_string(),
            path: None,
        },
        c: ProgrammedKey {
            key: "c?".to_string(),
            idx: "".to_string(),
            path: None,
        },
        v: ProgrammedKey {
            key: "v?".to_string(),
            idx: "".to_string(),
            path: None,
        },
        sink_idx: 0,
        sinks: Vec::new(),
        // Set terminal to raw mode to allow reading stdin one key at a time
        stdout: io::stdout().into_raw_mode().unwrap(),
    };

    for file in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if file.metadata().unwrap().is_file() {
            let file_path = file.path().to_path_buf();
            state.paths.push(file_path);
        }
    }

    if state.paths.len() == 0 {
        return;
    }

    // Audio backend
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    // Create a list of Sinks to be used in a round-robin fashion
    for _ in 0..NUM_SINKS {
        let sink = Sink::try_new(&stream_handle).unwrap();
        state.sinks.push(sink);
    }

    // Set up polling stdin for readiness. This allows for a blocking wait for key presses instead
    // of a busy wait.
    let stdin_os = io::stdin();
    let mut stdin_lock = stdin_os.lock();
    let stdin_fd = stdin_lock.as_raw_fd();
    let mut stdin_source = SourceFd(&stdin_fd);
    const STDIN_TOKEN: Token = Token(0);
    let mut poll = Poll::new().unwrap();
    poll.registry()
        .register(&mut stdin_source, STDIN_TOKEN, Interest::READABLE)
        .unwrap();
    let mut events = Events::with_capacity(128);
    let mut buffer = [0; 1024];

    // Play the first discovered sound file on start up
    handle_key_press(
        Some(state.paths[state.i].clone()),
        &state.sinks[postincrement(&mut state.sink_idx, NUM_SINKS)],
        &mut state.stdout,
        &make_idx(state.i, state.paths.len()),
        None,
    );

    'outer: loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                STDIN_TOKEN => {
                    if event.is_readable() {
                        match stdin_lock.read(&mut buffer) {
                            Ok(n) => {
                                for i in 0..n {
                                    let key;
                                    match buffer[i] {
                                        STDIN_CTRL_C => {
                                            key = termion::event::Key::Ctrl('c');
                                        }
                                        STDIN_CTRL_D => {
                                            key = termion::event::Key::Ctrl('d');
                                        }
                                        _ => {
                                            key = termion::event::Key::Char(buffer[i] as char);
                                        }
                                    }
                                    if resolve_key(key, &mut state) {
                                        break 'outer;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("\r{}", e);
                                break 'outer;
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    write!(state.stdout, "\r\n").unwrap();

    // Dump any programmed key sound file paths to stdout
    match &state.z {
        z if !z.path.is_none() => {
            write!(
                state.stdout,
                "z: {}\r\n",
                z.path.clone().unwrap().display().to_string()
            )
            .unwrap();
        }
        _ => (),
    }
    match &state.x {
        x if !x.path.is_none() => {
            write!(
                state.stdout,
                "x: {}\r\n",
                x.path.clone().unwrap().display().to_string()
            )
            .unwrap();
        }
        _ => (),
    }
    match &state.c {
        c if !c.path.is_none() => {
            write!(
                state.stdout,
                "c: {}\r\n",
                c.path.clone().unwrap().display().to_string()
            )
            .unwrap();
        }
        _ => (),
    }
    match &state.v {
        v if !v.path.is_none() => {
            write!(
                state.stdout,
                "v: {}\r\n",
                v.path.clone().unwrap().display().to_string()
            )
            .unwrap();
        }
        _ => (),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide path to directory to traverse as first argument\n");
        return ExitCode::from(1);
    }

    let path = &args[1];
    println!("Traversing {}\n", path);
    walk_and_play(path);

    return ExitCode::from(0);
}
