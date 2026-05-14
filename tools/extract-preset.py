#!/usr/bin/env python3
"""Extract milkrust-preset crate from milkrust-core/src/lib.rs"""

import os

with open("crates/milkrust-core/src/lib.rs", "r") as f:
    lines = f.readlines()

os.makedirs("crates/milkrust-preset/src", exist_ok=True)

# MilkRustValue is already in expr crate, so we re-export or reference it.
# Actually, preset crate depends on expr crate, so it'll import from there.
# We only extract the types/functions defined AFTER MilkRustValue.

# MilkRustEquations: 1328-1335
# MilkRustIndexedEntry: 1337-1342
# MilkRustPresetDocument: 1344-1377
# MilkRustPresetSet: 1379-1383
# is_numeric_milkrust_value: 1385-1387
# normalize_milkrust_value: 1389-1396
# append_milkrust_* helpers: 1398-1460
# assign_milkrust_indexed_equation: 1462-1480
# parse_milkrust_preset_text: 1482-1549
# parse_milkrust_preset_set: 1550-1569
# MilkRustFragment: 1571-1614
# parse_milkrust_fragment: 1616-1693
# serialize_milkrust_preset_set: 1695-1736
# parse_milkrust_sample_csv: 446-454

# parse_milkrust_sample_csv is a standalone public function at line 446
# Let's include it

# Compatibility section: MilkRustPresetCompatibility at 3587
# analyze_milkrust_preset_compatibility at 3693
# collect_milkrust_functions at 3641
# analyze_milkrust_shader_support at 4669
# MilkRustShaderProgram at 4076
# parse_milkrust_shader_program at 4346

# milk2 parsing: parse_milk2_section, parse_milk_sections, etc.
# Let's find them
milk2_functions = []
for i, line in enumerate(lines, 1):
    s = line.strip()
    if s.startswith('fn ') or s.startswith('pub fn '):
        if any(kw in s for kw in ['milk2', 'milk_', 'preset_set_to_milk']):
            milk2_functions.append((i, line.rstrip()))

print("Milk2-related functions:")
for ln, txt in milk2_functions:
    print(f"  {ln}: {txt}")

# parse_milkrust_preset at line 138
print(f"\nparse_milkrust_preset at line 138")
print(f"  {lines[137].rstrip()}")

# Find where parse_milkrust_preset ends
for i in range(138, 500):
    if i >= len(lines):
        break
    s = lines[i].strip()
    if s.startswith('fn ') and 'milkrust_preset' not in s.lower() and 'parse' in s.lower():
        print(f"  Next function at {i+1}: {s}")
        break
