use sample_rust_project::{User, UserService};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting user management system...");
    
    let mut service = UserService::new();
    
    // Create some users
    let admin = service.create_user(
        "admin".to_string(),
        "admin@example.com".to_string()
    )?;
    println!("Created admin user: {:?}", admin);
    
    let user1 = service.create_user(
        "alice".to_string(),
        "alice@example.com".to_string()
    )?;
    println!("Created user: {:?}", user1);
    
    let user2 = service.create_user(
        "bob".to_string(),
        "bob@example.com".to_string()
    )?;
    println!("Created user: {:?}", user2);
    
    // List all users
    println!("\nAll users:");
    for user in service.list_users() {
        println!("  - {} ({})", user.username, user.email);
    }
    
    // Update email
    service.update_email(user1.id, "alice@newdomain.com".to_string())?;
    println!("\nUpdated Alice's email");
    
    // Find user
    match service.find_user(user2.id) {
        Some(user) => println!("Found user: {}", user.username),
        None => println!("User not found"),
    }
    
    Ok(())
}