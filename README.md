# Tugas 1 — Kasus 2: Multithreading dengan Rust

Program Rust yang mendemonstrasikan konsep multithreading untuk membaca dan menampilkan isi file secara bersamaan menggunakan channel MPSC.

---

## Struktur Proyek

```
multithreading-rust/
├── Cargo.toml
├── data/
│   └── sample.txt        ← file input (1000 baris)
└── src/bin/
    ├── basic.rs          ← Program dasar: 2 thread
    ├── parallel.rs       ← Program paralel: N thread
    └── benchmark.rs      ← Benchmark perbandingan
```

---

## Program

### 1. `basic` — Dua Thread (Reader + Printer)

Dua thread berjalan bersamaan:
- **Thread Reader**: membaca `data/sample.txt` baris demi baris, lalu mengirim setiap baris ke channel.
- **Thread Printer**: menerima baris dari channel dan mencetaknya ke layar.

```bash
cargo run --bin basic
```

### 2. `parallel` — N Thread Paralel

File dibagi menjadi beberapa segmen, masing-masing dibaca oleh satu thread secara paralel. Hasilnya dikumpulkan, diurutkan ulang, lalu ditampilkan. Waktu eksekusi dicatat.

```bash
cargo run --bin parallel
```

Untuk mengubah jumlah thread, edit konstanta di `src/bin/parallel.rs`:

```rust
const NUM_THREADS: usize = 4;
```

---

## Konsep Utama

| Konsep | Penjelasan |
|---|---|
| `thread::spawn` | Membuat thread baru |
| `mpsc::channel` | Saluran komunikasi antar thread (many producer, single consumer) |
| `Arc` | Shared ownership data antar thread secara aman |
| `thread::join` | Menunggu thread selesai sebelum program berakhir |
