# Memory and Performance Guide: Rust on Resource-Constrained Environments (0.1 vCPU, 512 MB RAM)

> Target Environment: **Koyeb Free Tier** (0.1 vCPU, 512 MB RAM, Containerized Linux / cgroups)  
> Application: **Haven** (`haven-web`, `haven-domain`, `haven-db`)

---

## 1. Executive Summary & Mental Model

When running a production Rust web service inside a **512 MB RAM / 0.1 vCPU** container, the performance and memory rules change fundamentally:

1. **Stack micro-differences do NOT cause OOMs:**
   The 8-byte difference between a `Vec<T>` (24 bytes) and a `Box<[T]>` (16 bytes) on the stack is negligible. 10,000 stack references differ by only 80 KB (0.015% of total memory).
2. **What DOES cause OOMs (The Out-Of-Memory Killer):**
   - Allocator memory retention & fragmentation (`glibc` arena caching).
   - Unbounded thread stack multiplication (Tokio spawning 32 threads on a 32-core host node).
   - Oversized database connection pools buffering data.
   - Unbounded dynamic buffers (`Vec` doubling, large multipart file uploads in RAM).
3. **What causes CPU throttling on 0.1 vCPU:**
   - Excessive OS thread context switching.
   - Small allocation churn triggering memory page faults.
   - Polling loops without proper backoff or yield.

---

## 2. Deep Dive: Data Structures & Their Cost Profile

### 2.1. Memory Layouts on 64-Bit Systems

| Type | Stack Size | Heap Allocation | Capacity Growth | Best Use Case |
| :--- | :--- | :--- | :--- | :--- |
| **`[T; N]`** | $N \times \text{size\_of}(T)$ | **0 bytes** | Fixed (Compile-time) | Known static bounds (e.g. SHA-256 hash `[u8; 32]`, UUID bytes `[u8; 16]`) |
| **`&[T]` / `&str`** | 16 bytes (`ptr`, `len`) | **0 bytes** | Immutable borrow | **All function inputs** reading sequential data |
| **`SmallVec<[T; N]>`** | Inline buffer + tag | **0 bytes** (if $\le N$)<br>1 heap buffer (if $> N$) | Dynamic | Collections that are almost always small (e.g., 0–4 offer images) |
| **`Vec<T>`** | 24 bytes (`ptr`, `cap`, `len`) | $1 \times (\text{cap} \times \text{size})$ | Amortized $O(1)$ (doubles) | General dynamic collection being constructed or modified |
| **`Box<[T]>`** | 16 bytes (`ptr`, `len`) | $1 \times (\text{len} \times \text{size})$ | Fixed (Requires realloc) | Long-lived immutable arrays held in memory (zero spare capacity) |
| **`Box<T>`** | 8 bytes (`ptr`) | $1 \times \text{size\_of}(T)$ | N/A (Single item) | Recursive types, trait objects (`Box<dyn Port>`), large single structs |
| **`Arc<[T]>`** | 8 bytes (`ptr`) | $1 \times (\text{header} + \text{len} \times \text{size})$ | Fixed | Shared across async tasks without cloning buffer data |

### 2.2. Why `Vec<T>` is the Standard Default
- **Contiguous memory:** Highest possible CPU cache locality.
- **Universal compatibility:** Serde, SQLx, and the standard library deserialize and collect into `Vec`.
- **Amortized growth:** Adding items with `.push()` does not trigger an OS syscall or allocator lock on every call.

### 2.3. The Real Danger of `Vec<T>` on 512 MB: Capacity Doubling
When a `Vec` expands beyond its capacity, the allocator requests a buffer **twice as large**, copies existing items, and frees the old buffer:
$$\text{Capacity progression: } 0 \to 4 \to 8 \to 16 \to 32 \to 64 \to 128 \to \dots$$

If you collect 1,025 elements into a `Vec`, its capacity jumps to 2,048. For 1,023 elements, **the heap buffer is 50% empty wasted memory**.
- **Rule:** When the upper bound is known, always write `Vec::with_capacity(expected_count)`.

### 2.4. Zero-Cost Pattern: Iterators vs. Collecting
```rust
// ❌ BAD: Allocates a heap buffer, fills it, and drops it
let ids: Vec<Uuid> = offers.iter().map(|o| o.id).collect();
for id in ids {
    process(id);
}

//  ZERO-COST: 0 heap allocations, streaming iteration directly from source
for id in offers.iter().map(|o| o.id) {
    process(id);
}
```

---

## 3. The 4 Critical Resource Traps on Koyeb Free Tier

