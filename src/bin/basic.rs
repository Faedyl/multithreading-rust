// =============================================================
// Kasus 2 — Program Dasar: Dua Thread (Reader + Printer)
// Satu thread membaca file baris demi baris, thread lain
// menampilkan setiap baris ke layar melalui channel MPSC.
// =============================================================

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

fn main() {
    let path = "data/sample.txt";

    // Channel: reader mengirim String, printer menerima String
    let (tx, rx) = mpsc::channel::<String>();

    // ── Thread 1: Reader ─────────────────────────────────────
    let reader_thread = thread::spawn(move || {
        let file = File::open(path).expect("Gagal membuka file");
        let reader = BufReader::new(file);
        let mut count = 0usize;

        for line in reader.lines() {
            let line = line.expect("Gagal membaca baris");
            tx.send(line).expect("Gagal mengirim data ke channel");
            count += 1;
        }

        println!("[Reader] Selesai membaca {} baris.", count);
        // tx di-drop di sini → rx akan selesai iterasi
    });

    // ── Thread 2: Printer ────────────────────────────────────
    let printer_thread = thread::spawn(move || {
        let mut count = 0usize;

        for data in rx {
            println!("[Printer] {}", data);
            count += 1;
        }

        println!("[Printer] Selesai menampilkan {} baris.", count);
    });

    reader_thread.join().expect("Reader thread panik");
    printer_thread.join().expect("Printer thread panik");

    println!("\n[Main] Program selesai.");
}
