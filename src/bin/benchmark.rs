// =============================================================
// Benchmark: sequential vs basic vs parallel
// Scenario A: light work per line (I/O-dominated)
// Scenario B: heavy CPU work per line (CPU-dominated)
// =============================================================

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

const RUNS: u32 = 5;

// ── Light work: simple checksum ───────────────────────────────
fn light_work(s: &str) -> u64 {
    s.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64))
}

// ── Heavy work: repeated hashing (simulates real processing) ──
fn heavy_work(s: &str) -> u64 {
    let mut h = s.len() as u64;
    for _ in 0..5_000 {
        for b in s.bytes() {
            h = h.wrapping_mul(6364136223846793005)
                 .wrapping_add(b as u64 ^ 1442695040888963407);
        }
    }
    h
}

// ── Sequential baseline ───────────────────────────────────────
fn run_sequential<F: Fn(&str) -> u64>(path: &str, work: &F) -> Duration {
    let t = Instant::now();
    let file = File::open(path).unwrap();
    let _: u64 = BufReader::new(file)
        .lines()
        .map(|l| work(&l.unwrap()))
        .sum();
    t.elapsed()
}

// ── Basic: 2 threads producer-consumer ───────────────────────
fn run_basic<F: Fn(&str) -> u64 + Send + 'static>(path: &str, work: F) -> Duration {
    let t = Instant::now();
    let path = path.to_string();
    let (tx, rx) = mpsc::channel::<String>();

    let reader = thread::spawn(move || {
        let file = File::open(&path).unwrap();
        for line in BufReader::new(file).lines() {
            tx.send(line.unwrap()).unwrap();
        }
    });

    let processor = thread::spawn(move || -> u64 {
        rx.into_iter().map(|l| work(&l)).sum()
    });

    reader.join().unwrap();
    let _ = processor.join().unwrap();
    t.elapsed()
}

// ── Parallel: N threads, each owns a chunk ───────────────────
fn run_parallel<F: Fn(&str) -> u64 + Send + Sync + 'static + ?Sized>(
    path: &str,
    num_threads: usize,
    work: Arc<F>,
) -> Duration {
    let t = Instant::now();

    let file = File::open(path).unwrap();
    let lines: Arc<Vec<String>> = Arc::new(
        BufReader::new(file).lines().map(|l| l.unwrap()).collect(),
    );
    let total = lines.len();
    let chunk = (total + num_threads - 1) / num_threads;

    let handles: Vec<_> = (0..num_threads)
        .map(|tid| {
            let start = tid * chunk;
            let end = (start + chunk).min(total);
            if start >= total { return thread::spawn(|| 0u64); }
            let data = Arc::clone(&lines);
            let w = Arc::clone(&work);
            thread::spawn(move || -> u64 {
                data[start..end].iter().map(|l| w(l)).sum()
            })
        })
        .collect();

    let _: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    t.elapsed()
}

fn avg_run<F: Fn() -> Duration>(f: F) -> Duration {
    (0..RUNS).map(|_| f()).sum::<Duration>() / RUNS
}
fn fmt(d: Duration) -> String { format!("{:>8.3} ms", d.as_secs_f64() * 1000.0) }
fn spd(base: Duration, cmp: Duration) -> f64 { base.as_secs_f64() / cmp.as_secs_f64() }
fn bar(ratio: f64) -> String {
    let n = (ratio.min(3.0) / 3.0 * 24.0).round() as usize;
    "█".repeat(n)
}

fn main() {
    let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    println!("CPU cores : {}   |   Runs per config : {}", ncpu, RUNS);

    let files = [
        ("1k lines ",  "data/sample.txt"),
        ("100k lines", "data/large.txt"),
    ];
    let thread_counts: &[usize] = &[1, 2, 4, 8];

    for scenario in ["light", "heavy"] {
        let cpu_desc = if scenario == "light" {
            "Light CPU (checksum)     — I/O-dominated"
        } else {
            "Heavy CPU (5k hash iters)— CPU-dominated"
        };
        println!("\n══════════════════════════════════════════════════════════");
        println!(" Scenario: {}", cpu_desc);
        println!("══════════════════════════════════════════════════════════");
        println!("{:<24} {:>10}  {:>6}  visual (speedup vs seq)", "Config", "Avg time", "Speedup");
        println!("{}", "─".repeat(62));

        for (label, path) in &files {
            println!("  ┌ {} ──────────────────────────────────", label);

            let seq = if scenario == "light" {
                avg_run(|| run_sequential(path, &light_work))
            } else {
                avg_run(|| run_sequential(path, &heavy_work))
            };

            println!("  │ {:<22} {}   1.00x  {}", "sequential", fmt(seq), bar(1.0));

            let basic_time = if scenario == "light" {
                avg_run(|| run_basic(path, light_work))
            } else {
                avg_run(|| run_basic(path, heavy_work))
            };
            let bs = spd(seq, basic_time);
            println!("  │ {:<22} {}  {:>5.2}x  {}", "basic (2 threads)", fmt(basic_time), bs, bar(bs));

            for &n in thread_counts {
                let work_arc: Arc<dyn Fn(&str) -> u64 + Send + Sync> = if scenario == "light" {
                    Arc::new(light_work)
                } else {
                    Arc::new(heavy_work)
                };
                let par = avg_run(|| run_parallel(path, n, Arc::clone(&work_arc)));
                let sp = spd(seq, par);
                println!("  │ {:<22} {}  {:>5.2}x  {}",
                    format!("parallel {} threads", n), fmt(par), sp, bar(sp));
            }
            println!("  └{}", "─".repeat(61));
        }
    }

    println!("\n Key insight:");
    println!("  • Speedup < 1.0  → slower than sequential (thread overhead > gain)");
    println!("  • Speedup = N     → perfect linear scaling across N cores");
    println!("  • I/O scenario:  OS page-cache makes reads near-instant; threads add overhead");
    println!("  • CPU scenario:  real computation saturates a single core; more threads = faster");
}

