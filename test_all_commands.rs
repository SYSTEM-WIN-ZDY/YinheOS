// Comprehensive test for all filesystem commands
// This demonstrates all implemented commands

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct FileSystemItem {
    name: String,
    is_directory: bool,
    size: u32,
    contents: Option<String>,
    children: HashMap<String, FileSystemItem>,
}

struct FileSystem {
    root: FileSystemItem,
    current_path: Vec<String>,
}

impl FileSystem {
    fn new() -> Self {
        let mut root = FileSystemItem {
            name: "/".to_string(),
            is_directory: true,
            size: 0,
            contents: None,
            children: HashMap::new(),
        };

        // Add some initial directories and files
        root.children.insert("bin".to_string(), FileSystemItem {
            name: "bin".to_string(),
            is_directory: true,
            size: 1024,
            contents: None,
            children: HashMap::new(),
        });

        root.children.insert("etc".to_string(), FileSystemItem {
            name: "etc".to_string(),
            is_directory: true,
            size: 2048,
            contents: None,
            children: HashMap::new(),
        });

        root.children.insert("README.txt".to_string(), FileSystemItem {
            name: "README.txt".to_string(),
            is_directory: false,
            size: 512,
            contents: Some("Welcome to TerraOS!".to_string()),
            children: HashMap::new(),
        });

        FileSystem {
            root,
            current_path: Vec::new(),
        }
    }

    fn get_current_dir(&self) -> &FileSystemItem {
        let mut current = &self.root;
        for dir_name in &self.current_path {
            if let Some(child) = current.children.get(dir_name) {
                current = child;
            }
        }
        current
    }

    fn get_current_dir_mut(&mut self) -> &mut FileSystemItem {
        let mut current = &mut self.root;
        for dir_name in self.current_path.clone() {
            if let Some(child) = current.children.get_mut(&dir_name) {
                current = child;
            }
        }
        current
    }

    fn create_directory(&mut self, name: &str) -> Result<(), &'static str> {
        if name.is_empty() {
            return Err("Directory name cannot be empty");
        }

        let current = self.get_current_dir_mut();
        if !current.is_directory {
            return Err("Current location is not a directory");
        }

        if current.children.contains_key(name) {
            return Err("Directory already exists");
        }

        current.children.insert(name.to_string(), FileSystemItem {
            name: name.to_string(),
            is_directory: true,
            size: 4096,
            contents: None,
            children: HashMap::new(),
        });

        Ok(())
    }

    fn list_directory(&self, show_all: bool, long_format: bool) -> Result<Vec<String>, &'static str> {
        let current = self.get_current_dir();
        if !current.is_directory {
            return Err("Not a directory");
        }

        let mut entries = Vec::new();
        
        // Always include . and ..
        entries.push(".".to_string());
        entries.push("..".to_string());

        for (name, item) in &current.children {
            if !show_all && name.starts_with('.') {
                continue;
            }

            if long_format {
                let type_char = if item.is_directory { 'd' } else { '-' };
                let size = format_size(item.size);
                entries.push(format!("{} {} {} {}", type_char, "rwxr-xr-x", size, name));
            } else {
                entries.push(name.clone());
            }
        }

        Ok(entries)
    }

    fn delete_item(&mut self, name: &str, _recursive: bool) -> Result<(), &'static str> {
        let current = self.get_current_dir_mut();
        
        if name == "." || name == ".." {
            return Err("Cannot delete . or ..");
        }

        match current.children.remove(name) {
            Some(_) => Ok(()),
            None => Err("File or directory not found"),
        }
    }

    fn change_directory(&mut self, path: &str) -> Result<(), &'static str> {
        match path {
            "/" => {
                self.current_path.clear();
                Ok(())
            },
            ".." => {
                if !self.current_path.is_empty() {
                    self.current_path.pop();
                }
                Ok(())
            },
            "." => Ok(()),
            dir_name => {
                let current = self.get_current_dir();
                if let Some(item) = current.children.get(dir_name) {
                    if item.is_directory {
                        self.current_path.push(dir_name.to_string());
                        Ok(())
                    } else {
                        Err("Not a directory")
                    }
                } else {
                    Err("Directory not found")
                }
            }
        }
    }

    fn move_item(&mut self, src: &str, dst: &str) -> Result<(), &'static str> {
        if src == dst {
            return Ok(());
        }

        let current = self.get_current_dir_mut();
        
        let item = current.children.remove(src)
            .ok_or("Source not found")?;
        
        current.children.insert(dst.to_string(), item);
        Ok(())
    }

    fn copy_item(&mut self, src: &str, dst: &str, _recursive: bool) -> Result<(), &'static str> {
        if src == dst {
            return Err("Source and destination are the same");
        }

        let current = self.get_current_dir();
        let item = current.children.get(src)
            .ok_or("Source not found")?;

        let new_item = FileSystemItem {
            name: dst.to_string(),
            is_directory: item.is_directory,
            size: item.size,
            contents: item.contents.clone(),
            children: HashMap::new(), // Simplified: not actually copying children
        };

        let current = self.get_current_dir_mut();
        current.children.insert(dst.to_string(), new_item);
        Ok(())
    }

    fn get_current_path(&self) -> String {
        if self.current_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", self.current_path.join("/"))
        }
    }
}

