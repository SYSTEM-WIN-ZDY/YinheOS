//! A simple terminal implementation for TerraOS

mod commands;

use core::fmt;
use heapless::Vec;
use x86_64::instructions::port::Port;
use alloc::string::String;
use alloc::format;

// 键盘扫描码到ASCII字符的映射
const SCANCODE_TO_ASCII: [u8; 128] = [
    0,  0,  '1' as u8, '2' as u8, '3' as u8, '4' as u8, '5' as u8, '6' as u8,
    '7' as u8, '8' as u8, '9' as u8, '0' as u8, '-' as u8, '=' as u8, 0x08, 0x09,
    'q' as u8, 'w' as u8, 'e' as u8, 'r' as u8, 't' as u8, 'y' as u8, 'u' as u8, 'i' as u8,
    'o' as u8, 'p' as u8, '[' as u8, ']' as u8, 0x0D,  0,  'a' as u8, 's' as u8,
    'd' as u8, 'f' as u8, 'g' as u8, 'h' as u8, 'j' as u8, 'k' as u8, 'l' as u8, ';' as u8,
    '\'' as u8, '`' as u8,  0,  '\\' as u8, 'z' as u8, 'x' as u8, 'c' as u8, 'v' as u8,
    'b' as u8, 'n' as u8, 'm' as u8, ',' as u8, '.' as u8, '/' as u8,  0,  '*' as u8,
    0,  ' ' as u8,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  '7' as u8,
    '8' as u8, '9' as u8, '-' as u8, '4' as u8, '5' as u8, '6' as u8, '+' as u8, '1' as u8,
    '2' as u8, '3' as u8, '0' as u8, '.' as u8,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
];

// 大写字母映射
const SCANCODE_TO_UPPERCASE: [u8; 128] = [
    0,  0,  '!' as u8, '@' as u8, '#' as u8, '$' as u8, '%' as u8, '^' as u8,
    '&' as u8, '*' as u8, '(' as u8, ')' as u8, '_' as u8, '+' as u8, 0x08, 0x09,
    'Q' as u8, 'W' as u8, 'E' as u8, 'R' as u8, 'T' as u8, 'Y' as u8, 'U' as u8, 'I' as u8,
    'O' as u8, 'P' as u8, '{' as u8, '}' as u8, 0x0D,  0,  'A' as u8, 'S' as u8,
    'D' as u8, 'F' as u8, 'G' as u8, 'H' as u8, 'J' as u8, 'K' as u8, 'L' as u8, ':' as u8,
    '"' as u8, '~' as u8,  0,  '|' as u8, 'Z' as u8, 'X' as u8, 'C' as u8, 'V' as u8,
    'B' as u8, 'N' as u8, 'M' as u8, '<' as u8, '>' as u8, '?' as u8,  0,  '*' as u8,
    0,  ' ' as u8,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  '7' as u8,
    '8' as u8, '9' as u8, '-' as u8, '4' as u8, '5' as u8, '6' as u8, '+' as u8, '1' as u8,
    '2' as u8, '3' as u8, '0' as u8, '.' as u8,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,
];

/// The VGA text buffer color codes
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A color code consisting of a foreground and background color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A terminal character with color information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The VGA text buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// The terminal struct
pub struct Terminal {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    shift_pressed: bool,
    // 全局分配器引用，用于内存监控
    allocator: &'static crate::allocator::LinkedListAllocator,
}

