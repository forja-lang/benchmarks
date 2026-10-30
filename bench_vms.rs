// Comparativa: VM canónica (vm_fast) vs ruta JIT (jit_tiered) vs ForjaFast
// cargo run --release --bin bench-vms
//
// (F1-T2) `vm_jit::ForjaDT` fue archivado en `src/legacy/`. El único JIT vivo es
// `jit_tiered`, que aún no expone un ejecutor propio: se ejercita su política de
// compilación (hotness/OSR) y se ejecuta con la VM canónica `vm_fast::ForjaFast`.

use std::time::Instant;

fn main() {
    println!("═══════════════════════════════════════════════════");
    println!("  🔥 Forja VM: Canónica vs JIT (tiered) vs ForjaFast");
    println!("  1000 iteraciones, SIN output (eval pura)");
    println!("═══════════════════════════════════════════════════");
    println!();

    let iters = 1000;

    // Código fibonacci(30) sin Print
    let source = r#"
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

    let bc = compilar(source);

    println!("───────────────────────────────────────────────");
    println!("  TEST 1: fibonacci(30) — SIN PRINT");
    println!("───────────────────────────────────────────────");

    let t1 = medir(iters, || {
        // F1-B1: ForjaVM eliminada; se usa la VM canónica vm_fast::ForjaFast.
        let mut v = forja::vm_fast::ForjaFast::new();
        v.cargar_bytecode(bc.clone());
        v.ejecutar().unwrap();
    });
    let t3 = medir(iters, || {
        // (F1-T2) Ruta JIT: se ejercita la política de `jit_tiered` sobre el
        // bytecode y se ejecuta con la VM canónica.
        ejercitar_tiered(&bc);
        let mut v = forja::vm_fast::ForjaFast::new();
        v.cargar_bytecode(bc.clone());
        v.ejecutar().unwrap();
    });

    print_row("VM Canónica", t1, t1);
    print_row("VM JIT (tiered)", t3, t1);

    // ===== TEST 2: Bucle =====
    let src2 = "variable s = 0\nvariable i = 0\nmientras (i < 50000) { s = s + i\ni = i + 1 }";
    let bc2 = compilar(src2);

    println!();
    println!("───────────────────────────────────────────────");
    println!("  TEST 2: bucle 50000 iters");
    println!("───────────────────────────────────────────────");

    let t1 = medir(iters, || {
        let mut v = forja::vm_fast::ForjaFast::new();
        v.cargar_bytecode(bc2.clone());
        v.ejecutar().unwrap();
    });
    let t3 = medir(iters, || {
        ejercitar_tiered(&bc2);
        let mut v = forja::vm_fast::ForjaFast::new();
        v.cargar_bytecode(bc2.clone());
        v.ejecutar().unwrap();
    });

    print_row("VM Canónica", t1, t1);
    print_row("VM JIT (tiered)", t3, t1);

    // ===== Python reference =====
    println!();
    println!("───────────────────────────────────────────────");
    println!("  📊 vs Python (estimado)");
    println!("───────────────────────────────────────────────");
    println!("  Python fib(30):   ~200 μs/iter");
    println!("  Python bucle 50k: ~3000 μs/iter");
    println!();
    println!(
        "  Forja JIT fib(30)  vs Python: {:.1}x más rápido",
        200.0 / t3
    );
    println!(
        "  Forja JIT bucle 50k vs Python: {:.1}x más rápido",
        3000.0 / t3
    );
}

/// (F1-T2) Ejercita la política de compilación de `jit_tiered` (hotness + OSR)
/// sobre el bytecode dado. Reemplaza al antiguo `ForjaDT` archivado.
fn ejercitar_tiered(bc: &[forja::bytecode::Opcode]) {
    use forja::bytecode::Opcode;
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

fn compilar(source: &str) -> Vec<forja::bytecode::Opcode> {
    let mut gen = forja::bytecode::GeneradorBytecode::new();
    let mut lexer = forja::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = forja::parser::Parser::new(tokens);
    let prog = parser.parse().unwrap();
    gen.generar(&prog).unwrap()
}

fn medir(iters: usize, mut f: impl FnMut()) -> f64 {
    let inicio = Instant::now();
    for _ in 0..iters {
        f();
    }
    inicio.elapsed().as_secs_f64() * 1_000_000.0 / iters as f64
}

fn print_row(nombre: &str, us: f64, baseline: f64) {
    let ratio = baseline / us;
    let stars = if ratio >= 2.0 {
        " ⭐"
    } else if ratio >= 1.5 {
        " ★"
    } else if ratio >= 1.1 {
        " ✓"
    } else {
        ""
    };
    println!(
        "  {:<30} {:>8.2} μs/iter  ({:.2}x){}",
        nombre, us, ratio, stars
    );
}
