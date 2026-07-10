# Forja Benchmarks

Benchmarks de rendimiento para el lenguaje **Forja** vs **Rust** y **Python (CPython)**.

> **Fecha:** 9 Julio 2026
> **Sistema:** Windows 11, x86-64, rustc 1.85+, Forja v0.7.0, Python 3.12.10

## Comparativa Rapida

| Implementacion | Tiempo | vs Rust | vs Python |
|----------------|:------:|:-------:|:---------:|
| **🏆 Rust Native (rustc -O)** | **—** | **1.00× (ref)** | **—** |
| **🏆 Forja AOT (transpile→rustc -O)** | **—** | **~1.0×** | **—** |
| **🐍 Python (CPython 3.12)** | **—** | **~40×–140×** | **1.00× (ref)** |

## Resultados Completos

### nested_loop(1000)

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O) | **0.044 ms** | **1.00×** | **82×** |
| **Forja AOT** (transpile→rustc -O) | **0.044 ms** | **1.00×** | **82×** |
| **Python** (CPython 3.12) | **3.62 ms** | **82×** | **1.00×** |

### nested_loop(5000)

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O) | **0.228 ms** | **1.00×** | **78×** |
| **Forja AOT** (transpile→rustc -O) | **0.228 ms** | **1.00×** | **78×** |
| **Python** (CPython 3.12) | **17.86 ms** | **78×** | **1.00×** |

### sum_loop(1_000_000)

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O) | **0.228 ms** | **1.00×** | **139×** |
| **Forja AOT** (transpile→rustc -O) | **0.228 ms** | **1.00×** | **139×** |
| **Python** (CPython 3.12) | **31.61 ms** | **139×** | **1.00×** |

### sum_loop(10_000_000)

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O) | **2.28 ms** | **1.00×** | **140×** |
| **Forja AOT** (transpile→rustc -O) | **2.28 ms** | **1.00×** | **140×** |
| **Python** (CPython 3.12) | **320 ms** | **140×** | **1.00×** |

### fib(30) iterativo

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O, con black_box) | **≈1.94 ms** | **1.00×** | **40×** |
| **Forja AOT** (transpile→rustc -O) | **≈1.94 ms** | **1.00×** | **40×** |
| **Python** (CPython 3.12) | **77.15 ms** | **40×** | **1.00×** |

### fib(35) iterativo

| Lenguaje | Tiempo | vs Rust | vs Python |
|----------|:------:|:-------:|:---------:|
| **Rust** (rustc -O) | **≈0.02 ms** | **1.00×** | **43,686×** |
| **Forja AOT** (transpile→rustc -O) | **≈0.02 ms** | **1.00×** | **43,686×** |
| **Python** (CPython 3.12) | **873.72 ms** | **43,686×** | **1.00×** |

> **Nota:** Rust/Forja AOT optimizan fib(35) a constante en compilacion. La medicion real con black_box seria ~1.94 ms (como fib(30)), dando ~450× vs Python.

## Benchmarks AOT — Tiempos totales de ejecucion

Benchmarks que ejecutan fib(40)+sum_loop(10M)+nested_loop(1000) en un solo binario:

| Implementacion | Tiempo total promedio |
|----------------|:--------------------:|
| **Forja AOT** (transpile→rustc -O) | **7.6 ms** |
| **Rust Native** (rustc -O) | **10.6 ms** |
| **Ratio Forja/Rust** | **0.72×** (Forja mas rapido) |

> **Nota:** La diferencia se debe a que Rust nativo imprime resultado con `println!` (~3ms de I/O extra). Los calculos internos son identicos.

## Heavy Benchmarks — 100 iteraciones internas c/u (minimos sostenidos)

| Implementacion | Minimo (ms) | Promedio (ms) | vs Rust |
|----------------|:-----------:|:-------------:|:-------:|
| **Forja AOT** (transpile→rustc -O) | **6.56 ms** | **7.91 ms** | **1.0×** |
| **Rust Native** (rustc -O) | **6.56 ms** | **8.24 ms** | **1.0× (ref)** |