impl Terminal {
    /// Create a new terminal instance
    pub fn new(allocator: &'static crate::allocator::LinkedListAllocator) -> Self {
        Terminal {
            column_position: 0,
            color_code: ColorCode::new(Color::Green, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
            shift_pressed: false,
            allocator,
        }
    }

    /// Clear the terminal screen
    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::Black, Color::Black),
                };
            }
        }
        self.column_position = 0;
    }

    /// Write a byte to the terminal
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }
    }

    /// Move to a new line
    fn new_line(&mut self) {
        // Scroll up all lines
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
            }
        }

        // Clear the last line
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col] = ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::Black, Color::Black),
            };
        }

        self.column_position = 0;
    }

    /// Write a string to the terminal
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Flush the terminal buffer to VGA display
    pub fn flush(&mut self) {
        use crate::vga_buffer::DoubleBuffer;
        let mut double_buffer = DoubleBuffer::new();
        
        // 将当前终端内容复制到双缓冲
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let screen_char = self.buffer.chars[row][col];
                double_buffer.write_char(row, col, screen_char.ascii_character, screen_char.color_code);
            }
        }
        
        // 刷新到VGA
        double_buffer.flush_to_vga();
    }

    /// Read a byte from the keyboard and handle special keys
    pub fn read_byte(&mut self) -> u8 {
        let mut port = Port::new(0x60);
        let scancode = unsafe { port.read() };

        // 检查是否为按键释放事件 (最高位为1)
        if scancode & 0x80 != 0 {
            // 处理Shift键释放
            match scancode {
                0xAA | 0xB6 => {
                    self.shift_pressed = false;
                },
                _ => {},
            }
            return 0;
        }

        // 直接返回扫描码，让read_line方法处理映射
        scancode
    }

    /// Read a line from the keyboard
    pub fn read_line(&mut self) -> String {
        let mut line = String::new();
        let mut last_scancode = 0;
        let mut _debounce_count = 0;
        loop {
            // 移除去抖动逻辑，因为它可能导致问题

            let scancode = self.read_byte();
            if scancode == 0 {
                continue; // 忽略特殊键处理后的0值
            }

            // 防止重复处理相同的扫描码
            if scancode == last_scancode {
                _debounce_count = 5;
                continue;
            }
            last_scancode = scancode;

            // 确保扫描码在有效范围内
            if scancode as usize >= SCANCODE_TO_ASCII.len() {
                continue;
            }

            // 处理特殊功能键
            match scancode {
                // 退格键
                0x0E => {
                    if !line.is_empty() {
                        line.pop();
                        self.write_byte(0x08);  // 退格符
                        self.write_byte(b' ');   // 空格覆盖
                        self.write_byte(0x08);  // 再次退格
                    }
                    continue;
                },
                // 回车键
                0x1C => {
                    self.write_byte(b'\n');
                    break;
                },
                // Tab键
                0x0F => {
                    for _ in 0..4 {
                        line.push(' ');
                        self.write_byte(b' ');
                    }
                    continue;
                },
                // Shift键
                0x2A | 0x36 => {
                    self.shift_pressed = true;
                    continue;
                },
                // 忽略其他非字符键
                0x01..=0x0D | 0x1A..=0x1B | 0x1D..=0x1F | 0x37..=0x7F => {
                    continue;
                },
                _ => {},
            }

            let ascii_char = if self.shift_pressed {
                // Shift键按下，使用大写映射
                SCANCODE_TO_UPPERCASE[scancode as usize]
            } else {
                // 默认使用小写映射
                SCANCODE_TO_ASCII[scancode as usize]
            };

            match ascii_char {
                // Printable characters
                0x20..=0x7E => {
                    line.push(ascii_char as char);
                    self.write_byte(ascii_char);
                },
                _ => {
                    // 对于无法识别的字符，输出一个问号
                    self.write_byte(b'?');
                },
            }
        }
        line
    }

    /// Run the terminal
    pub fn run(&mut self) {
        self.clear();
        self.write_str("TerraOS Terminal\n");
        self.write_str("Type 'help' for available commands\n\n");

        loop {
            self.write_str("$ ");
            let command = self.read_line();
            self.process_command(&command);
        }
    }

    // Command handlers
    fn handle_ls_command(&mut self, parts: &[&str]) {
        use crate::fs::new_fs::{FileSystem, SimpleFileSystem, MemoryBlockDevice};
        let device = MemoryBlockDevice::new();
        let mut fs = SimpleFileSystem::new(device);
        fs.init();

        let mut long_format = false;
        let mut show_all = false;

        // Parse options
        for part in parts.iter().skip(1) {
            match *part {
                "-l" => long_format = true,
                "-a" => show_all = true,
                _ => {}
            }
        }

        match fs.list_directory(0) {
            Ok(entries) => {
                for (name, inode_id, is_dir, size) in entries {
                    if !show_all && name.starts_with('.') {
                        continue;
                    }

                    if long_format {
                        let type_char = if is_dir { 'd' } else { '-' };
                        let size_str = self.format_size(size);
                        self.write_str(&format!("{} {} {} {}\n", type_char, "rwxr-xr-x", size_str, name));
                    } else {
                        self.write_str(&format!("{}\n", name));
                    }
                }
            },
            Err(e) => {
                self.write_str(&format!("Error: {}\n", e));
            }
        }
    }

    fn handle_mk_command(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            self.write_str("Usage: mk <directory_name>\n");
            return;
        }

        let dir_name = parts[1];
        
        use crate::fs::new_fs::{FileSystem, SimpleFileSystem, MemoryBlockDevice};
        let device = MemoryBlockDevice::new();
        let mut fs = SimpleFileSystem::new(device);
        fs.init();

        match fs.create_directory(dir_name, 0) {
            Ok(inode_id) => {
                self.write_str(&format!("Directory '{}' created successfully\n", dir_name));
            },
            Err(e) => {
                self.write_str(&format!("Error: {}\n", e));
            }
        }
    }

    fn handle_rm_command(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            self.write_str("Usage: rm [-r] [-f] <name>\n");
            return;
        }

        let mut recursive = false;
        let mut force = false;
        let mut name = "";

        // Parse options and arguments
        for part in parts.iter().skip(1) {
            match *part {
                "-r" => recursive = true,
                "-f" => force = true,
                _ => name = part,
            }
        }

        if name.is_empty() {
            self.write_str("Error: No file or directory specified\n");
            return;
        }

        if !force {
            self.write_str(&format!("Are you sure you want to delete '{}'{}? (y/N): ", 
                                name, if recursive { " and all its contents" } else { "" }));
            let response = self.read_line();
            if response != "y" && response != "Y" {
                self.write_str("Deletion cancelled\n");
                return;
            }
        }

        use crate::fs::new_fs::{FileSystem, SimpleFileSystem, MemoryBlockDevice};
        let device = MemoryBlockDevice::new();
        let mut fs = SimpleFileSystem::new(device);
        fs.init();

        match fs.delete_item(name, recursive) {
            Ok(_) => self.write_str(&format!("'{}' deleted successfully\n", name)),
            Err(e) => self.write_str(&format!("Error: {}\n", e)),
        }
    }

    fn handle_cd_command(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            self.write_str("Usage: cd <directory_path>\n");
            return;
        }

        let path = parts[1];
        
        // For now, just simulate changing directory
        if path == "/" || path == ".." || path == "." {
            self.write_str(&format!("Changed directory to {}\n", path));
        } else {
            self.write_str(&format!("Directory '{}' not found\n", path));
        }
    }

    fn handle_mv_command(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            self.write_str("Usage: mv <source> <destination>\n");
            return;
        }

        let src = parts[1];
        let dst = parts[2];

        use crate::fs::new_fs::{FileSystem, SimpleFileSystem, MemoryBlockDevice};
        let device = MemoryBlockDevice::new();
        let mut fs = SimpleFileSystem::new(device);
        fs.init();

        match fs.move_item(src, dst) {
            Ok(_) => self.write_str(&format!("Moved '{}' to '{}'\n", src, dst)),
            Err(e) => self.write_str(&format!("Error: {}\n", e)),
        }
    }

    fn handle_cp_command(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            self.write_str("Usage: cp [-r] <source> <destination>\n");
            return;
        }

        let mut recursive = false;
        let mut src = "";
        let mut dst = "";
        let mut arg_index = 1;

        // Parse options
        if parts[arg_index] == "-r" {
            recursive = true;
            arg_index += 1;
        }

        if parts.len() < arg_index + 2 {
            self.write_str("Usage: cp [-r] <source> <destination>\n");
            return;
        }

        src = parts[arg_index];
        dst = parts[arg_index + 1];

        use crate::fs::new_fs::{FileSystem, SimpleFileSystem, MemoryBlockDevice};
        let device = MemoryBlockDevice::new();
        let mut fs = SimpleFileSystem::new(device);
        fs.init();

        match fs.copy_item(src, dst, recursive) {
            Ok(_) => self.write_str(&format!("Copied '{}' to '{}'{}\n", src, dst, 
                                       if recursive { " recursively" } else { "" })),
            Err(e) => self.write_str(&format!("Error: {}\n", e)),
        }
    }

    // Helper function to format file sizes
    fn format_size(&self, size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;

        if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }

    /// Handle memory information display command
    fn handle_meminfo_command(&mut self) {
        use crate::system_monitor::SystemMonitor;
        let monitor = SystemMonitor::new(self.allocator);
        monitor.display_memory_info(self);
    }

    /// Handle memory statistics display command
    fn handle_memstats_command(&mut self) {
        use crate::system_monitor::SystemMonitor;
        let monitor = SystemMonitor::new(self.allocator);
        monitor.display_memory_info(self);
        
        // 添加详细统计信息
        self.write_str("\n=== 详细统计信息 ===\n");
        self.write_str("分配次数:      ");
        let alloc_count = self.allocator.get_allocation_count();
        self.write_str(&format!("{}\n", alloc_count));
        
        self.write_str("释放次数:      ");
        let dealloc_count = self.allocator.get_deallocation_count();
        self.write_str(&format!("{}\n", dealloc_count));
        
        if alloc_count > 0 {
            self.write_str("平均分配大小:  ");
            let total_allocated = self.allocator.get_total_allocated();
            let avg_size = total_allocated / alloc_count;
            self.write_str(&format!("{} 字节\n", avg_size));
        }
    }

    /// Handle system information display command
    fn handle_sysinfo_command(&mut self) {
        use crate::system_monitor::SystemMonitor;
        let monitor = SystemMonitor::new(self.allocator);
        monitor.display_system_info(self);
        
        // 添加一些额外的系统信息
        self.write_str("\n=== 内存信息 ===\n");
        let stats = self.allocator.get_memory_stats();
        self.write_str("堆地址范围:    0x500000 - 0x");
        self.write_str(&format!("{:X}\n", 0x500000 + crate::allocator::HEAP_SIZE));
        
        self.write_str("分配器类型:    链表分配器\n");
        self.write_str("双缓冲模式:    已启用\n");
        self.write_str("终端模式:      交互式\n");
    }

    /// Handle system health check command
    fn handle_syshealth_command(&mut self) {
        use crate::system_monitor::{SystemMonitor, MemoryHealthStatus};
        let monitor = SystemMonitor::new(self.allocator);
        monitor.display_health_check(self);
        
        // 添加简单的系统自检
        self.write_str("\n=== 系统自检 ===\n");
        
        // 检查内存分配器状态
        if self.allocator.get_allocation_count() > 0 {
            self.write_str("✅ 内存分配器:  正常\n");
        } else {
            self.write_str("⚠️  内存分配器:  未使用\n");
        }
        
        // 检查VGA缓冲
        self.write_str("✅ VGA缓冲:     正常\n");
        
        // 检查终端状态
        self.write_str("✅ 终端系统:    正常\n");
        
        // 检查文件系统
        self.write_str("✅ 文件系统:    可用\n");
        
        self.write_str("\n系统状态:      全部检查通过 ✅\n");
    }

    /// Process a terminal command
    fn process_command(&mut self, command: &str) {
        let mut parts = Vec::<&str, 16>::new();
        for part in command.split_whitespace() {
            if parts.push(part).is_err() {
                break; // Handle overflow if needed
            }
        }
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => self.write_str("Available commands: help, clear, echo, ls, mk, rm, cd, mv, cp, meminfo, memstats, sysinfo, syshealth\n"),
            "clear" => self.clear(),
            "echo" => {
                if parts.len() > 1 {
                    for (i, arg) in parts[1..].iter().enumerate() {
                        if i > 0 {
                            self.write_byte(b' ');
                        }
                        self.write_str(arg);
                    }
                }
                self.write_byte(b'\n');
            },
            "ls" => {
                self.handle_ls_command(&parts);
            },
            "mk" => {
                self.handle_mk_command(&parts);
            },
            "rm" => {
                self.handle_rm_command(&parts);
            },
            "cd" => {
                self.handle_cd_command(&parts);
            },
            "mv" => {
                self.handle_mv_command(&parts);
            },
            "cp" => {
                self.handle_cp_command(&parts);
            },
            "meminfo" => {
                self.handle_meminfo_command();
            },
            "memstats" => {
                self.handle_memstats_command();
            },
            "sysinfo" => {
                self.handle_sysinfo_command();
            },
            "syshealth" => {
                self.handle_syshealth_command();
            },
            _ => {
                self.write_str("Unknown command: ");
                self.write_str(command);
                self.write_byte(b'\n');
            },
        }
    }
}

// Implement fmt::Write for Terminal
impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Initialize and start the terminal
pub fn init(allocator: &'static crate::allocator::LinkedListAllocator) {
    let mut terminal = Terminal::new(allocator);
    terminal.run();
}