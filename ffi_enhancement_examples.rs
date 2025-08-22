// FFI Enhancement Examples
// This file demonstrates usage of the new FFI bindings added to maa-sys

use maa_sys::{Assistant, Result, AsstTaskId};
use serde_json::json;

/// Example 1: Get list of active tasks
async fn example_get_tasks_list(assistant: &Assistant) -> Result<()> {
    println!("Getting active tasks list...");
    
    let tasks = assistant.get_tasks_list()?;
    
    if tasks.is_empty() {
        println!("No active tasks found");
    } else {
        println!("Found {} active tasks:", tasks.len());
        for (i, task_id) in tasks.iter().enumerate() {
            println!("  Task {}: ID = {}", i + 1, task_id);
        }
    }
    
    Ok(())
}

/// Example 2: Set task parameters dynamically
async fn example_set_task_params(assistant: &Assistant, task_id: AsstTaskId) -> Result<()> {
    println!("Setting parameters for task ID: {}", task_id);
    
    // Example: Update combat task parameters
    let new_params = json!({
        "stage": "CE-5",
        "medicine": 999,
        "stone": 0,
        "times": 5
    });
    
    assistant.set_task_params(task_id, new_params.to_string())?;
    println!("Successfully updated task parameters");
    
    Ok(())
}

/// Example 3: Navigation control - back to home
async fn example_back_to_home(assistant: &Assistant) -> Result<()> {
    println!("Navigating back to home screen...");
    
    assistant.back_to_home()?;
    println!("Successfully returned to home screen");
    
    Ok(())
}

/// Example 4: Complete workflow - Dynamic task management
async fn example_dynamic_task_management(assistant: &Assistant) -> Result<()> {
    println!("=== Dynamic Task Management Workflow ===");
    
    // 1. Check current tasks
    let initial_tasks = assistant.get_tasks_list()?;
    println!("Initial tasks count: {}", initial_tasks.len());
    
    // 2. Add a new combat task
    let task_params = json!({
        "stage": "1-7",
        "medicine": 999,
        "times": 10
    });
    
    let new_task_id = assistant.append_task("Fight", task_params.to_string())?;
    println!("Added new combat task with ID: {}", new_task_id);
    
    // 3. Check tasks list again
    let updated_tasks = assistant.get_tasks_list()?;
    println!("Updated tasks count: {}", updated_tasks.len());
    
    // 4. Modify the task parameters if needed
    if updated_tasks.contains(&new_task_id) {
        let modified_params = json!({
            "stage": "CE-5",  // Changed from 1-7 to CE-5
            "medicine": 999,
            "times": 5        // Reduced from 10 to 5
        });
        
        assistant.set_task_params(new_task_id, modified_params.to_string())?;
        println!("Modified parameters for task ID: {}", new_task_id);
    }
    
    // 5. Ensure we're at home screen before starting
    assistant.back_to_home()?;
    
    // 6. Start the tasks
    assistant.start()?;
    println!("Started task execution");
    
    Ok(())
}

/// Example 5: Task monitoring and control
async fn example_task_monitoring(assistant: &Assistant) -> Result<()> {
    println!("=== Task Monitoring Example ===");
    
    // Monitor tasks periodically
    for i in 0..5 {
        println!("Check #{}", i + 1);
        
        let tasks = assistant.get_tasks_list()?;
        println!("Active tasks: {}", tasks.len());
        
        if assistant.running() {
            println!("Assistant is currently running");
        } else {
            println!("Assistant is idle");
        }
        
        if assistant.connected() {
            println!("Device is connected");
        } else {
            println!("Device is not connected");
        }
        
        // Sleep for a bit before next check
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("MAA FFI Enhancement Examples");
    
    // Note: In real usage, you would:
    // 1. Load the MAA Core library: Assistant::load("/path/to/libMaaCore")
    // 2. Set user directory and load resources
    // 3. Create assistant with proper callback
    // 4. Connect to device
    
    // For this example, we'll just show the API usage patterns
    println!("This example shows the API usage patterns.");
    println!("In real usage, create and initialize the Assistant properly.");
    
    // Example assistant creation (commented out for demo):
    /*
    let assistant = Assistant::new(None, None);
    
    // Run examples
    example_get_tasks_list(&assistant).await?;
    example_set_task_params(&assistant, 1).await?;
    example_back_to_home(&assistant).await?;
    example_dynamic_task_management(&assistant).await?;
    example_task_monitoring(&assistant).await?;
    */
    
    Ok(())
}