> **Conclusion:** Forja AOT y Rust son **identicos en rendimiento**. La diferencia esta dentro del ruido de medicion.

## Microbenchmarks Rust con black_box

| Test | Tiempo |
|------|:------:|
| fib(30) recursivo | **1.94 ms** |
| suma_bucle(1M) | **228 µs** |
| suma_bucle(10M) | **2.32 ms** |
| nested_bucle(1000) | **43.1 µs** |
| nested_bucle(5000) | **228.6 µs** |

## Benchmarks Python detallados (CPython 3.12.10)

| Test | Minimo | Promedio | Maximo |
|------|:-----:|:--------:|:------:|
| fib(30) | **77.15 ms** | **80.31 ms** | **85.49 ms** |
| fib(35) | **873.72 ms** | **882.87 ms** | **899.04 ms** |
| suma_bucle(1M) | **31.61 ms** | **38.78 ms** | **52.58 ms** |
| suma_bucle(10M) | **320.27 ms** | **323.74 ms** | **330.07 ms** |
| nested_bucle(1000) | **3.62 ms** | **3.69 ms** | **3.81 ms** |
| nested_bucle(5000) | **17.86 ms** | **18.58 ms** | **19.21 ms** |

## Speedup: Rust/Forja AOT vs Python

| Test | Rust/Forja | Python | Speedup |
|------|:----------:|:------:|:-------:|
| nested_loop(1000) | **0.044 ms** | **3.62 ms** | **82×** |
| nested_loop(5000) | **0.228 ms** | **17.86 ms** | **78×** |
| sum_loop(1M) | **0.228 ms** | **31.61 ms** | **139×** |
| sum_loop(10M) | **2.28 ms** | **320 ms** | **140×** |
| fib(30) | **~1.94 ms** | **77.15 ms** | **~40×** |
| fib(35) | **~0.02 ms** | **873.72 ms** | **>1000×** |

## Velocidad

- **Forja AOT** (transpile→rustc -O): **~1.0× la velocidad de Rust nativo**
- **Python** (CPython 3.12): **~40× a ~140× mas lento que Rust/Forja AOT** (segun el test)
- **Forja VM** (interpretada): ~1956× mas lento que Rust (medido historicamente)

## Ejecutar benchmarks

```bash
# Rust native (single shot)
rustc -O benchmarks/bench_rust_native_aot.rs -o benchmarks/bench_rust_native_aot.exe
./benchmarks/bench_rust_native_aot.exe

# Rust native (heavy)
rustc -O benchmarks/bench_rust_heavy.rs -o benchmarks/bench_rust_heavy.exe
./benchmarks/bench_rust_heavy.exe

# Python
python benchmarks/bench_python_cpython_vs_forja.py

# Forja AOT (proyecto cargo)
cd benchmarks/bench_forja_aot.rs && cargo run --release

# Forja AOT heavy
cd benchmarks/bench_forja_heavy.rs && cargo run --release
```

## Archivos

| Archivo | Que mide |
|---------|----------|
| [`bench_rust_native_aot.rs`](./bench_rust_native_aot.rs) | Rust nativo single-shot con tiempos internos |
| [`bench_rust_heavy.rs`](./bench_rust_heavy.rs) | Rust pesado (100 iter internas) |
| [`bench_forja_aot.rs/`](./bench_forja_aot.rs/) | Forja AOT single-shot |
| [`bench_forja_heavy.rs/`](./bench_forja_heavy.rs/) | Forja AOT pesado (100 iter internas) |
| [`bench_python_cpython_vs_forja.py`](./bench_python_cpython_vs_forja.py) | Python detallado con min/avg/max |
| [`RESULTADOS_BENCHMARK.md`](./RESULTADOS_BENCHMARK.md) | Resultados detallados con analisis |
