use std::arch::x86_64::*;
use std::collections::BinaryHeap;

use hdrhistogram::*;
use rand::prelude::*;
use min_max_heap::MinMaxHeap;


fn get_ts() -> u64 {
    unsafe {
        let mut val = 0;
        __rdtscp(&mut val)
    }
}

struct CombHeap {
    largest_vals: MinMaxHeap::<u64>,
    rest: BinaryHeap::<u64>,
}

const LARGE_MAX_SIZE: usize = 64;

impl CombHeap {
    pub fn new() -> CombHeap {
        CombHeap {
            largest_vals: MinMaxHeap::new(),
            rest: BinaryHeap::new(),
        }
    }

    pub fn pop(&mut self) -> Option<u64> {
        if let Some(rval) = self.largest_vals.pop_max() {
            if self.largest_vals.len() == 0 {
                if let Some(rest) = self.rest.pop() {
                    self.largest_vals.push(rest);
                }
            }
            Some(rval)
        } else {
            None
        }
    }

    pub fn push(&mut self, val: u64) {
        if self.largest_vals.len() == 0 {
            self.largest_vals.push(val);
        } else {
            let small_too_big = self.largest_vals.len() >= LARGE_MAX_SIZE;
            let belongs_small =
                val >= *self.largest_vals.peek_min().unwrap() ||
                (!small_too_big && (self.rest.len() > 0 && val >= *self.rest.peek().unwrap()));
            if belongs_small {
                if small_too_big {
                    self.rest.push(self.largest_vals.pop_min().unwrap());
                }
                self.largest_vals.push(val);
            } else {
                self.rest.push(val);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.rest.len() + self.largest_vals.len()
    }

    pub fn peek(&self) -> Option<&u64> {
        self.largest_vals.peek_max()
    }
}

const HEAP_COUNT: usize = 1000;
const HEAP_SIZE: usize = 100000;
const HEAP_OVERHANG: usize = HEAP_SIZE/10;

const ITERS: usize = 100000000;

fn print_cycles(val: f64, hist: &Histogram::<u64>) {
    let cycles = hist.value_at_quantile(val);
    println!("{}th percentile: {} cycles, ~{}ns", val*100.0, cycles, cycles/4);
}

fn main() {
    let mut hist = Histogram::<u64>::new_with_bounds(1, /* 20 ms */ 4 * 1000 * 1000 * 20, 5).unwrap();
    let mut heaps = Vec::new();
    for i in 0..HEAP_COUNT {
        let mut heap = CombHeap::new();
        for i in 0..HEAP_SIZE {
            heap.push(rand::random());
        }
        heaps.push(heap);
    }

    let mut rng = rand::thread_rng(); 

    for i in 0..ITERS {
        let which = rand::random::<usize>() % HEAP_COUNT;
        let heap = &mut heaps[which];
        let add = if heap.len() < HEAP_SIZE - HEAP_OVERHANG {
            true
        } else if heap.len() > HEAP_SIZE + HEAP_OVERHANG {
            false
        } else {
            rand::random()
        };
        if add {
            let chance: f64 = rng.gen();
            if chance > 0.50 {
                heap.push(rand::random());
            } else {
                heap.push(heap.peek().unwrap() + 1)
            }
        } else {
            let start = get_ts();
            heap.pop();
            let end = get_ts();
            if end > start {
                let time = end - start;
                hist.record(time);
            }
        };

    }

    println!("# of samples: {}", hist.len());
    print_cycles(0.1, &hist);
    print_cycles(0.25, &hist);
    print_cycles(0.5, &hist);
    print_cycles(0.75, &hist);
    print_cycles(0.90, &hist);
    print_cycles(0.95, &hist);
    print_cycles(0.99, &hist);
    print_cycles(0.999, &hist);
    print_cycles(0.9999, &hist);
}