### 3.1. Trap 1: Tokio Thread Pool Oversizing
* **The Root Cause:** `#[tokio::main]` initializes worker threads based on the number of CPU cores it detects. On containerized platforms, the host machine often has **16, 32, or 64 physical cores**, while your cgroup container is throttled to **0.1 vCPU**.
* **The Impact:**
  - Tokio spawns 16–32 threads.
  - Each thread allocates an OS stack (typically 2 MB).
  - 32 threads $\times$ 2 MB = **64 MB of RAM consumed just by idle thread stacks** (12.5% of your entire 512 MB limit).
  - 32 threads fighting for 0.1 vCPU leads to intense CPU throttling and context-switch latency.
* **The Fix:**
  Explicitly limit Tokio worker threads to 1 or 2 in `crates/haven-web/src/main.rs`:
  ```rust
  #[tokio::main(flavor = "multi_thread", worker_threads = 2)]
  async fn main() -> Result<(), Box<dyn std::error::Error>> { ... }
  ```

### 3.2. Trap 2: Database Connection Pool Bloat
* **The Root Cause:** `DB_POOL_SIZE` in `haven-web` currently defaults to 20 connections.
* **The Impact:**
  - Each connection maintains active socket buffers in both PostgreSQL and `haven-web`.
  - 20 connections running concurrent queries on 0.1 vCPU causes CPU thrashing without improving throughput.
* **The Fix:**
  Set `DB_POOL_SIZE=3` (or maximum 5) in Koyeb environment variables:
  ```env
  DB_POOL_SIZE=3
  ```

### 3.3. Trap 3: Glibc Malloc Memory Fragmentation
* **The Root Cause:** Standard Linux builds use GNU Libc (`glibc`). Glibc creates separate memory arenas per thread and **relies on heuristics that rarely return freed memory pages back to the Linux kernel**.
* **The Impact:**
  Even when your active data is only 25 MB, temporary memory spikes (deserializing JSON, parsing images) cause RSS (Resident Set Size) to climb to **200 MB – 400 MB** and stay there, eventually triggering an OOM kill.
* **The Fix:**
  Use `tikv-jemallocator` or `mimalloc` as the global allocator, configured with aggressive background page purging:
  ```rust
  // In crates/haven-web/src/main.rs
  #[global_allocator]
  static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
  ```
  And set the environment variable in your Dockerfile / Koyeb service:
  ```env
  MALLOC_CONF="background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:1000"
  ```
  This forces the allocator to return unused virtual memory to the OS within 1 second.

### 3.4. Trap 4: In-Memory File & Image Buffering
* **The Root Cause:** Reading full image payloads into `Vec<u8>` in RAM during upload.
* **The Impact:**
  If two users upload 5 MB photos simultaneously:
  - Incoming HTTP multipart buffer: $2 \times 5\text{ MB} = 10\text{ MB}$
  - Extracted image vector: $2 \times 5\text{ MB} = 10\text{ MB}$
  - Outgoing reqwest payload to Cloudinary: $2 \times 5\text{ MB} = 10\text{ MB}$
  - **Spike:** ~30 MB of RAM for just two concurrent requests.
* **The Fix:**
  - Enforce a strict request body size limit in web handlers (e.g., maximum 2 MB or 3 MB per image).
  - Stream multipart directly to external storage whenever possible, or avoid duplicating byte vectors in memory.

---

## 4. Codebase Decision Checklist

| Scenario | Recommended Choice | Avoid | Reason |
| :--- | :--- | :--- | :--- |
| **Function parameter** | `&[T]` or `&str` | `Vec<T>` or `String` | Zero-cost borrowing; allows caller to pass arrays, slices, or vectors without allocation. |
| **Image collection per offer** | `SmallVec<[OfferImage; 4]>` or `Vec<OfferImage>` with known capacity | Unbounded growing `Vec` | 95%+ of offers have 1–4 images; avoids heap overhead or excess capacity. |
| **Fixed cryptographic hashes** | `[u8; 32]` or `[u8; 16]` | `Vec<u8>` | Stack-allocated, zero heap, compiler-enforced length. |
| **Database list queries** | Paginated `Vec<T>` (strict `LIMIT`) | Unbounded `.fetch_all()` | Prevents table-scale memory dumping into process RAM. |
| **Intermediate transformations** | Iterator chaining (`.iter().map().filter()`) | `.collect::<Vec<_>>()` | Eliminates temporary heap buffers. |
| **Long-lived shared config/cache** | `Arc<[T]>` or `Arc<str>` | Cloned `Vec<T>` | 8-byte pointer sharing; eliminates duplicate heap buffers across tasks. |

---

## 5. Deployment Environment Configuration (Koyeb Free Tier)

Ensure these environment variables are set in your Koyeb service dashboard or `app.yaml`:

```env
# Concurrency & CPU constraint (0.1 vCPU)
TOKIO_WORKER_THREADS=2

# Database connection pool (Postgres)
DB_POOL_SIZE=3

# Memory allocator page reclamation (if using jemalloc)
MALLOC_CONF="background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:1000"

# Application environment
APP_ENV=production
```
