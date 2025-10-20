// ABOUTME: Test script to debug storage initialization
// ABOUTME: This example directly tests the storage manager initialization

use orkee_projects::manager::get_storage_manager;

#[tokio::main]
async fn main() {
    println!("Attempting to initialize storage manager...");
    match get_storage_manager().await {
        Ok(manager) => {
            println!("✅ Storage manager initialized successfully");
            let storage = manager.storage();
            match storage.list_projects().await {
                Ok(projects) => println!("✅ Found {} projects", projects.len()),
                Err(e) => println!("❌ Error listing projects: {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize storage manager:");
            eprintln!("Error: {:?}", e);
        }
    }
}
