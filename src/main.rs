use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time,
};

#[allow(dead_code)]
mod gradescope;

fn main() {
    let filenames: Vec<String> = vec![
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_2.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_2.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_2.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_2.yml".to_string(),
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml".to_string(),
    ];

    let all: Arc<Mutex<Vec<gradescope::Export>>> = Arc::new(Mutex::new(Vec::new()));

    println!("Loading submissions");

    let start = time::Instant::now();

    let handles: Vec<JoinHandle<_>> = filenames
        .into_iter()
        .map(|filename| {
            let all = Arc::clone(&all);
            let join_handle = std::thread::spawn(move || {
                let result = gradescope::load_export(&filename);
                if let Ok(result) = result {
                    let mut all = all.lock().unwrap();
                    all.push(result);
                }
            });
            join_handle
        })
        .collect();

    // join all the threads and check for errors
    let results: Vec<_> = handles.into_iter().map(|handle| handle.join()).collect();
    if results.iter().any(|result| result.is_err()) {
        println!("Error loading submissions");
        return;
    }
    println!("Loaded submissions in {}ms", start.elapsed().as_millis());

    // Redeclare all as a vector of Export

    // find total number of submissions
    let total_submissions: usize = all.lock().unwrap().iter().map(|export| export.len()).sum();
    println!("Total submissions: {}", total_submissions);
}
