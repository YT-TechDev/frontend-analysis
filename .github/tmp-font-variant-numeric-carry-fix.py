from pathlib import Path

path = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
lines = path.read_text().splitlines()
out = []
i = 0
normalized_sites = 0

while i < len(lines):
    line = lines[i]
    out.append(line)
    if line.strip() != "font_variant_ligatures_observations,":
        i += 1
        continue

    indent = line[: len(line) - len(line.lstrip())]
    i += 1
    while i < len(lines) and lines[i].strip() == "font_variant_numeric_observations,":
        i += 1
    out.append(f"{indent}font_variant_numeric_observations,")
    normalized_sites += 1

if normalized_sites == 0:
    raise SystemExit("no font-variant-ligatures tuple carry sites found")

print(f"normalized {normalized_sites} font-variant-numeric carry site(s)")
path.write_text("\n".join(out) + "\n")
