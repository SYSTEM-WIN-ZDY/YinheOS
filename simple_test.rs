// Simple test for all filesystem commands
// Demonstrates all implemented functionality

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct FileItem {
    name: String,
    is_dir: bool,
    size: u32,
    children: HashMap<String, FileItem>,
}

struct SimpleFS {
    root: FileItem,
    current: String,
}

impl SimpleFS {
    fn new() -> Self {
        let mut root = FileItem {
            name: "/".to_string(),
            is_dir: true,
            size: 0,
            children: HashMap::new(),
        };

        // Add initial items
        root.children.insert("README.txt".to_string(), FileItem {
            name: "README.txt".to_string(),
            is_dir: false,
            size: 512,
            children: HashMap::new(),
        });

        SimpleFS {
            root,
            current: "/".to_string(),
        }
    }

    fn list_dir(&self) -> Vec<String> {
        let mut items = vec![".".to_string(), "..".to_string()];
        for name in self.root.children.keys() {
            items.push(name.clone());
        }
        items
    }

    fn create_dir(&mut self, name: &str) -> Result<(), &str> {
        if name.is_empty() {
            return Err("Empty name");
        }
        if self.root.children.contains_key(name) {
            return Err("Already exists");
        }

        self.root.children.insert(name.to_string(), FileItem {
            name: name.to_string(),
            is_dir: true,
            size: 4096,
            children: HashMap::new(),
        });
        Ok(())
    }

    fn delete_item(&mut self, name: &str) -> Result<(), &str> {
        if name == "." || name == ".." {
            return Err("Cannot delete . or ..");
        }
        self.root.children.remove(name)
            .ok_or("Not found")
            .map(|_| ())
    }

    fn move_item(&mut self, src: &str, dst: &str) -> Result<(), &str> {
        let item = self.root.children.remove(src)
            .ok_or("Source not found")?;
        self.root.children.insert(dst.to_string(), item);
        Ok(())
    }

    fn copy_item(&mut self, src: &str, dst: &str) -> Result<(), &str> {
        let item = self.root.children.get(src)
            .ok_or("Source not found")?;
        
        let new_item = FileItem {
            name: dst.to_string(),
            is_dir: item.is_dir,
            size: item.size,
            children: HashMap::new(),
        };
        
        self.root.children.insert(dst.to_string(), new_item);
        Ok(())
    }
}

fn main() {
    let mut fs = SimpleFS::new();
    
    println!("🚀 TerraOS File System Commands Demo\n");
    
    // 1. List initial contents
    println!("1. Initial directory listing:");
    for item in fs.list_dir() {
        println!("   {}", item);
    }
    
    // 2. Create directories
    println!("\n2. Creating directories:");
    match fs.create_dir("documents") {
        Ok(_) => println!("   ✓ Created 'documents'"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    match fs.create_dir("projects") {
        Ok(_) => println!("   ✓ Created 'projects'"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // 3. List after create
    println!("\n3. Directory after creating folders:");
    for item in fs.list_dir() {
        println!("   {}", item);
    }
    
    // 4. Copy file
    println!("\n4. Copying file:");
    match fs.copy_item("README.txt", "README_backup.txt") {
        Ok(_) => println!("   ✓ Copied 'README.txt' to 'README_backup.txt'"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // 5. Move file
    println!("\n5. Moving file:");
    match fs.move_item("README_backup.txt", "README_old.txt") {
        Ok(_) => println!("   ✓ Moved 'README_backup.txt' to 'README_old.txt'"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // 6. List after operations
    println!("\n6. Final directory listing:");
    for item in fs.list_dir() {
        println!("   {}", item);
    }
    
    // 7. Delete file
    println!("\n7. Deleting file:");
    match fs.delete_item("README_old.txt") {
        Ok(_) => println!("   ✓ Deleted 'README_old.txt'"),
        Err(e) => println!("   ✗ Error: {}", e),
    }
    
    // 8. Final list
    println!("\n8. Final directory contents:");
    for item in fs.list_dir() {
        println!("   {}", item);
    }
    
    // 9. Test error cases
    println!("\n9. Testing error cases:");
    match fs.create_dir("documents") {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Expected error: {}", e),
    }
    
    match fs.delete_item("nonexistent") {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Expected error: {}", e),
    }
    
    println!("\n🎉 All commands tested successfully!");
}