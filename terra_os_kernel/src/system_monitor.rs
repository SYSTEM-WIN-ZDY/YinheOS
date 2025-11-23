use crate::allocator::{LinkedListAllocator, MemoryStats};
use crate::terminal::Terminal;

pub struct SystemMonitor {
    allocator: &'static LinkedListAllocator,
}

impl SystemMonitor {
    pub const fn new(allocator: &'static LinkedListAllocator) -> Self {
        SystemMonitor { allocator }
    }

    pub fn display_memory_info(&self, terminal: &mut Terminal) {
        let stats = self.allocator.get_memory_stats();
        
        terminal.write_str("=== 内存监控信息 ===\n");
        terminal.write_str("总堆大小:        ");
        self.format_bytes(stats.total_heap_size, terminal);
        terminal.write_str("\n");

        terminal.write_str("当前已分配:      ");
        self.format_bytes(stats.current_allocated, terminal);
        terminal.write_str(" (");
        self.format_percentage((stats.current_allocated as f64 / stats.total_heap_size as f64) * 100.0, terminal);
        terminal.write_str(")\n");

        terminal.write_str("当前可用:        ");
        self.format_bytes(stats.free_memory, terminal);
        terminal.write_str(" (");
        self.format_percentage((stats.free_memory as f64 / stats.total_heap_size as f64) * 100.0, terminal);
        terminal.write_str(")\n");

        terminal.write_str("历史最大分配:    ");
        self.format_bytes(stats.max_allocated, terminal);
        terminal.write_str(" (");
        self.format_percentage((stats.max_allocated as f64 / stats.total_heap_size as f64) * 100.0, terminal);
        terminal.write_str(")\n");

        terminal.write_str("\n=== 分配统计 ===\n");
        terminal.write_str("总分配次数:     ");
        self.format_number(stats.allocation_count, terminal);
        terminal.write_str("\n");

        terminal.write_str("总释放次数:     ");
        self.format_number(stats.deallocation_count, terminal);
        terminal.write_str("\n");

        terminal.write_str("总分配内存:     ");
        self.format_bytes(stats.allocated, terminal);
        terminal.write_str("\n");

        terminal.write_str("总释放内存:     ");
        self.format_bytes(stats.freed, terminal);
        terminal.write_str("\n");

        terminal.write_str("\n=== 性能指标 ===\n");
        
        if stats.allocation_count > 0 {
            let avg_alloc_size = stats.allocated / stats.allocation_count;
            terminal.write_str("平均分配大小:   ");
            self.format_bytes(avg_alloc_size, terminal);
            terminal.write_str("\n");
        }

        if stats.deallocation_count > 0 {
            let avg_free_size = stats.freed / stats.deallocation_count;
            terminal.write_str("平均释放大小:   ");
            self.format_bytes(avg_free_size, terminal);
            terminal.write_str("\n");
        }

        terminal.write_str("内存利用率:     ");
        self.format_percentage((stats.current_allocated as f64 / stats.total_heap_size as f64) * 100.0, terminal);
        terminal.write_str("\n");

        terminal.write_str("碎片化程度:     ");
        let fragmentation = if stats.current_allocated > 0 {
            ((stats.total_heap_size - stats.free_memory) as f64 / stats.total_heap_size as f64) * 100.0
        } else {
            0.0
        };
        self.format_percentage(fragmentation, terminal);
        terminal.write_str("\n");
    }

    pub fn display_system_info(&self, terminal: &mut Terminal) {
        terminal.write_str("=== 系统信息 ===\n");
        terminal.write_str("操作系统:       TerraOS (Rust Kernel)\n");
        terminal.write_str("内核版本:       0.1.0\n");
        terminal.write_str("构建时间:       运行时统计\n");
        terminal.write_str("架构:           x86_64\n");
        
        terminal.write_str("\n=== 系统状态 ===\n");
        terminal.write_str("系统状态:       正常运行\n");
        terminal.write_str("终端:           已初始化\n");
        terminal.write_str("内存管理:       已启用\n");
        terminal.write_str("VGA缓冲:        双缓冲模式\n");
    }

