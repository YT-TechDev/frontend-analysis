from pathlib import Path

path = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
lines = path.read_text().splitlines()
out = []
inserted = 0
for index, line in enumerate(lines):
    out.append(line)
    if line.strip() != "font_variant_ligatures_observations,":
        continue
    next_line = lines[index + 1].strip() if index + 1 < len(lines) else ""
    if next_line == "font_variant_numeric_observations,":
        continue
    indent = line[: len(line) - len(line.lstrip())]
    out.append(f"{indent}font_variant_numeric_observations,")
    inserted += 1

if inserted == 0:
    print("font-variant-numeric carry already complete")
else:
    print(f"inserted {inserted} missing carry element(s)")

path.write_text("\n".join(out) + "\n")
