use crate::models::execution::{self, EXECUTION_HISTORY_LIMIT};
use crate::storage::output;
use sqlx::SqlitePool;
use std::path::Path;

pub async fn enforce_execution_history_limit(pool: &SqlitePool, output_dir: &Path) {
    let stale_executions =
        match execution::prune_execution_history(pool, EXECUTION_HISTORY_LIMIT).await {
            Ok(executions) => executions,
            Err(e) => {
                eprintln!("Failed to prune stale execution history: {}", e);
                return;
            }
        };

    for stale_execution in stale_executions {
        if let Some(output_file) = stale_execution.output_file {
            if let Err(e) = output::delete_output_file(output_dir, &output_file).await {
                eprintln!(
                    "Failed to delete output file '{}' for stale execution '{}': {}",
                    output_file, stale_execution.id, e
                );
            }
            continue;
        }

        if let Err(e) =
            output::delete_output_files_for_execution(output_dir, &stale_execution.id).await
        {
            eprintln!(
                "Failed to delete output files for stale execution '{}': {}",
                stale_execution.id, e
            );
        }
    }
}
