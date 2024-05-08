// atomic 연산
// store-and-load
// fetch-and-modify
// compare-and-exchange

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

// 정지 플래그
pub fn example1() {
    static STOP: AtomicBool = AtomicBool::new(false);

    let background_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {}
    });

    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}")
        }
    }

    STOP.store(true, Relaxed);
    background_thread.join().unwrap();
}

// 진행 상황 보고
pub fn example2() {
    let num_done = AtomicUsize::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..100 {
                process_item(i);
                num_done.store(i + 1, Relaxed);
            }
        });

        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 done");
            thread::sleep(Duration::from_secs(1));
        }
    });
    println!("Done!");
}


// 동기화
pub fn example3() {
    let num_done = AtomicUsize::new(0);
    let main_thread = thread::current();

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..100 {
                process_item(i);
                num_done.store(i + 1, Relaxed);
                main_thread.unpark();
            }
        });

        loop {
            let n = num_done.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });
    println!("Done!");
}

pub fn fetch_add_example() {
    let a = AtomicI32::new(0);
    let b = a.fetch_add(23, Relaxed);
    let c = a.load(Relaxed);

    println!("b = {b}");
    println!("c = {c}");
}

pub fn example4() {
    let num_dome = &AtomicUsize::new(0);

    thread::scope(|s| {
        for t in 0..4 {
            s.spawn(move || {
                for i in 0..25 {
                    process_item(i);
                    num_dome.fetch_add(1, Relaxed);
                }
            });
        }

        loop {
            let n = num_dome.load(Relaxed);
            if n == 100 {
                break;
            }
            println!("Working.. {n}/100 done");
            thread::sleep(Duration::from_secs(1));
        }
    });
}


fn process_item(i: usize) {
    thread::sleep(Duration::from_millis(i as u64 * 50));
}