fn format_size(size: u32) -> String {
    const KB: u32 = 1024;
    const MB: u32 = KB * 1024;

    if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

fn main() {
    let mut fs = FileSystem::new();
    
    println!("🚀 TerraOS File System Command Test\n");
    
    // Test 1: ls command
    println!("📁 Test 1: ls command");
    match fs.list_directory(false, false) {
        Ok(entries) => {
            println!("Current directory contents:");
            for entry in entries {
                println!("  {}", entry);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 2: ls -l command
    println!("\n📁 Test 2: ls -l command");
    match fs.list_directory(false, true) {
        Ok(entries) => {
            println!("Detailed directory contents:");
            for entry in entries {
                println!("  {}", entry);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 3: create directory
    println!("\n📁 Test 3: create directory");
    match fs.create_directory("test_dir") {
        Ok(_) => println!("✓ Directory 'test_dir' created"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 4: ls after create
    println!("\n📁 Test 4: ls after create");
    match fs.list_directory(false, false) {
        Ok(entries) => {
            for entry in entries {
                println!("  {}", entry);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
    
    // Test 5: change directory
    println!("\n📁 Test 5: change directory");
    match fs.change_directory("test_dir") {
        Ok(_) => {
            println!("✓ Changed to directory 'test_dir'");
            println!("Current path: {}", fs.get_current_path());
        },
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 6: move back
    println!("\n📁 Test 6: move back");
    match fs.change_directory("..") {
        Ok(_) => {
            println!("✓ Moved back to parent");
            println!("Current path: {}", fs.get_current_path());
        },
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 7: copy file
    println!("\n📁 Test 7: copy file");
    match fs.copy_item("README.txt", "README_copy.txt", false) {
        Ok(_) => println!("✓ Copied 'README.txt' to 'README_copy.txt'"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 8: move file
    println!("\n📁 Test 8: move file");
    match fs.move_item("README_copy.txt", "MOVED_README.txt") {
        Ok(_) => println!("✓ Moved 'README_copy.txt' to 'MOVED_README.txt'"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 9: delete file
    println!("\n📁 Test 9: delete file");
    match fs.delete_item("MOVED_README.txt", false) {
        Ok(_) => println!("✓ Deleted 'MOVED_README.txt'"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 10: final ls
    println!("\n📁 Test 10: final ls -l");
    match fs.list_directory(false, true) {
        Ok(entries) => {
            println!("Final directory contents:");
            for entry in entries {
                println!("  {}", entry);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n🎉 All tests completed!");
}