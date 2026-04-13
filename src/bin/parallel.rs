// =============================================================
// Kasus 2 — Program Paralel: Pembacaan File dengan N Thread
// File dibagi menjadi beberapa segmen; setiap thread membaca
// segmennya sendiri secara bersamaan, lalu hasilnya dikumpulkan
// dan ditampilkan ke layar secara terurut.
// =============================================================

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;

const NUM_THREADS: usize = 4;
const PATH: &str = "data/sample.txt";

fn main() {
    println!("=== Parallel File Reader ({} thread) ===\n", NUM_THREADS);

    // ── Langkah 1: Muat seluruh baris ke memori ──────────────
    let file = File::open(PATH).expect("Gagal membuka file");
    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .map(|l| l.expect("Gagal membaca baris"))
        .collect();

    let total = lines.len();
    println!("[Main] Total baris dalam file: {}\n", total);

    // Bungkus Vec dalam Arc agar dapat di-share ke banyak thread
    let lines = Arc::new(lines);

    let start = Instant::now();

    // ── Langkah 2: Bagi file & jalankan N reader thread ──────
    let (tx, rx) = mpsc::channel::<(usize, String)>(); // (nomor_baris, isi)
    let chunk_size = (total + NUM_THREADS - 1) / NUM_THREADS;
    let mut handles = Vec::new();

    for t in 0..NUM_THREADS {
        let start_idx = t * chunk_size;
        let end_idx = (start_idx + chunk_size).min(total);

        if start_idx >= total {
            break;
        }

        let tx_clone = tx.clone();
        let lines_clone = Arc::clone(&lines);

        let handle = thread::spawn(move || {
            println!(
                "[Thread {:2}] Membaca baris {:4} – {:4}  ({} baris)",
                t,
                start_idx,
                end_idx - 1,
                end_idx - start_idx
            );

            for i in start_idx..end_idx {
                tx_clone
                    .send((i, lines_clone[i].clone()))
                    .expect("Gagal mengirim data");
            }

            println!("[Thread {:2}] Selesai.", t);
        });

        handles.push(handle);
    }

    // Lepas tx asli agar rx tahu semua pengirim sudah selesai
    drop(tx);

    // ── Langkah 3: Kumpulkan & urutkan hasil ─────────────────
    let mut results: Vec<(usize, String)> = rx.into_iter().collect();
    results.sort_unstable_by_key(|&(idx, _)| idx);

    // Tunggu semua reader thread selesai
    for h in handles {
        h.join().expect("Reader thread panik");
    }

    let elapsed = start.elapsed();

    // ── Langkah 4: Tampilkan hasil ────────────────────────────
    println!("\n[Printer] Menampilkan {} baris:\n", results.len());
    for (_, line) in &results {
        println!("{}", line);
    }

    println!("\n========================================");
    println!("Jumlah thread  : {}", NUM_THREADS);
    println!("Total baris    : {}", total);
    println!("Waktu eksekusi : {:?}", elapsed);
    println!("========================================");
}
