// Simple test for filesystem functionality
// This is a standalone test to verify our filesystem logic

#[derive(Debug)]
struct TestFileSystem {
    directories: Vec<String>,
}

impl TestFileSystem {
    fn new() -> Self {
        TestFileSystem {
            directories: Vec::new(),
        }
    }

    fn create_directory(&mut self, name: &str) -> Result<(), &'static str> {
        if name.is_empty() {
            return Err("Directory name cannot be empty");
        }
        
        if self.directories.contains(&name.to_string()) {
            return Err("Directory already exists");
        }
        
        self.directories.push(name.to_string());
        Ok(())
    }

    fn list_directories(&self) -> Vec<&String> {
        self.directories.iter().collect()
    }

    fn remove_directory(&mut self, name: &str) -> Result<(), &'static str> {
        let index = self.directories.iter().position(|d| d == name);
        match index {
            Some(i) => {
                self.directories.remove(i);
                Ok(())
            },
            None => Err("Directory not found"),
        }
    }
}

fn main() {
    let mut fs = TestFileSystem::new();
    
    // Test 1: Create directory
    match fs.create_directory("test_dir") {
        Ok(_) => println!("✓ Directory 'test_dir' created successfully"),
        Err(e) => println!("✗ Error creating directory: {}", e),
    }
    
    // Test 2: List directories
    println!("Directories: {:?}", fs.list_directories());
    
    // Test 3: Create duplicate directory
    match fs.create_directory("test_dir") {
        Ok(_) => println!("✗ Duplicate directory created unexpectedly"),
        Err(e) => println!("✓ Duplicate directory correctly rejected: {}", e),
    }
    
    // Test 4: Create another directory
    match fs.create_directory("another_dir") {
        Ok(_) => println!("✓ Directory 'another_dir' created successfully"),
        Err(e) => println!("✗ Error creating directory: {}", e),
    }
    
    // Test 5: List all directories
    println!("All directories: {:?}", fs.list_directories());
    
    // Test 6: Remove directory
    match fs.remove_directory("test_dir") {
        Ok(_) => println!("✓ Directory 'test_dir' removed successfully"),
        Err(e) => println!("✗ Error removing directory: {}", e),
    }
    
    // Test 7: List after removal
    println!("Directories after removal: {:?}", fs.list_directories());
}