    fn format_bytes(&self, bytes: u64, terminal: &mut Terminal) {
        if bytes < 1024 {
            terminal.write_str(&format!("{} B", bytes));
        } else if bytes < 1024 * 1024 {
            terminal.write_str(&format!("{}.{} KB", bytes / 1024, (bytes % 1024) * 10 / 1024));
        } else {
            terminal.write_str(&format!("{}.{} MB", bytes / (1024 * 1024), (bytes % (1024 * 1024)) * 10 / (1024 * 1024)));
        }
    }

    fn format_percentage(&self, percentage: f64, terminal: &mut Terminal) {
        terminal.write_str(&format!("{:.1}%", percentage));
    }

    fn format_number(&self, number: u64, terminal: &mut Terminal) {
        terminal.write_str(&format!("{}", number));
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        self.allocator.get_memory_stats()
    }

    pub fn check_memory_health(&self) -> MemoryHealth {
        let stats = self.get_memory_stats();
        let usage_percent = (stats.current_allocated as f64 / stats.total_heap_size as f64) * 100.0;
        let free_percent = (stats.free_memory as f64 / stats.total_heap_size as f64) * 100.0;

        let mut status = MemoryHealthStatus::Healthy;
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // 检查内存使用情况
        if usage_percent > 90.0 {
            status = MemoryHealthStatus::Critical;
            warnings.push("内存使用率过高 (超过90%)");
            recommendations.push("考虑清理不必要的内存分配");
        } else if usage_percent > 75.0 {
            status = MemoryHealthStatus::Warning;
            warnings.push("内存使用率较高 (超过75%)");
            recommendations.push("监控内存使用情况");
        }

        // 检查碎片化
        let fragmentation = if stats.current_allocated > 0 {
            ((stats.total_heap_size - stats.free_memory) as f64 / stats.total_heap_size as f64) * 100.0
        } else {
            0.0
        };

        if fragmentation > 80.0 {
            warnings.push("内存碎片化严重");
            recommendations.push("考虑重新组织内存分配策略");
        }

        // 检查分配失败
        if stats.total_allocated == 0 && stats.allocation_count > 0 {
            status = MemoryHealthStatus::Error;
            warnings.push("检测到分配异常");
        }

        MemoryHealth {
            status,
            usage_percent,
            free_percent,
            fragmentation,
            warnings,
            recommendations,
        }
    }

    pub fn display_health_check(&self, terminal: &mut Terminal) {
        let health = self.check_memory_health();
        
        terminal.write_str("=== 内存健康检查 ===\n");
        terminal.write_str("健康状态:       ");
        
        match health.status {
            MemoryHealthStatus::Healthy => {
                terminal.write_str("正常\n");
                terminal.write_str("✅ 系统内存状态良好\n");
            }
            MemoryHealthStatus::Warning => {
                terminal.write_str("警告\n");
                terminal.write_str("⚠️  内存使用率较高，需要关注\n");
            }
            MemoryHealthStatus::Critical => {
                terminal.write_str("严重\n");
                terminal.write_str("🚨 内存使用率过高！\n");
            }
            MemoryHealthStatus::Error => {
                terminal.write_str("错误\n");
                terminal.write_str("❌ 检测到内存分配异常\n");
            }
        }

        if !health.warnings.is_empty() {
            terminal.write_str("\n⚠️  警告信息:\n");
            for warning in health.warnings {
                terminal.write_str("• ");
                terminal.write_str(warning);
                terminal.write_str("\n");
            }
        }

        if !health.recommendations.is_empty() {
            terminal.write_str("\n💡 建议:\n");
            for recommendation in health.recommendations {
                terminal.write_str("• ");
                terminal.write_str(recommendation);
                terminal.write_str("\n");
            }
        }

        terminal.write_str("\n内存使用率:     ");
        self.format_percentage(health.usage_percent, terminal);
        terminal.write_str("\n");

        terminal.write_str("可用内存比例:   ");
        self.format_percentage(health.free_percent, terminal);
        terminal.write_str("\n");

        terminal.write_str("碎片化程度:     ");
        self.format_percentage(health.fragmentation, terminal);
        terminal.write_str("\n");
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryHealthStatus {
    Healthy,
    Warning,
    Critical,
    Error,
}

pub struct MemoryHealth {
    pub status: MemoryHealthStatus,
    pub usage_percent: f64,
    pub free_percent: f64,
    pub fragmentation: f64,
    pub warnings: Vec<&'static str>,
    pub recommendations: Vec<&'static str>,
}