use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, null_mut};
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub const HEAP_START: usize = 0x_500_000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

struct ListNode {
    size: usize,
    next: Option<*mut ListNode>,
}

pub struct LinkedListAllocator {
    head: AtomicUsize,
    // 内存统计
    total_allocated: AtomicU64,
    total_freed: AtomicU64,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
    current_allocated: AtomicU64,
    max_allocated: AtomicU64,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator { 
            head: AtomicUsize::new(0),
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            current_allocated: AtomicU64::new(0),
            max_allocated: AtomicU64::new(0),
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&self, addr: usize, size: usize) {
        // Ensure the region is large enough to hold a ListNode
        assert!(size >= core::mem::size_of::<ListNode>());

        let node = ListNode { size, next: None };
        let node_ptr = addr as *mut ListNode;
        ptr::write(node_ptr, node);

        self.add_node(node_ptr);
    }

    unsafe fn add_node(&self, node_ptr: *mut ListNode) {
        let mut current_head = self.head.load(Ordering::SeqCst);
        loop {
            (*node_ptr).next = if current_head == 0 { None } else { Some(current_head as *mut ListNode) };
            match self.head.compare_exchange_weak(current_head, node_ptr as usize, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(x) => current_head = x,
            }
        }
    }

    unsafe fn find_free_region(&self, size: usize, align: usize) -> Option<(*mut ListNode, *mut ListNode)> {
        let mut current_head = self.head.load(Ordering::SeqCst);
        let mut prev_node_ptr: *mut ListNode = null_mut();

        while current_head != 0 {
            let current_node_ptr = current_head as *mut ListNode;
            let current_node = &*current_node_ptr;

            let aligned_start = align_up(current_node_ptr as usize + core::mem::size_of::<ListNode>(), align);
            let required_size = size + (aligned_start - (current_node_ptr as usize + core::mem::size_of::<ListNode>()));

            if current_node.size >= required_size {
                // Found a suitable region
                return Some((prev_node_ptr, current_node_ptr));
            }

            prev_node_ptr = current_node_ptr;
            current_head = if let Some(next_node) = current_node.next { next_node as usize } else { 0 };
        }
        None
    }

    unsafe fn remove_node(&self, prev_node_ptr: *mut ListNode, node_ptr: *mut ListNode) {
        let next_node = (*node_ptr).next;
        if prev_node_ptr == null_mut() {
            // Removing the head node
            let new_head = if let Some(next) = next_node { next as usize } else { 0 };
            self.head.store(new_head, Ordering::SeqCst);
        } else {
            // Removing a non-head node
            (*prev_node_ptr).next = next_node;
        }
    }

    // 内存统计方法
    pub fn get_total_allocated(&self) -> u64 {
        self.total_allocated.load(Ordering::Relaxed)
    }

    pub fn get_total_freed(&self) -> u64 {
        self.total_freed.load(Ordering::Relaxed)
    }

    pub fn get_current_allocated(&self) -> u64 {
        self.current_allocated.load(Ordering::Relaxed)
    }

    pub fn get_max_allocated(&self) -> u64 {
        self.max_allocated.load(Ordering::Relaxed)
    }

    pub fn get_allocation_count(&self) -> u64 {
        self.allocation_count.load(Ordering::Relaxed)
    }

    pub fn get_deallocation_count(&self) -> u64 {
        self.deallocation_count.load(Ordering::Relaxed)
    }

    pub fn get_free_memory(&self) -> u64 {
        let mut current_head = self.head.load(Ordering::SeqCst);
        let mut total_free = 0;

        while current_head != 0 {
            let current_node_ptr = current_head as *mut ListNode;
            let current_node = &*current_node_ptr;
            total_free += current_node.size as u64;
            current_head = if let Some(next_node) = current_node.next { next_node as usize } else { 0 };
        }

        total_free
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_heap_size: HEAP_SIZE as u64,
            allocated: self.get_current_allocated(),
            freed: self.get_total_freed(),
            current_allocated: self.get_current_allocated(),
            max_allocated: self.get_max_allocated(),
            allocation_count: self.get_allocation_count(),
            deallocation_count: self.get_deallocation_count(),
            free_memory: self.get_free_memory(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_heap_size: u64,
    pub allocated: u64,
    pub freed: u64,
    pub current_allocated: u64,
    pub max_allocated: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub free_memory: u64,
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(core::mem::size_of::<ListNode>());
        let align = layout.align();

        if let Some((prev_node_ptr, node_ptr)) = self.find_free_region(size, align) {
            self.remove_node(prev_node_ptr, node_ptr);

            let node = &*node_ptr;
            let alloc_start = align_up(node_ptr as usize + core::mem::size_of::<ListNode>(), align);
            let alloc_end = alloc_start + size;

            let remaining_size = node.size - (alloc_end - node_ptr as usize);
            if remaining_size > 0 {
                self.add_free_region(alloc_end, remaining_size);
            }

            // 更新统计信息
            self.total_allocated.fetch_add(size as u64, Ordering::Relaxed);
            self.current_allocated.fetch_add(size as u64, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);
            
            // 更新最大分配记录
            let mut current_max = self.max_allocated.load(Ordering::Relaxed);
            loop {
                if self.current_allocated.load(Ordering::Relaxed) > current_max {
                    match self.max_allocated.compare_exchange_weak(
                        current_max, 
                        self.current_allocated.load(Ordering::Relaxed), 
                        Ordering::Relaxed, 
                        Ordering::Relaxed
                    ) {
                        Ok(_) => break,
                        Err(x) => current_max = x,
                    }
                } else {
                    break;
                }
            }

            alloc_start as *mut u8
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(core::mem::size_of::<ListNode>());
        let new_node_ptr = ptr as *mut ListNode;
        (*new_node_ptr).size = size;
        self.add_node(new_node_ptr);

        // 更新统计信息
        self.total_freed.fetch_add(size as u64, Ordering::Relaxed);
        self.current_allocated.fetch_sub(size as u64, Ordering::Relaxed);
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}