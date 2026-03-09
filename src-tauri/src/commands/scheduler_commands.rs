use crate::scheduler::job_scheduler::{JobCallback, Scheduler, SchedulerState};
use crate::scheduler::parse_simple_schedule;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

/// Start the scheduler
#[tauri::command]
pub async fn start_scheduler(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    scheduler_guard
        .start()
        .await
        .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    
    Ok("Scheduler started successfully".to_string())
}

/// Stop the scheduler
#[tauri::command]
pub async fn stop_scheduler(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    scheduler_guard
        .stop()
        .await
        .map_err(|e| format!("Failed to stop scheduler: {}", e))?;
    
    Ok("Scheduler stopped successfully".to_string())
}

/// Get scheduler status
#[tauri::command]
pub async fn get_scheduler_status(
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
) -> Result<String, String> {
    let scheduler = scheduler.inner().clone();
    let scheduler_guard = scheduler.lock().await;
    
    let state = scheduler_guard.get_state().await;
    
    let status = match state {
        SchedulerState::Running => "running",
        SchedulerState::Stopped => "stopped",
    };
    
    Ok(status.to_string())
}

/// Get cron expression from task
fn get_task_cron_expression(task: &crate::models::task::Task) -> Option<String> {
    if let Some(cron) = &task.cron_expression {
        Some(cron.as_str().to_string())
    } else if let Some(json) = &task.simple_schedule {
        parse_simple_schedule(json).ok()
    } else {
        None
    }
}

/// Reload scheduler with all enabled tasks from database
#[tauri::command]
pub async fn reload_scheduler(
    pool: State<'_, Arc<SqlitePool>>,
    scheduler: State<'_, Arc<Mutex<Scheduler>>>,
    app: AppHandle,
) -> Result<String, String> {
    let pool = pool.inner().clone();
    let scheduler = scheduler.inner().clone();
    
    // Get all tasks from database
    let tasks = crate::models::task::get_all_tasks(&pool)
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))?;
    
    // Stop scheduler if running
    {
        let scheduler_guard = scheduler.lock().await;
        if scheduler_guard.get_state().await == SchedulerState::Running {
            let _ = scheduler_guard.stop().await;
        }
    }
    
    // Track loaded count
    let mut loaded_count = 0;
    let mut errors = Vec::new();
    
    // Add each enabled task with schedule
    for task in tasks.iter() {
        // Skip disabled tasks
        if task.enabled == 0 {
            continue;
        }
        
        // Get cron expression
        let cron_expr = match get_task_cron_expression(&task) {
        Some(expr) => expr,
        None => {
            errors.push(format!("Task '{}' has no valid schedule", task.id));
            continue;
        }
    };
        
        // Create callback for this task
        let task_id = task.id.clone();
        let pool_clone = pool.clone();
        let app_handle = app.clone();
        let task_timeout = task.timeout_seconds as u64;
        
        let callback: JobCallback = Arc::new(move || {
            let task_id = task_id.clone();
            let pool = pool_clone.clone();
            let app = app_handle.clone();
            let timeout = task_timeout;
            
            Box::pin(async move {
                // Execute the task
                let result = execute_task_internal(task_id, pool, app, timeout).await;
                if let Err(e) = result {
                    eprintln!("Task execution failed: {}", e);
                }
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        });
        
        // Add job to scheduler
        let scheduler_guard = scheduler.lock().await;
        match scheduler_guard.add_job(&task.id, &cron_expr, callback).await {
                Ok(_) => loaded_count += 1,
                Err(e) => errors.push(format!("Failed to add job for task '{}': {}", task.id, e)),
            }
    }
    
    // Start scheduler
    {
        let scheduler_guard = scheduler.lock().await;
        scheduler_guard.start().await
            .map_err(|e| format!("Failed to start scheduler: {}", e))?;
    }
    
    // Return result
    if errors.is_empty() {
        Ok(format!("Successfully loaded {} tasks", loaded_count))
    } else {
        Ok(format!("Loaded {} tasks with {} errors: {}", loaded_count, errors.len(), errors.join(", ")))
    }
}

/// Internal function to execute a task (used by scheduler callbacks)
/// Internal function to execute a task (used by scheduler callbacks)
pub async fn execute_task_internal(
    task_id: String,
    pool: Arc<SqlitePool>,
    app: AppHandle,
    timeout_seconds: u64,
) -> Result<(), String> {
    use crate::models::execution::ExecutionStatus;
    use crate::opencode::executor::run_opencode_task;
    use crate::storage::output;
    use chrono::Utc;
    
    // Get task
    let task = crate::models::task::get_task(&pool, &task_id)
        .await
        .map_err(|e| format!("Task not found: {}", e))?;
    
    // Create execution record
    let new_execution = crate::models::execution::NewExecution {
        task_id: task_id.clone(),
        session_id: None,
        status: Some(crate::models::execution::ExecutionStatus::Running),
        output_file: None,
        error_message: None,
    };
    
    let execution = crate::models::execution::create_execution(&pool, new_execution)
        .await
        .map_err(|e| format!("Failed to create execution: {}", e))?;
    
    // Run opencode task
    let result = run_opencode_task(&task.prompt, None, Some(timeout_seconds), None).await;
    
    // Save output
    let output_dir = output::get_output_directory(&app)
        .map_err(|e| format!("Failed to get output directory: {}", e))?;
    
    output::create_output_directory(&output_dir)
        .await
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    
    let (status, finished_at, output_file, error_message) = match result {
        Ok(opencode_output) => {
            let (final_status, err_msg) = if opencode_output.timed_out {
                (ExecutionStatus::Timeout, Some("Execution timed out".to_string()))
            } else if !opencode_output.success {
                (ExecutionStatus::Failed, Some(opencode_output.stderr.clone()))
            } else {
                (ExecutionStatus::Success, None)
            };
            
            let content = format!(
                "Session ID: {}\n\n=== STDOUT ===\n{}\n\n=== STDERR ===\n{}",
                opencode_output.session_id,
                opencode_output.stdout,
                opencode_output.stderr
            );
            
            let file_path = output::write_output_file(&output_dir, &execution.id, &content)
                .await
                .map_err(|e| format!("Failed to write output file: {}", e))?;
            
            let file_path_str = file_path.to_string_lossy().to_string();
            
            (final_status, Utc::now().to_rfc3339(), Some(file_path_str), err_msg)
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            let content = format!("Error: {}", error_msg);
            
            let _ = output::write_output_file(&output_dir, &execution.id, &content).await;
            
            (ExecutionStatus::Failed, Utc::now().to_rfc3339(), None, Some(error_msg))
        }
    };
    
    // Update execution record
    let update = crate::models::execution::UpdateExecution {
        session_id: None,
        status: Some(status.clone()),
        finished_at: Some(finished_at),
        output_file: output_file.clone(),
        error_message: error_message.clone(),
    };
    
    crate::models::execution::update_execution(&pool, &execution.id, update)
        .await
        .map_err(|e| format!("Failed to update execution: {}", e))?;
    
    Ok(())
}
