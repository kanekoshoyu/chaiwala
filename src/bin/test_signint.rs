// setup signint handler
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() {
    // Create an atomic flag to indicate whether a termination signal has been received
    let termination_requested = Arc::new(AtomicBool::new(false));
    let sigterm_flag = termination_requested.clone();

    // Spawn a Tokio task to listen for SIGINT signals
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");
        sigint.recv().await;
        println!("Received SIGINT signal. Shutting down gracefully...");
        sigterm_flag.store(true, Ordering::SeqCst);
    });

    // Your main application logic here
    while !termination_requested.load(Ordering::SeqCst) {
        // Perform your application's work
        println!("Running...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // Perform any necessary cleanup or shutdown actions here
    cleanup_and_shutdown();
}

fn cleanup_and_shutdown() {
    // Add your cleanup and shutdown logic here
    // For example, release resources, save state, close connections, etc.
    println!("Cleanup and shutdown logic executed.");
}
