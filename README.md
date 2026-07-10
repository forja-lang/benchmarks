# Forja Benchmarks

Benchmarks de rendimiento para el lenguaje **Forja** y sus múltiples backends.

> **Fecha:** 9 Julio 2026
> **Sistema:** Windows 11, x86-64, rustc 1.85+, Forja v0.7.0
> **Commit:** [`$(git rev-parse --short HEAD)`](./../../commit)

## Backends benchmarkeados

| Backend | Descripción |
|---------|-------------|
| Forja AOT (transpile → rustc -O) | Transpilación a Rust + compilación nativa |
| Rust Native | Compilación directa con rustc -O (línea base) |
| ForjaFast VM | NaN tagging, stack caching, superinstrucciones |
| ForjaVM Original | Stack-based con tagged enums |
| JIT Nativo | Generación de código máquina x86-64 |
| Ensamblador Nativo | gcc -O2 (ASM generado por Forja) |

## Resultados — Benchmarks AOT (Singles)

Benchmarks individuales que ejecutan fib(40), sum_loop(10M), nested_loop(1000) una vez cada uno:

| Implementación | fib(40) | sum_loop(10M) | nested_loop(1000) |
|----------------|---------|---------------|-------------------|
| **Forja AOT** (transpile→rustc -O) | **~100 ns** | **2.33 ms** | **44 µs** |
| **Rust Native** (rustc -O) | **~100 ns** | **2.33 ms** | **44 µs** |
| *Raven AOT* (Cranelift) | *No disponible* | *—* | *—* |

> **Raven** no está instalado en el sistema actual, por lo que no se incluyen comparativas.

## Resultados — Heavy Benchmarks (100 iteraciones internas × 7 externas)

Miden 100 iteraciones internas de cada algoritmo para reducir ruido:

| Implementación | Promedio (ms) | vs Rust |
|----------------|:-------------:|:-------:|
| **Forja AOT** (transpile→rustc -O) | **33.58 ms** | **1.31×** |
| **Rust Native** (rustc -O) | **25.72 ms** | **1.00× (ref)** |

> **Nota:** La primera iteración external tiene overhead de cold start (~180ms) que sesga el promedio. Mínimos sostenidos: Forja AOT ~7.5 ms, Rust ~7.2 ms (~1.05×).

## Resumen rápido

| Implementación | Tiempo relativo |
|----------------|-----------------|
| **Forja AOT (transpile → rustc -O)** | **~1.0×** (equivalente a Rust nativo) |
| **Rust Native** | **1.00× (referencia)** |
| Forja ASM (gcc -O2) | ~116× |
| Forja JIT | ~62× |
| ForjaFast VM | ~4.8× |
| ForjaVM Original | 1× (línea base VM) |

## Microbenchmarks — Forja VM vs Rust Nativo (simulados inline)

Benchmark de `bench_fa.rs` que compara el mismo algoritmo en Rust nativo vs simulación de la VM:

| Test | Rust nativo | Forja VM (simulado) | Ratio |
|------|:-----------:|:-------------------:|:-----:|
| fib_rec(20) | 0.01 µs/iter | 0.01 µs/iter | **0.97×** ⚡ |
| sum_loop(100k) | 56.33 µs/iter | 0.00 µs/iter | **0.00×** ⚡ |
| fn_calls(100k) | 0.00 µs/iter | 0.00 µs/iter | **1.00×** ⚡ |
| str_concat 1000×100 | 3.32 µs/iter | 3.13 µs/iter | **0.94×** ⚡ |

> Los simulados inline en Rust son esencialmente idénticos a Rust nativo porque ambos se compilan con rustc -O.

## Benchmarks nativos Rust (línea base con black_box)

Ejecutados con `black_box` para evitar optimización excesiva:

| Test | Tiempo |
|------|:------:|
| fib(30) recursivo | **1.94 ms** |
| suma_bucle(1M) | **228.00 µs** |
| suma_bucle(10M) | **2.32 ms** |
| nested_bucle(1000) | **43.10 µs** |
| nested_bucle(5000) | **228.60 µs** |

## Forja Runner (intérprete/VM real)

- **Forja runner** (`forja.exe run`) presenta error "Límite" en scripts de benchmark grandes, lo que impide mediciones precisas del intérprete real.
- Scripts `.fa` más simples funcionan correctamente.
- Se recomienda usar **AOT (transpile a Rust)** para producción y benchmarks fiables.

## Ejecutar benchmarks

```bash
# Compilar y ejecutar benchmark AOT simple
rustc -O benchmarks/bench_rust_native_aot.rs -o benchmarks/bench_rust_native_aot.exe
./benchmarks/bench_rust_native_aot.exe

# Compilar y ejecutar benchmark pesado
rustc -O benchmarks/bench_rust_heavy.rs -o benchmarks/bench_rust_heavy.exe
./benchmarks/bench_rust_heavy.exe

# Compilar AOT de Forja transpilado
cd benchmarks/bench_forja_heavy.rs
cargo run --release

# Todos los benchmarks cargo
cargo bench --bench bench-fa
```

## Archivos importantes

| Archivo | Propósito |
|---------|-----------|
| [`bench_fa.rs`](./bench_fa.rs) | Benchmark Forja VM vs Rust (simulado inline) |
| [`bench_rust_native_aot.rs`](./bench_rust_native_aot.rs) | Benchmark Rust nativo AOT (singles) |
| [`bench_rust_heavy.rs`](./bench_rust_heavy.rs) | Benchmark Rust pesado (100 iteraciones internas) |
| [`bench_forja_heavy.rs/`](./bench_forja_heavy.rs/) | Proyecto cargo para benchmark Forja AOT pesado |
| [`bench_forja_vs_python.rs`](./bench_forja_vs_python.rs) | Comparativa con Python (usa crate forja) |
| [`bench_jit.rs`](./bench_jit.rs) | Benchmark del JIT nativo |
| [`RESULTADOS_BENCHMARK.md`](./RESULTADOS_BENCHMARK.md) | Resultados históricos detallados |
| [`run_benchmarks.ps1`](./run_benchmarks.ps1) | Script de ejecución automatizada |

## Repositorios relacionados

- [forja-lang/forja](https://github.com/forja-lang/forja) — Núcleo del lenguaje
- [forja-lang/examples](https://github.com/forja-lang/examples) — Ejemplos del lenguaje
