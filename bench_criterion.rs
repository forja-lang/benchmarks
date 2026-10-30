//! # Benchmarks estadísticos con Criterion
//!
//! Migra la metodología de `bench_forjafast` / `bench_vms` (medias aritméticas
//! con `Instant::now()`) a **Criterion**, el framework estadístico de Rust:
//!
//! - Mediana, percentiles e **intervalos de confianza** (bootstrap)
//! - Detección de **outliers** y de **regresiones** vs. una baseline
//! - Ancho de banda/throughput y tamaño de muestra adaptativo
//! - Informes HTML con gráficos en `target/criterion/`
//!
//! ## Grupos
//!
//! | Grupo | Mide |
//! |-------|------|
//! | `vm-hot`  | VM caliente: misma instancia, `reset()` + `ejecutar()` por iteración |
//! | `vm-cold` | VM fría: creación + carga + ejecución por iteración |
//! | `compilar`| Pipeline de compilación (lexer → parser → bytecode) |
//!
//! ## Ejecutar
//!
//! ```bash
//! # Suite completa
//! cargo bench --bench bench-criterion --features bench
//!
//! # Solo un grupo
//! cargo bench --bench bench-criterion --features bench -- vm-hot
//!
//! # Guardar baseline y comparar regresiones después de un cambio
//! cargo bench --bench bench-criterion --features bench -- --save-baseline base
//! cargo bench --bench bench-criterion --features bench -- --baseline base
//!
//! # Informe HTML (generado en target/criterion/report/)
//! cargo bench --bench bench-criterion --features bench -- --plotting-backend plotters
//! ```

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use forja::bytecode::{GeneradorBytecode, Opcode};
use forja::lexer::Lexer;
use forja::parser::Parser;
use forja::vm_fast::ForjaFast;
// (F1-T2) `forja::vm_jit::{compilar_bytecode, ForjaDT}` fue archivado en
// `src/legacy/`. La ruta JIT viva es `jit_tiered` (ver `tiered_hotness`).

const MAX_INSTRUCCIONES: usize = 1_000_000_000;

/// Carga de trabajo: (nombre, fuente Forja sin `escribir` — eval pura,
/// elementos de throughput estimados para el reporte).
const CARGA: [(&str, &str, u64); 4] = [
    ("fib30", FIB_FA, 30),
    ("suma10k", SUMA_FA, 10_000),
    ("fib15rec", FIB_REC_FA, 15),
    ("condicional", COND_FA, 1),
];

/// Variantes con `escribir` para verificar resultados una sola vez al inicio.
const VERIFICACION: [(&str, &str, &str); 4] = [
    ("fib30", FIB_FA_PRINT, "832040"),
    ("suma10k", SUMA_FA_PRINT, "49995000"),
    ("fib15rec", FIB_REC_FA_PRINT, "610"),
    ("condicional", COND_FA_PRINT, "verdadero"),
];

// ── Compilación ────────────────────────────────────────────────────────────────

fn compilar(source: &str) -> Vec<Opcode> {
    let mut gen = GeneradorBytecode::new();
    let tokens = Lexer::new(source).tokenize().unwrap();
    let prog = Parser::new(tokens).parse().unwrap();
    gen.generar(&prog).unwrap()
}

fn compilar_optimizado(source: &str) -> Vec<Opcode> {
    forja::bytecode::optimizar_indices(&compilar(source))
}

/// (F1-T2) Ejercita la política de compilación de `jit_tiered` (hotness + OSR)
/// sobre el bytecode dado. Reemplaza al antiguo `ForjaDT` archivado.
fn tiered_hotness(bc: &[Opcode]) {
    use forja::jit_tiered::{ShouldCompile, TieredJit};
    let mut tiered = TieredJit::new();
    for (ip, op) in bc.iter().enumerate() {
        match op {
            Opcode::DefinirFuncion(nombre, _) => tiered.register_function(nombre.as_ref()),
            Opcode::Llamar(nombre, _) => {
                match tiered.on_function_call(nombre.as_ref()) {
                    ShouldCompile::No | ShouldCompile::Yes(_) => (),
                };
            }
            _ => {}
        }
        let _ = tiered.on_loop_iteration("main", ip);
    }
}

// ── Verificación ───────────────────────────────────────────────────────────────

/// Ejecuta una vez cada workload con salida y chequea que el resultado sea el
/// esperado. Corre fuera de la medición para no contaminar las muestras.
fn verificar_resultados() {
    for (nombre, src, esperado) in VERIFICACION {
        let bc = compilar_optimizado(src);
        let mut vm = ForjaFast::new();
        vm.set_max_inst(MAX_INSTRUCCIONES);
        vm.cargar_bytecode(bc);
        vm.ejecutar().expect("el programa debe ejecutar");
        let out = vm.obtener_output();
        assert!(
            !out.is_empty() && out[0] == esperado,
            "[{}] resultado incorrecto: esperado {:?}, obtenido {:?}",
            nombre,
            esperado,
            out
        );
    }
}

// ── Benchmarks ─────────────────────────────────────────────────────────────────

