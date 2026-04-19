//! Task Scheduler
//!
//! 基于 tokio::time 的任务调度器，参考 memoh-X CronPool 设计。
//! 无需引入 robfig/cron crate，利用现有 tokio full feature。

use super::models::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

/// 共享任务调度器
pub struct TaskScheduler {
    /// 任务ID -> 定时器句柄
    handles: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    /// 任务ID -> 互斥锁（防止重叠执行）
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(HashMap::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册一个定时任务
    /// 
    /// - once 任务：立即执行（不注册定时器，由调用方直接触发）
    /// - daily/weekly/cron 任务：注册 tokio 定时器
    pub fn register<F>(
        &self,
        task: &Task,
        callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn() + Send + 'static,
    {
        let task_id = task.id.clone();

        // 先注销已存在的
        self.unregister(&task_id);

        // 创建互斥锁
        let lock = Arc::new(tokio::sync::Mutex::new(()));
        {
            let mut locks = self.locks.lock().unwrap();
            locks.insert(task_id.clone(), lock.clone());
        }

        match task.schedule_type {
            ScheduleType::Once => {
                // 一次性任务不注册定时器，由调用方直接触发
                log::info!("[TaskScheduler] Registered once task: {}", task_id);
            }
            ScheduleType::Daily => {
                let handle = self.spawn_interval(task_id.clone(), Duration::from_secs(86400), lock, callback);
                {
                    let mut handles = self.handles.lock().unwrap();
                    handles.insert(task_id.clone(), handle);
                }
                log::info!("[TaskScheduler] Registered daily task: {}", task_id);
            }
            ScheduleType::Weekly => {
                let handle = self.spawn_interval(task_id.clone(), Duration::from_secs(604800), lock, callback);
                {
                    let mut handles = self.handles.lock().unwrap();
                    handles.insert(task_id.clone(), handle);
                }
                log::info!("[TaskScheduler] Registered weekly task: {}", task_id);
            }
            ScheduleType::Cron => {
                // 解析 cron 表达式，转换为 interval
                let interval_duration = Self::parse_cron_interval(task.cron_pattern.as_deref())?;
                let handle = self.spawn_interval(task_id.clone(), interval_duration, lock, callback);
                {
                    let mut handles = self.handles.lock().unwrap();
                    handles.insert(task_id.clone(), handle);
                }
                log::info!("[TaskScheduler] Registered cron task: {} (interval: {:?})", task_id, interval_duration);
            }
        }

        Ok(())
    }

    /// 注销任务定时器
    pub fn unregister(&self, task_id: &str) {
        let mut handles = self.handles.lock().unwrap();
        if let Some(handle) = handles.remove(task_id) {
            handle.abort();
            log::info!("[TaskScheduler] Unregistered task: {}", task_id);
        }

        let mut locks = self.locks.lock().unwrap();
        locks.remove(task_id);
    }

    /// 检查任务是否已注册
    pub fn is_registered(&self, task_id: &str) -> bool {
        let handles = self.handles.lock().unwrap();
        handles.contains_key(task_id)
    }

    /// 获取任务的执行锁（用于立即执行时防止重叠）
    pub fn get_lock(&self, task_id: &str) -> Option<Arc<tokio::sync::Mutex<()>>> {
        let locks = self.locks.lock().unwrap();
        locks.get(task_id).cloned()
    }

    /// 创建或获取任务的执行锁
    pub fn ensure_lock(&self, task_id: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self.locks.lock().unwrap();
        locks.entry(task_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }

    /// 停止所有定时器
    pub fn stop_all(&self) {
        let mut handles = self.handles.lock().unwrap();
        for (id, handle) in handles.drain() {
            handle.abort();
            log::info!("[TaskScheduler] Stopped task: {}", id);
        }
        let mut locks = self.locks.lock().unwrap();
        locks.clear();
    }

    // ==================== Internal ====================

    fn spawn_interval<F>(
        &self,
        task_id: String,
        duration: Duration,
        lock: Arc<tokio::sync::Mutex<()>>,
        callback: F,
    ) -> JoinHandle<()>
    where
        F: Fn() + Send + 'static,
    {
        tokio::spawn(async move {
            let mut ticker = interval(duration);
            // 第一次延迟执行，避免启动时立即触发
            ticker.tick().await;

            loop {
                ticker.tick().await;

                // 尝试获取锁，如果任务还在执行则跳过本次触发
                if let Ok(_guard) = lock.try_lock() {
                    log::info!("[TaskScheduler] Triggering scheduled task: {}", task_id);
                    callback();
                } else {
                    log::warn!("[TaskScheduler] Skipping overlapping trigger for task: {}", task_id);
                }
            }
        })
    }

    /// 简化版 cron 解析：将 cron 表达式转为 interval 周期
    /// 支持格式: "*/N * * * *" (每N分钟) 或 "0 H * * *" (每天H点)
    fn parse_cron_interval(pattern: Option<&str>) -> Result<Duration, Box<dyn std::error::Error>> {
        let pattern = pattern.ok_or("Cron pattern is required for cron schedule type")?;
        let parts: Vec<&str> = pattern.split_whitespace().collect();

        if parts.len() < 2 {
            return Err("Invalid cron pattern: need at least minute and hour fields".into());
        }

        // 检查是否是 */N 格式（每N分钟）
        if parts[0].starts_with("*/") {
            if let Some(num_str) = parts[0].strip_prefix("*/") {
                if let Ok(minutes) = num_str.parse::<u64>() {
                    return Ok(Duration::from_secs(minutes * 60));
                }
            }
        }

        // 如果是具体的分钟+小时（如 "0 3 * * *" 每天3点），默认返回24小时
        // 实际触发逻辑由更上层管理（下次运行时间计算）
        Ok(Duration::from_secs(86400))
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
