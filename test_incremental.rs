// Test file for incremental updates
pub fn test_function() {
    println!("Testing incremental updates!");
}

pub struct TestStruct {
    pub field: String,
}

impl TestStruct {
    pub fn new() -> Self {
        TestStruct {
            field: "test".to_string(),
        }
    }
    
    pub fn process(&self) {
        println!("Processing: {}", self.field);
    }
}