/// VM caliente: la instancia se crea una vez fuera del loop de medición y se
/// reusa con `reset()`. Mide la velocidad de ejecución del bytecode puro.
fn bench_vm_hot(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm-hot");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(3));

    for (nombre, src, elems) in CARGA {
        let bc = compilar_optimizado(src);
        let bc_raw = compilar(src);
        group.throughput(Throughput::Elements(elems));

        // F1-B1: ForjaVM eliminada; la columna «original» mide ahora ForjaFast.
        group.bench_with_input(BenchmarkId::new("original", nombre), &bc, |b, bc| {
            let mut vm = ForjaFast::new();
            vm.set_max_inst(MAX_INSTRUCCIONES);
            vm.cargar_bytecode(bc.to_vec());
            b.iter(|| {
                vm.reset();
                vm.ejecutar().unwrap();
            });
        });

        // (F1-T2) La columna «jit» ejercita la política de `jit_tiered` (hotness/OSR)
        // y ejecuta con la VM canónica. `vm_jit::ForjaDT` fue archivado.
        group.bench_with_input(BenchmarkId::new("jit", nombre), &bc_raw, |b, bc_raw| {
            let mut vm = ForjaFast::new();
            vm.set_max_inst(MAX_INSTRUCCIONES);
            vm.cargar_bytecode(bc_raw.to_vec());
            b.iter(|| {
                tiered_hotness(bc_raw);
                vm.reset();
                vm.ejecutar().unwrap();
            });
        });

        group.bench_with_input(BenchmarkId::new("forjafast", nombre), &bc, |b, bc| {
            let mut vm = ForjaFast::new();
            vm.set_max_inst(MAX_INSTRUCCIONES);
            vm.cargar_bytecode(bc.to_vec());
            b.iter(|| {
                vm.reset();
                vm.ejecutar().unwrap();
            });
        });
    }
    group.finish();
}

/// VM fría: cada iteración crea la VM, carga el bytecode y ejecuta. Mide el
/// costo de instanciación + ejecución (cercano al arranque de un programa).
fn bench_vm_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("vm-cold");
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(2));

    for (nombre, src, elems) in CARGA {
        let bc = compilar_optimizado(src);
        let bc_raw = compilar(src);
        group.throughput(Throughput::Elements(elems));

        group.bench_with_input(BenchmarkId::new("original", nombre), &bc, |b, bc| {
            b.iter(|| {
                let mut vm = ForjaFast::new();
                vm.set_max_inst(MAX_INSTRUCCIONES);
                vm.cargar_bytecode(bc.to_vec());
                vm.ejecutar().unwrap();
            });
        });

        // (F1-T2) Columna «jit» sobre la ruta viva `jit_tiered` + VM canónica.
        group.bench_with_input(BenchmarkId::new("jit", nombre), &bc_raw, |b, bc_raw| {
            b.iter(|| {
                tiered_hotness(bc_raw);
                let mut vm = ForjaFast::new();
                vm.set_max_inst(MAX_INSTRUCCIONES);
                vm.cargar_bytecode(bc_raw.to_vec());
                vm.ejecutar().unwrap();
            });
        });

        group.bench_with_input(BenchmarkId::new("forjafast", nombre), &bc, |b, bc| {
            b.iter(|| {
                let mut vm = ForjaFast::new();
                vm.set_max_inst(MAX_INSTRUCCIONES);
                vm.cargar_bytecode(bc.to_vec());
                vm.ejecutar().unwrap();
            });
        });
    }
    group.finish();
}

/// Pipeline de compilación: desde fuente hasta bytecode (sin ejecución).
fn bench_compilar(c: &mut Criterion) {
    let mut group = c.benchmark_group("compilar");
    group.sample_size(30);
    group.measurement_time(std::time::Duration::from_secs(2));

    for (nombre, src, _) in CARGA {
        group.bench_with_input(BenchmarkId::new("bytecode", nombre), &src, |b, src| {
            b.iter(|| criterion::black_box(compilar(src)));
        });
        group.bench_with_input(
            BenchmarkId::new("pipeline-completa", nombre),
            &src,
            |b, src| {
                b.iter(|| forja::compilar_pipeline_completa(src).unwrap().0);
            },
        );
    }
    group.finish();
}

fn bencher(c: &mut Criterion) {
    verificar_resultados();
    bench_vm_hot(c);
    bench_vm_cold(c);
    bench_compilar(c);
}

criterion_group!(benches, bencher);
criterion_main!(benches);

// ── Fuentes Forja (sin `escribir`, eval pura) ──────────────────────────────────

const FIB_FA: &str = r#"
funcion fib(n) {
    si (n <= 1) { retornar n }
    variable a = 0
    variable b = 1
    variable i = 2
    mientras (i <= n) {
        variable t = a + b
        a = b
        b = t
        i = i + 1
    }
    retornar b
}
variable r = fib(30)
"#;

const SUMA_FA: &str = r#"
variable s = 0
variable i = 0
mientras (i < 10000) {
    s = s + i
    i = i + 1
}
"#;

const FIB_REC_FA: &str = r#"
funcion fib(n) {
    si (n <= 1) { retornar n }
    retornar fib(n-1) + fib(n-2)
}
variable r = fib(15)
"#;

const COND_FA: &str = r#"
si (5 > 3) {
    variable x = 1
} sino {
    variable x = 0
}
"#;

// ── Variantes con `escribir` para verificación ─────────────────────────────────

const FIB_FA_PRINT: &str = r#"
funcion fib(n) {
    si (n <= 1) { retornar n }
    variable a = 0
    variable b = 1
    variable i = 2
    mientras (i <= n) {
        variable t = a + b
        a = b
        b = t
        i = i + 1
    }
    retornar b
}
escribir(fib(30))
"#;

const SUMA_FA_PRINT: &str = r#"
variable s = 0
variable i = 0
mientras (i < 10000) {
    s = s + i
    i = i + 1
}
escribir(s)
"#;

const FIB_REC_FA_PRINT: &str = r#"
funcion fib(n) {
    si (n <= 1) { retornar n }
    retornar fib(n-1) + fib(n-2)
}
escribir(fib(15))
"#;

const COND_FA_PRINT: &str = r#"
si (5 > 3) { escribir("verdadero") } sino { escribir("falso") }
"#;
