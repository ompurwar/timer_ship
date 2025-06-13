use super::{LogEntry, LogOperation, OpLog};
use crate::core::{Timer, TimerData, Timers};
use log::{debug, info};

/// Manages recovery of timer state from operation logs
pub struct RecoveryManager {
    oplog: OpLog,
}

impl RecoveryManager {
    pub fn new(oplog: OpLog) -> Self {
        Self { oplog }
    }

    /// Recovers timer state from operation logs
    pub fn recover_from_logs(
        &self,
        timers: &Timers,
        timer_data: &TimerData,
    ) -> std::io::Result<()> {
        info!("Starting recovery from logs...");
        let logs = self.oplog.read_logs()?;
        let log_count = logs.len();

        for entry in &logs {
            match &entry.operation {
                LogOperation::SetTimer {
                    timer_id,
                    expires_at,
                    data,
                } => {
                    let timer = Timer::with_id(*expires_at, *timer_id);
                    timer_data.add_data(*timer_id, data.clone());
                    timers.add_timer(timer);
                    debug!("Recovered SetTimer: ID {}, expires_at {}", timer_id, expires_at);
                }
                LogOperation::RemoveTimer { timer_id } => {
                    timers.remove_timer(*timer_id);
                    timer_data.remove_data(*timer_id);
                    debug!("Recovered RemoveTimer: ID {}", timer_id);
                }
            }
        }

        info!("Recovery completed. Processed {} log entries.", log_count);
        Ok(())
    }
}
