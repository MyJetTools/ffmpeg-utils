use rust_extensions::TaskCompletion;

use super::FFmpegExecutionResult;

pub struct ToPcmCommand {
    pub file: String,
    pub last_sample_data: usize,
    pub task_completion: TaskCompletion<FFmpegExecutionResult, String>,
}
