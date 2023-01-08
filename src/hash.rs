use data_encoding::HEXUPPER;
use indicatif::ProgressBar;
use sha2::{Digest, Sha512};
use std::fs::metadata;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

static NTHREADS: i32 = 4;

struct Result {
    hash: String,
    path: String,
}

pub fn hash_file(path: &str) -> String {
    let mut hasher = Sha512::new();
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut buffer = [0; 1024 * 20];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(buffer);
    }
    HEXUPPER.encode(hasher.finalize().as_ref())
}

pub fn count_files(path: &str) -> u64 {
    let wal = WalkDir::new(path).into_iter();
    let mut count: u64 = 0;
    for entry in wal {
        let p = entry.unwrap().path().display().to_string();
        let meta = metadata(p).unwrap();
        if meta.is_dir() {
            continue;
        }
        count += 1;
    }
    count
}

pub fn hash(path: &str, output_file: &str) {
    let of = output_file.to_owned();
    let wal = Arc::new(Mutex::new(WalkDir::new(path).into_iter()));
    let (tx, rx): (Sender<Result>, Receiver<Result>) = mpsc::channel();

    println!("Contando arquivos...");
    let n_files = count_files(path);

    let pg_thread = thread::spawn(move || {
        let mut count = 0;
        let mut output = File::create(&of).unwrap();
        let pg = ProgressBar::new(n_files);
        while count < NTHREADS {
            let res = rx.recv().unwrap();
            if res.hash == "finish" {
                count += 1;
            } else {
                pg.inc(1);
                output
                    .write_all(format!("{} {}\n", res.path, res.hash).as_bytes())
                    .unwrap();
            }
        }
        let hash_hash = hash_file(of.as_str());
        println!("\nHash do hash:\n{}", hash_hash);
    });

    println!("Iniciando calculo de hashes");
    let mut handles = vec![];
    for _ in 0..NTHREADS {
        let thread_tx = tx.clone();
        let wal = Arc::clone(&wal);
        let handle = thread::spawn(move || {
            loop {
                let mut w = wal.lock().unwrap();
                let entry = w.next();
                match entry {
                    Some(e) => {
                        let p = e.unwrap().path().display().to_string();
                        let meta = metadata(&p).unwrap();
                        if meta.is_dir() {
                            continue;
                        }

                        let h = hash_file(p.as_str());
                        let r = Result { hash: h, path: p };
                        thread_tx.send(r).unwrap();
                    }
                    None => {
                        break;
                    }
                }
            }

            let r = Result {
                hash: String::from("finish"),
                path: String::new(),
            };
            thread_tx.send(r).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    pg_thread.join().unwrap();
}
