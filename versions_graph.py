#!/usr/bin/env python3
"""Acumula corridas de `scripts/bench-releases.py` y publica el grafo por versión.

Vive en el repo `benchmarks`: el resultado se publica en el dashboard
(este mismo directorio `dashboard/`), así que la página queda en
https://forja-lang.github.io/forja/benchmarks/

Toma el JSON de una corrida, lo suma a un histórico (una fila por versión,
bloque y unidad) y escribe `dashboard/index.html` con un SVG inline: el
rendimiento comparativo entre versiones, una línea por bloque.

Normalización: cada tiempo se divide por `metadata.calibracion_ms` de su propia
corrida (un bucle fijo en Python) y se expresa como % de la mejor versión medida
para ese mismo bloque. Los ms absolutos no son comparables entre corridas —el
runner cambia—, pero el cociente sí. La memoria (pico de RSS, solo en Linux con
GNU time) se muestra aparte, en MB.

Solo stdlib: no usa CDN ni JS.

Uso (desde la raíz del repo principal):
    python benchmarks/versions_graph.py --json benchmarks/dashboard/corrida.json
"""

from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path

AQUI = Path(__file__).resolve().parent
# Mismo estimador que el veredicto de bench-releases.py: percentil bajo.
CUANTIL = 0.10
COLORES = ["#4cc9f0", "#f72585", "#f9c74f", "#90be6d", "#b5179e", "#43aa8b"]


def cuantil(xs: list[float], q: float = CUANTIL) -> float:
    ordenados = sorted(xs)
    if len(ordenados) == 1:
        return ordenados[0]
    k = int(round(q * (len(ordenados) - 1)))
    return ordenados[max(0, min(len(ordenados) - 1, k))]


def clave_version(v: str) -> list:
    """Orden natural: v0.9.10 va después de v0.9.9."""
    return [int(p) if p.isdigit() else p for p in re.split(r"(\d+)", v)]


def puntos_de(corrida: dict) -> list[dict]:
    meta = corrida["metadata"]
    calibracion = meta.get("calibracion_ms") or 0.0
    if not calibracion:
        raise SystemExit("✗ la corrida no trae `calibracion_ms`: no se puede normalizar")
    puntos = []
    for bloque in corrida["bloques"]:
        nombre = f"{bloque['ruta']}:{bloque['workload']}"
        base = {"bloque": nombre, "ruta": bloque["ruta"],
                "fecha": meta["fecha"], "plataforma": meta["plataforma"]}
        for version, muestras in bloque["muestras"].items():
            # `startup` no tiene suelo y no cambia por versión, pero es la serie
            # más útil para ver si el arranque del CLI se degradó.
            ms = cuantil(muestras)
            puntos.append({**base, "version": version, "unidad": "ms",
                           "valor": round(ms, 4), "indice": round(ms / calibracion, 9)})
        # La memoria no se normaliza por calibración: ya está en MB.
        for version, mb in (bloque.get("memoria") or {}).items():
            if mb is None:
                continue
            puntos.append({**base, "version": version, "unidad": "MB",
                           "valor": round(mb, 2), "indice": round(mb, 6)})
    return puntos


