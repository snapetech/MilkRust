#!/usr/bin/env python3
"""Extract milkrust-expr crate from milkrust-core/src/lib.rs"""

with open("crates/milkrust-core/src/lib.rs", "r") as f:
    lines = f.readlines()

# MilkRustValue definition: lines 1301-1326 (1-indexed)
# Tokenizer + parser: lines 4684-5448
# Helper functions: lines 5450-5713
# (Tests start at 5714)

header = """// =============================================================================
// milkrust-expr: Expression tokenizer, recursive-descent parser, and equation
// evaluation engine for MilkDrop-compatible preset scripts.
//
// This crate implements the full expression grammar including:
//   - Tokenization (identifiers, numbers, operators, parentheses)
//   - Recursive-descent parser with precedence climbing
//   - 50+ scoped math functions (sin, cos, atan2, pow, etc.)
//   - Loop/while/exec2/exec3 control flow
//   - megabuf/gmegabuf indexed memory
//   - Deterministic pseudo-random via seeded counter
// =============================================================================

use std::collections::BTreeMap;

"""

# MilkRustValue (lines 1301-1326)
value_def = lines[1300:1326]

# Tokenizer through end of parser (lines 4684-5713)
expr_code = lines[4683:5713]

# Merge: header + value_def + expr_code
final = header + "".join(value_def) + "".join(expr_code)

with open("crates/milkrust-expr/src/lib.rs", "w") as f:
    f.write(final)

print("Extracted expr crate: MilkRustValue + full expression engine")
print(f"  Value def: {len(value_def)} lines")
print(f"  Expr code: {len(expr_code)} lines")
print(f"  Total: {len(value_def) + len(expr_code)} lines")
