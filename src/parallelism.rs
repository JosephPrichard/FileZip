/*
 * Copyright (c) Joseph Prichard 2022.
 */

use std::thread::available_parallelism;

pub fn configure_thread_pool(multithreaded: bool, file_count: usize) {
    // configure the rayon thread pool based on -mt flag
    let threads = if multithreaded {
        file_count.min(available_parallelism().unwrap().get())
    } else {
        1
    };
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("Failed to configure thread pool");

    println!("Running with {} threads", threads);
}