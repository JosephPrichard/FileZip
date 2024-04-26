// Joseph Prichard
// 4/25/2024
// Threading operations

use std::io;
use std::thread::available_parallelism;
use rayon::ThreadPool;

pub fn configure_thread_pool(multithreaded: bool, file_count: usize) -> io::Result<ThreadPool> {
    // configure the rayon thread pool based on -mt flag
    let threads = if multithreaded {
        file_count.min(available_parallelism()?.get())
    } else {
        1
    };

    println!("Running with {} threads", threads);
    let tp = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("Failed to configure thread pool");
    Ok(tp)
}
