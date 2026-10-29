#!/usr/bin/env python3
"""Check that docs/stdlib.json documents exactly the public functions
defined in the .simf files under simf/lib.

Exits silently with status 0 on an exact match; otherwise prints the
mismatches to stderr and exits with status 1.
"""

import json
import re
import sys
from pathlib import Path

PUB_FN = re.compile(r"^\s*pub\s+fn\s+(\w+)", re.MULTILINE)

functions = {
    name
    for path in Path("simf/lib").rglob("*.simf")
    for name in PUB_FN.findall(path.read_text(encoding="utf-8"))
}

docs = json.loads(Path("docs/stdlib.json").read_text(encoding="utf-8"))
docs_functions = {fun["simplicityhl_name"] for fun in docs}

if docs_functions != functions:
    for label, names in [("Unimplemented", docs_functions - functions),
                         ("Undocumented", functions - docs_functions)]:
        if names:
            print(f"{label} functions:", ", ".join(sorted(names)), file=sys.stderr)
    sys.exit(1)
