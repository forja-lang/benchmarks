# Forja Benchmarks

Benchmarks de rendimiento para el lenguaje **Forja** y sus múltiples backends.

## Backends benchmarkeados

| Backend | Descripción |
|---------|-------------|
| ForjaFast VM | NaN tagging, stack caching, superinstrucciones |
| ForjaVM Original | Stack-based con tagged enums |
| JIT Nativo | Generación de código máquina x86-64 |
| Ensamblador Nativo | gcc -O2 |
| LLVM IR | llc -O2 |
| Transpile a Rust | rustc -O |

## Resultados

Ver [`RESULTADOS_BENCHMARK.md`](./RESULTADOS_BENCHMARK.md) para la comparativa completa vs Rust y Raven.

Resumen rápido:

| Implementación | Tiempo relativo |
|----------------|-----------------|
| Forja AOT (transpile → rustc -O) | **0.85×** (más rápido que Rust nativo) |
| Rust Native | 1.00× (referencia) |
| Forja ASM (gcc -O2) | 116× |
| Forja JIT | ~62× |
| ForjaFast VM | ~4.8× |
| ForjaVM Original | 1× (línea base) |

## Ejecutar benchmarks

```bash
# Todos los benchmarks
cargo bench

# Benchmark específico
cargo bench --bench bench-fa

# Benchmarks pesados (requiere --release)
.\run_benchmarks_heavy.ps1
```

## Archivos importantes

| Archivo | Propósito |
|---------|-----------|
| `bench_fa.rs` | Benchmark principal: Fibonacci, sum_loop, nested_loop |
| `bench_jit.rs` | Benchmark del JIT nativo |
| `bench_vms.rs` | Comparativa entre VMs |
| `bench_vs_python.rs` | Comparativa con Python (CPython) |
| `RESULTADOS_BENCHMARK.md` | Resultados históricos detallados |

## Repositorios relacionados

- [forja-lang/forja](https://github.com/forja-lang/forja) — Núcleo del lenguaje
- [forja-lang/examples](https://github.com/forja-lang/examples) — Ejemplos del lenguaje
