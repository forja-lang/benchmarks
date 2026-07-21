# 🏎️ Forja Benchmarks

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)]()
[![Forja](https://img.shields.io/badge/Forja-0.8.7-blue)]()
[![Python](https://img.shields.io/badge/Python-3.12.10-yellow?logo=python)]()
[![Windows](https://img.shields.io/badge/Windows-11-blue?logo=windows)]()

> **Comparativa de velocidad:** `Forja AOT` vs `Rust Native` vs `CPython 3.12` en Windows 11 (x86-64)

---

## ⚡ De un vistazo

```
Rust    ████████████████████████████████████████████████████  1.0×  (referencia)
Forja   ████████████████████████████████████████████████████  ~1.0×  (identico)
Python  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░                      40×–140× mas lento
```

> ✅ **Forja AOT = Rust nativo.** Forja transpila a Rust y compila con `rustc -O`. Mismo LLVM backend, mismo codigo de maquina.

---

## 📊 Resultados clave

| Prueba | 🦀 Rust / 🔧 Forja | 🐍 Python | ⚡ diferencia |
|--------|:------------------:|:---------:|:-------------:|
| `nested_loop(1000)` | **0.044 ms** | **3.62 ms** | **82×** |
| `nested_loop(5000)` | **0.228 ms** | **17.86 ms** | **78×** |
| `sum_loop(1M)` | **0.228 ms** | **31.61 ms** | **139×** |
| `sum_loop(10M)` | **2.28 ms** | **320 ms** | **140×** |
| `fib(30)` | **~1.94 ms** | **77.15 ms** | **~40×** |
| `fib(35)` | **~0.02 ms** | **873.72 ms** | **>1000×** |

```
Python vs Rust/Forja:

nested_loop(1000)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  82× mas lento
nested_loop(5000)  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  78× mas lento
sum_loop(1M)       ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  139× mas lento
sum_loop(10M)      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  140× mas lento
fib(30)            ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  40× mas lento
fib(35)            ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  1000× mas lento
```

---

## 🚀 Benchmarks AOT (single shot)

Benchmark ejecuta `fib(40) + sum_loop(10M) + nested_loop(1000)` en un solo binario.

| Implementacion | Tiempo total | vs Rust |
|----------------|:------------:|:-------:|
| **Forja AOT** (transpile → rustc -O) | **7.6 ms** | **0.72×** |
| **Rust Native** (rustc -O) | **10.6 ms** | **1.00× (ref)** |

> ℹ️ Rust nativo incluye ~3ms de I/O (`println!`). Los calculos internos son identicos (fib(40)=100ns, sum=2.28ms, nested=44µs).

---

## 🏋️ Heavy Benchmarks (100 iteraciones internas, 7 runs)

| Implementacion | Minimo | Promedio | vs Rust |
|----------------|:-----:|:--------:|:-------:|
| **Forja AOT** (transpile → rustc -O) | **6.56 ms** | **7.91 ms** | **1.00×** |
| **Rust Native** (rustc -O) | **6.56 ms** | **8.24 ms** | **1.00× (ref)** |

> ✅ **Forja AOT y Rust son funcionalmente identicos.** La minima diferencia en promedios es ruido de medicion (cold start, scheduling del OS).

---

## 🐍 Python detallado (CPython 3.12.10)

| Test | Minimo | Promedio | Maximo |
|------|:-----:|:--------:|:------:|
| `fib(30)` | **77.15 ms** | **80.31 ms** | **85.49 ms** |
| `fib(35)` | **873.72 ms** | **882.87 ms** | **899.04 ms** |
| `suma_bucle(1M)` | **31.61 ms** | **38.78 ms** | **52.58 ms** |
| `suma_bucle(10M)` | **320.27 ms** | **323.74 ms** | **330.07 ms** |
| `nested_bucle(1000)` | **3.62 ms** | **3.69 ms** | **3.81 ms** |
| `nested_bucle(5000)` | **17.86 ms** | **18.58 ms** | **19.21 ms** |

---

## 💎 Conclusion

| Aspecto | Resultado |
|---------|-----------|
| **Forja AOT vs Rust** | **~1.0×** — Identicos. Mismo LLVM backend. |
| **Python vs Rust/Forja** | **40×–140× mas lento** — Tipico de lenguajes interpretados. |
| **Forja VM (interpretada)** | ~1956× mas lento que Rust (medido historicamente). |
| **Recomendacion** | Usar `forja transpile` + `rustc -O` para maxima velocidad. |

> 📖 Para analisis detallado, metodologia, speedup por test y microbenchmarks con black_box, ver [`RESULTADOS_BENCHMARK.md`](./RESULTADOS_BENCHMARK.md).

---

## 🛠️ Ejecutar benchmarks

```bash
# Rust native (single shot)
rustc -O benchmarks/bench_rust_native_aot.rs -o benchmarks/bench_rust_native_aot.exe
./benchmarks/bench_rust_native_aot.exe

# Rust native (heavy)
rustc -O benchmarks/bench_rust_heavy.rs -o benchmarks/bench_rust_heavy.exe
./benchmarks/bench_rust_heavy.exe

# Python
python benchmarks/bench_python_cpython_vs_forja.py

# Forja AOT
cd benchmarks/bench_forja_aot.rs && cargo run --release

# Forja AOT heavy
cd benchmarks/bench_forja_heavy.rs && cargo run --release
```
