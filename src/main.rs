// COP4520 HW 1: Finding Primes
// Using a multithreaded segmented Sieve of Eratosthenes
// to find the sum of all primes under 100,000,000 and the
// 10 biggest primes within that
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;
use std::fs;

fn main() {
    segmented((100_000_000 as f64) as i32);
}

// Simple Sieve of Eratosthenes implemented from pseudocode on Wikipedia
fn sieve(limit: i32) -> Vec<i32> {
    let mut prime = vec![true; limit as usize];
    let mut num = 2;
    while num * num <= limit {
        if prime[num as usize] {
            for i in ((num * num)..limit).step_by(num as usize) {
                prime[i as usize] = false;
            }
        }
        num += 1;
    }
    let mut prime_ints = Vec::new();
    for i in 2..limit {
        if prime[i as usize] {
            // println!("{}", i);
            prime_ints.push(i);
        }
    }
    return prime_ints;
}

// Segmented Sieve also described on Wikipedia
fn segmented(limit: i32) {
    // According to wikipedia we only need to calculate up to sqrt n primes
    // before we can segment the rest out
    let sqrt_n = (limit as f32).sqrt() as i32 + 1;
    let sqrt_n_sieve = Arc::new(sieve(sqrt_n));

    // Distribute numbers amongst the threads and set num_threads
    let num_threads = 8;
    let segment_size = (limit - sqrt_n) / num_threads;

    // Create holder for threads and initialize shared primes vector
    let mut jobs: Vec<JoinHandle<()>> = Vec::new();
    let mut primes_vector = vec![];
    primes_vector.extend(sqrt_n_sieve.iter());
    let primes = Arc::new(Mutex::new(primes_vector));

    // Track time
    let now = Instant::now();

    // Spawn num_threads
    for i in 0..num_threads {
        let sieved: Arc<Vec<i32>> = sqrt_n_sieve.clone();
        let primes_arc = Arc::clone(&primes);
        jobs.push(
            thread::spawn(move || {
                let high = if sqrt_n + segment_size * (i + 1) < limit { sqrt_n + segment_size * (i + 1)} else { limit };
                let mut segment_primes = calculate_segment(sqrt_n + segment_size * i, high, &sieved);
                // Use Mutex to add things to shared primes vector after all computation is done
                let mut primes_vector = primes_arc.lock().unwrap();
                primes_vector.append(& mut segment_primes);
            })
        );
    }
    // Wait for threads to finish
    for thread in jobs.into_iter(){
        thread.join().unwrap();
    }

    // Get elapsed time
    let time = now.elapsed();

    // Get the sum of primes and the max 10 primes
    let mut max_10: VecDeque<i32> = VecDeque::new();
    let mut sum_of_primes: i64 = 0;
    let mut num_of_primes = 0;
    for prime in primes.lock().unwrap().iter() {
        num_of_primes += 1;
        sum_of_primes += *prime as i64;
        if max_10.len() == 0 { max_10.push_front(*prime)}
        if prime > max_10.front().unwrap() {
            max_10.push_front(*prime);
        }
        if max_10.len() > 10 { max_10.pop_back(); }
    }
    // Write output to file
    let mut output = format!("{} {} {}\n", time.as_secs_f64(), num_of_primes, sum_of_primes);
    for max_prime in max_10.iter().rev() {
        output.push_str(&*format!("{},", max_prime));
    }
    // println!("{}", output);
    fs::write("primes.txt", output).expect("Can not write file");
    // print!("10 max primes: ");
    // for max_prime in max_10 {
    //     print!("{}, ", max_prime);
    // }
    // println!("\nSum: {}", sum_of_primes);
}

fn calculate_segment(low: i32, high: i32, initial_primes: &Vec<i32>) -> Vec<i32> {
    // Make bool list the size of this segment
    let mut primes = vec![true; (high - low + 1) as usize];
    for prime in initial_primes {
        // Lowest multiple of this prime within this segment
        // Add the prime to the lowest multiple to bring it within the segment if not in it
        let lowest_multiple = if (low / *prime) * *prime >= low { (low / *prime) * *prime } else { (low / *prime) * *prime + *prime };
        // Mark all multiples of this prime as not prime
        for i in (lowest_multiple..(high+1)).step_by(*prime as usize) {
            primes[(i - low) as usize] = false;
        }
    }
    // Make and return list of ints after marking non-primes
    let mut prime_ints = Vec::new();
    for i in low..(high+1) {
        if primes[(i - low) as usize] {
            prime_ints.push(i);
        }
    }
    return prime_ints;
}