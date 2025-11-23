# TerraOS - 简易裸机操作系统

TerraOS 是一个使用 Rust 编程语言开发的简易裸机操作系统，能够在无底层操作系统支持的环境中直接启动。该系统包含一个简单的文件系统以实现基本的文件管理功能。

## 特性

- 裸机启动，无需底层操作系统支持
- 使用 Rust 编写，具有内存安全保证
- 简易的文件系统实现
- VGA 文本模式输出

## 项目结构

- `terra_os_kernel/` - 操作系统内核源代码
  - `src/` - 内核源代码
    - `main.rs` - 内核入口点
    - `fs/` - 文件系统实现
  - `Cargo.toml` - Rust 包配置文件
  - `x86_64-terra_os.json` - 自定义目标规范
  - `Makefile` - 构建脚本

## 构建要求

- Rust 工具链 (推荐使用 nightly 版本)
- cargo-bootimage 工具

安装必要的工具：

```bash
cargo install cargo-bootimage
```

## 构建步骤

1. 进入内核目录：
   ```bash
   cd terra_os_kernel
   ```

2. 构建内核镜像：
   ```bash
   cargo bootimage
   ```

3. 生成的可启动镜像将位于 `target/x86_64-terra_os/debug/bootimage-terra_os_kernel.bin`

## 运行

可以使用 QEMU 来运行生成的镜像：

```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-terra_os/debug/bootimage-terra_os_kernel.bin
```

## 文件系统

当前实现包含一个简单的只读内存文件系统，具有以下结构：

```
/
├── README.md
├── kernel.bin
└── docs/
    └── design.md
```

文件系统实现在 `src/fs/mod.rs` 中，支持基本的文件和目录查找功能。