def acumular(historial: Path, puntos: list[dict]) -> list[dict]:
    """Suma los puntos al histórico, pisando el mismo (versión, bloque, unidad, plataforma)."""
    previos: list[dict] = []
    if historial.exists():
        try:
            previos = json.loads(historial.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            print(f"⚠ {historial} corrupto; se recrea")
    nuevos = {(p["version"], p["bloque"], p["unidad"], p["plataforma"]) for p in puntos}
    todos = [p for p in previos
             if (p.get("version"), p.get("bloque"), p.get("unidad"),
                 p.get("plataforma")) not in nuevos]
    todos += puntos
    todos.sort(key=lambda p: (clave_version(p["version"]), p["bloque"], p["unidad"]))
    historial.parent.mkdir(parents=True, exist_ok=True)
    historial.write_text(json.dumps(todos, indent=1, ensure_ascii=False), encoding="utf-8")
    return todos


def svg(puntos: list[dict], versiones: list[str], ancho: int = 940,
        alto: int = 440) -> str:
    """Un punto por versión y bloque, en % de la mejor versión medida para ese bloque.

    Normalizar por bloque evita que la ruta más lenta aplaste a la más rápida en
    la misma escala, y el cociente ms/calibración ya quitó la velocidad del runner.
    """
    izq, der, arriba, abajo = 84, 250, 24, 56
    minimos = {b: min(p["indice"] for p in puntos if p["bloque"] == b)
               for b in {p["bloque"] for p in puntos}}
    relativo = lambda p: p["indice"] / minimos[p["bloque"]]
    valores = [relativo(p) for p in puntos]
    # Siempre se ve el 100 % y un margen mínimo: con dos versiones casi iguales
    # el ruido no debe parecer un acantilado.
    holgura = max(0.02, (max(valores) - min(valores)) * 0.15)
    vmin = min(valores + [1.0]) - holgura
    vmax = max(valores + [1.0]) + holgura
    etiqueta = (lambda v: f"{v * 100:.1f} %") if (vmax - vmin) < 0.1 \
        else (lambda v: f"{v * 100:.0f} %")
    ancho_util = ancho - izq - der
    alto_util = alto - arriba - abajo
    x_de = {v: izq + (ancho_util * i / max(1, len(versiones) - 1))
            for i, v in enumerate(versiones)}
    y_de = lambda v: arriba + alto_util * (1 - (v - vmin) / (vmax - vmin))

    partes = [f'<svg viewBox="0 0 {ancho} {alto}" role="img" '
              f'aria-label="Rendimiento por versión" class="grafo">']
    for i in range(5):
        v = vmin + (vmax - vmin) * i / 4
        y = y_de(v)
        partes.append(f'<line x1="{izq}" y1="{y:.1f}" x2="{izq + ancho_util}" '
                      f'y2="{y:.1f}" class="rejilla"/>')
        partes.append(f'<text x="{izq - 10}" y="{y + 4:.1f}" class="eje" '
                      f'text-anchor="end">{etiqueta(v)}</text>')
    for v in versiones:
        partes.append(f'<text x="{x_de[v]:.1f}" y="{alto - abajo + 22}" class="eje" '
                      f'text-anchor="middle">{v}</text>')

    bloques = sorted({p["bloque"] for p in puntos})
    for i, bloque in enumerate(bloques):
        color = COLORES[i % len(COLORES)]
        serie = sorted((p for p in puntos if p["bloque"] == bloque),
                       key=lambda p: clave_version(p["version"]))
        coords = " ".join(f"{x_de[p['version']]:.1f},{y_de(relativo(p)):.1f}" for p in serie)
        if len(serie) > 1:
            partes.append(f'<polyline points="{coords}" fill="none" stroke="{color}" '
                          f'stroke-width="2" opacity="0.9"/>')
        for p in serie:
            partes.append(
                f'<circle cx="{x_de[p["version"]]:.1f}" cy="{y_de(relativo(p)):.1f}" r="4" '
                f'fill="{color}"><title>{bloque} — {p["version"]}: {p["valor"]} ms '
                f'({relativo(p) * 100:.1f} % del mejor)</title></circle>')
        ly = arriba + 14 + i * 20
        partes.append(f'<rect x="{izq + ancho_util + 18}" y="{ly - 9}" width="12" '
                      f'height="12" fill="{color}" rx="2"/>')
        partes.append(f'<text x="{izq + ancho_util + 36}" y="{ly + 1}" class="eje">'
                      f'{bloque}</text>')

    partes.append(f'<line x1="{izq}" y1="{arriba + alto_util}" x2="{izq + ancho_util}" '
                  f'y2="{arriba + alto_util}" class="eje-linea"/>')
    partes.append(f'<text x="16" y="{arriba + alto_util / 2:.0f}" class="eje" '
                  f'transform="rotate(-90 16 {arriba + alto_util / 2:.0f})" '
                  f'text-anchor="middle">% del tiempo de la mejor versión medida</text>')
    partes.append("</svg>")
    return "\n".join(partes)


def tabla(corrida: dict) -> str:
    comps = corrida.get("comparaciones") or []
    if not comps:
        return "<p>Sin comparaciones en esta corrida.</p>"
    ref = corrida["metadata"]["referencia"]
    filas = "".join(
        f"<tr><td>{c['ruta']}</td><td>{c['workload']}</td>"
        f"<td>{c['estimador_a']:.1f}</td><td>{c['estimador_b']:.1f}</td>"
        f"<td>{c['delta_pct']:+.1f} %</td><td>{c['p']:.3f}</td>"
        f"<td class='{c['veredicto'].replace(' ', '-')}'>{c['veredicto']}</td></tr>"
        for c in comps)
    return (f"<table><caption>{ref} (referencia) vs el resto</caption>"
            "<tr><th>Ruta</th><th>Carga</th><th>p10 ref (ms)</th><th>p10 otra (ms)</th>"
            f"<th>Δ</th><th>p</th><th>Veredicto</th></tr>{filas}</table>")


def tabla_memoria(puntos: list[dict], versiones: list[str]) -> str:
    """Pico de RSS por bloque y versión (vacío en plataformas sin GNU time)."""
    memoria = [p for p in puntos if p["unidad"] == "MB"]
    if not memoria:
        return ("<p><em>Sin datos de memoria: esta plataforma no tiene GNU time "
                "(<code>/usr/bin/time -v</code>). Se mide en el CI de Linux.</em></p>")
    bloques = sorted({p["bloque"] for p in memoria})
    celdas = {}
    for p in memoria:
        celdas[(p["bloque"], p["version"])] = p["valor"]
    cabecera = "".join(f"<th>{v}</th>" for v in versiones)
    filas = ""
    for b in bloques:
        valores = {v: celdas.get((b, v)) for v in versiones}
        filas += (f"<tr><td>{b}</td>" + "".join(
            f"<td>{valores[v]:.1f} MB</td>" if valores[v] is not None else "<td>—</td>"
            for v in versiones) + "</tr>")
    return ("<table><caption>Pico de memoria (RSS, mediana de 3 corridas)</caption>"
            f"<tr><th>Bloque</th>{cabecera}</tr>{filas}</table>")


def html(puntos: list[dict], corrida: dict, plataforma: str) -> str:
    tiempos = [p for p in puntos if p["unidad"] == "ms"]
    versiones = sorted({p["version"] for p in puntos}, key=clave_version)
    meta = corrida["metadata"]
    grafo = svg(tiempos, sorted({p["version"] for p in tiempos}, key=clave_version)) \
        if tiempos else "<p>Sin tiempos medidos.</p>"
    generado = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    return f"""<!doctype html>
<html lang="es"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Forja — rendimiento por versión</title>
<style>
 body {{ background:#0e1116; color:#e6edf3; font:15px/1.5 system-ui,sans-serif;
        margin:0 auto; max-width:1020px; padding:32px 16px 64px }}
 h1 {{ font-size:24px; margin:0 0 4px }} h2 {{ font-size:18px; margin-top:36px }}
 p, li {{ color:#adbac7 }} a {{ color:#4cc9f0 }}
 .grafo {{ width:100%; height:auto; background:#11161d; border-radius:10px; padding:8px }}
 .rejilla {{ stroke:#25303b }} .eje {{ fill:#8b98a5; font-size:12px }}
 .eje-linea {{ stroke:#3d4a58 }}
 table {{ border-collapse:collapse; width:100%; margin-top:12px; font-size:14px }}
 th, td {{ padding:6px 8px; border-bottom:1px solid #25303b; text-align:left }}
 th {{ color:#8b98a5; font-weight:600 }} caption {{ text-align:left; color:#8b98a5;
      padding-bottom:6px }}
 .más-rápido {{ color:#4cc9f0 }} .más-lento {{ color:#f72585 }}
 .sin-diferencia {{ color:#8b98a5 }} .SIN-TRABAJO-REAL, .SALIDAS-DISTINTAS {{ color:#f9c74f }}
 code {{ background:#11161d; padding:1px 5px; border-radius:4px }}
</style></head><body>
<h1>Rendimiento de Forja por versión</h1>
<p>Plataforma <code>{plataforma}</code> · generado {generado} · última corrida
{meta['fecha']} ({', '.join(meta['tags'])}, {meta['pares']} parejas,
estimador percentil {meta['quantile'] * 100:.0f}).</p>
<p>Cada punto es el tiempo de una ruta dividido por la
<strong>calibración de la máquina</strong> de esa misma corrida
({meta['calibracion_ms']:.1f} ms de bucle fijo) y expresado como porcentaje de
la mejor versión medida para esa misma carga. Los ms absolutos no se pueden
comparar entre corridas porque el runner cambia; este cociente sí. Bajar es
mejorar; 100 % es la más rápida medida hasta ahora.</p>
{grafo}
<h2>Memoria</h2>
{tabla_memoria(puntos, versiones)}
<h2>Veredicto de la última corrida</h2>
<p>Un veredicto exige p &lt; 0.05, IC95 sin 0 y efecto ≥
{meta['min_efecto_pct']:.0f} %. Las rutas con menos de
{meta['min_trabajo_pct']:.0f} % de cálculo real sobre el suelo no opinan.</p>
{tabla(corrida)}
<h2>Cómo se genera</h2>
<p>Lo corre el job <code>bench-versiones</code> de
<code>.github/workflows/cd.yml</code> después de publicar cada release:
descarga los binarios de las últimas versiones desde GitHub, los mide
intercalados con orden aleatorizado y suma el resultado a
<code>data/versions.json</code>. El informe completo de cada corrida queda en
<code>informe.md</code>.</p>
</body></html>
"""


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--json", required=True, help="JSON de una corrida de bench-releases.py")
    p.add_argument("--historial", default=str(AQUI / "dashboard" / "data" / "versions.json"),
                   help="histórico acumulado (se crea)")
    p.add_argument("--out", default=str(AQUI / "dashboard"),
                   help="directorio donde escribir index.html")
    args = p.parse_args()

    corrida = json.loads(Path(args.json).read_text(encoding="utf-8"))
    nuevos = puntos_de(corrida)
    historial = acumular(Path(args.historial), nuevos)

    plataforma = corrida["metadata"]["plataforma"]
    puntos = [p for p in historial if p.get("plataforma") == plataforma]
    salida = Path(args.out)
    salida.mkdir(parents=True, exist_ok=True)
    (salida / "index.html").write_text(html(puntos, corrida, plataforma), encoding="utf-8")
    print(f"▸ {salida / 'index.html'}: {len(puntos)} puntos, "
          f"{len({p['version'] for p in puntos})} versiones ({plataforma})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
