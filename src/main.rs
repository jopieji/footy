use tokio::time;
use log::Level;

async fn run() {
    log::info!("Sleeping");
    time::sleep(time::Duration::from_secs(1)).await;
    log::info!("Awake!");
}

fn main() {
    println!("\nGlobal football CLI\n============================\n");

    simple_logger::init_with_level(Level::Info).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = run();

    rt.block_on(future);

    // can concurrently make requests using tokio::join!() macro with all necessary function calls as arguments
}
