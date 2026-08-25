#!/usr/bin/env python3

# This consumes stdlib.json and generates docs/documentation/stdlib.md.
#
# Adapted from jets.md.py, which generates the jets reference page
# from a JSON file with the same general structure.

import datetime
import json
import os
import sys

SCRIPT_NAME = os.path.basename(__file__)


def build_preamble(source_path):
    return """# SimplicityHL standard library reference
<!-- Generated from {} by {} on {} -->

The SimplicityHL standard library provides various functions useful in developing smart contracts.

Here is a complete list of the available library functions, their <a href="../../simplicityhl-reference/type/">type signatures</a>, and a description of what they do.

Some library functions can fail or panic. This allows a Simplicity program to refuse a proposed transaction by performing a mandatory assertion; these functions' return type is `()` below. The failure or panic effect produced by these functions, or the corresponding behavior of jets, is ultimately the *only* way to decline a transaction.

For more built-in SimplicityHL functions, see the [jets reference](../documentation/jets).
""".format(source_path, SCRIPT_NAME, datetime.datetime.now().date().isoformat())


def new_section(section_name, introduction=""):
    template = """
## {}

{}

???+ "Click to hide"
    | <div style="width:22em">Standard library function</div> | Description |
    | ----------------------------------- | ----------- |"""
    return template.format(section_name, introduction)


def escape_cell(text):
    """Escape characters that would break a Markdown table cell."""
    return text.replace("|", "\\|")


def format_function(name, input_type, output_type, description):
    signature = escape_cell("{}({}) -> {}".format(name, input_type, output_type))
    description = escape_cell(description)
    return "    | `{}` | {} |".format(signature, description)


def main():
    if len(sys.argv) != 2:
        sys.exit("usage: {} <stdlib.json>".format(SCRIPT_NAME))

    source_path = sys.argv[1]
    with open(source_path) as f:
        functions = json.load(f)

    output = [build_preamble(source_path)]

    # Assumes that all functions in a section are adjacent in the .json input!
    current_section = None
    for func in functions:
        description = func["description"].replace("\n", "<br>")
        section = func["section"]
        if section != current_section:
            output.append(new_section(section))
            current_section = section
        output.append(format_function(
            func["simplicityhl_name"], func["input_type"], func["output_type"], description
        ))

    print("\n".join(output))
    print()


if __name__ == "__main__":
    main()
