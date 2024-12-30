use jwalk::WalkDir;
use rodio::Sink;
use std::env;
use std::fs::File;
use std::process::ExitCode;
use std::io;
use std::io::{BufReader, Stdout, Write};
use std::path::PathBuf;
use std::{thread, time};
use termion;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

const NUM_SINKS: usize = 16;

struct ProgrammedKey<'a> {
    key: String,
    idx: String,
    path: Option<&'a PathBuf>,
}

fn handle_key_press(
    path: Option<&PathBuf>,
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
        path.unwrap().display().to_string(),
    )
    .unwrap();
    stdout.lock().flush().unwrap();

    if noplay {
        return;
    }

    // Stop whatever may be already playing
    sink.stop();

    if let Ok(sound_file) = File::open(path.unwrap().display().to_string()) {
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
    let mut i: usize = 0;
    let mut sink_idx: usize = 0;
    let mut paths = Vec::new();
    let mut z: ProgrammedKey = ProgrammedKey {
        key: "z?".to_string(),
        idx: "".to_string(),
        path: None,
    };
    let mut x: ProgrammedKey = ProgrammedKey {
        key: "x?".to_string(),
        idx: "".to_string(),
        path: None,
    };
    let mut c: ProgrammedKey = ProgrammedKey {
        key: "c?".to_string(),
        idx: "".to_string(),
        path: None,
    };
    let mut v: ProgrammedKey = ProgrammedKey {
        key: "v?".to_string(),
        idx: "".to_string(),
        path: None,
    };

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

    // Create a list of Sinks to be used in a round-robin fashion
    let mut sinks: Vec<Sink> = vec![];
    for _ in 0..NUM_SINKS {
        let sink = Sink::try_new(&stream_handle).unwrap();
        sinks.push(sink);
    }

    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();

    handle_key_press(
        Some(&paths[i]),
        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
        &mut stdout,
        &make_idx(i, paths.len()),
        None,
    );

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
                    handle_key_press(
                        Some(&paths[i]),
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &make_idx(i, paths.len()),
                        None,
                    );
                }
                termion::event::Key::Char('k') | termion::event::Key::Char('o') => {
                    if i == 0 {
                        i = paths.len() - 1;
                    } else {
                        i -= 1;
                    }
                    handle_key_press(
                        Some(&paths[i]),
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &make_idx(i, paths.len()),
                        None,
                    );
                }
                termion::event::Key::Char('z') => match &z {
                    z if !z.path.is_none() => {
                        handle_key_press(
                            z.path,
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &"".to_string(),
                            Some(&z),
                        );
                    }
                    _ => {
                        handle_key_press(
                            Some(&paths[i]),
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &make_idx(i, paths.len()),
                            Some(&z),
                        );
                    }
                },
                termion::event::Key::Char('Z') => {
                    z.key = "z".to_string();
                    z.idx = make_idx(i, paths.len());
                    z.path = Some(&paths[i]);
                    handle_key_press(
                        z.path,
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &"".to_string(),
                        Some(&z),
                    );
                }
                termion::event::Key::Char('x') => match &x {
                    x if !x.path.is_none() => {
                        handle_key_press(
                            x.path,
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &"".to_string(),
                            Some(&x),
                        );
                    }
                    _ => {
                        handle_key_press(
                            Some(&paths[i]),
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &make_idx(i, paths.len()),
                            Some(&x),
                        );
                    }
                },
                termion::event::Key::Char('X') => {
                    x.key = "x".to_string();
                    x.idx = make_idx(i, paths.len());
                    x.path = Some(&paths[i]);
                    handle_key_press(
                        x.path,
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &"".to_string(),
                        Some(&x),
                    );
                }
                termion::event::Key::Char('c') => match &c {
                    c if !c.path.is_none() => {
                        handle_key_press(
                            c.path,
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &"".to_string(),
                            Some(&c),
                        );
                    }
                    _ => {
                        handle_key_press(
                            Some(&paths[i]),
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &make_idx(i, paths.len()),
                            Some(&c),
                        );
                    }
                },
                termion::event::Key::Char('C') => {
                    c.key = "c".to_string();
                    c.idx = make_idx(i, paths.len());
                    c.path = Some(&paths[i]);
                    handle_key_press(
                        c.path,
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &"".to_string(),
                        Some(&c),
                    );
                }
                termion::event::Key::Char('v') => match &v {
                    v if !v.path.is_none() => {
                        handle_key_press(
                            v.path,
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &"".to_string(),
                            Some(&v),
                        );
                    }
                    _ => {
                        handle_key_press(
                            Some(&paths[i]),
                            &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                            &mut stdout,
                            &make_idx(i, paths.len()),
                            Some(&v),
                        );
                    }
                },
                termion::event::Key::Char('V') => {
                    v.key = "v".to_string();
                    v.idx = make_idx(i, paths.len());
                    v.path = Some(&paths[i]);
                    handle_key_press(
                        v.path,
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &"".to_string(),
                        Some(&v),
                    );
                }
                termion::event::Key::Char('s') => {
                    stop_all(&sinks);
                }
                termion::event::Key::Char('q') => {
                    break 'outer;
                }
                termion::event::Key::Ctrl('c') => {
                    break 'outer;
                }
                _ => {
                    handle_key_press(
                        Some(&paths[i]),
                        &sinks[postincrement(&mut sink_idx, NUM_SINKS)],
                        &mut stdout,
                        &make_idx(i, paths.len()),
                        None,
                    );
                    stdout.lock().flush().unwrap();
                }
            }
        }
        thread::sleep(time::Duration::from_micros(1000));
    }
    write!(stdout, "\r\n").unwrap();

    match &z {
        z if !z.path.is_none() => {
            write!(stdout, "z: {}\r\n", z.path.unwrap().display().to_string()).unwrap();
        }
        _ => (),
    }
    match &x {
        x if !x.path.is_none() => {
            write!(stdout, "x: {}\r\n", x.path.unwrap().display().to_string()).unwrap();
        }
        _ => (),
    }
    match &c {
        c if !c.path.is_none() => {
            write!(stdout, "c: {}\r\n", c.path.unwrap().display().to_string()).unwrap();
        }
        _ => (),
    }
    match &v {
        v if !v.path.is_none() => {
            write!(stdout, "v: {}\r\n", v.path.unwrap().display().to_string()).unwrap();
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
