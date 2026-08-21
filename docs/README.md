# Documentation

This directory is used to generate documentation for the SimplicityHL
standard library, which eventually should end up at

BlockstreamResearch/simplicity-lang-org/docs/documentation/stdlib.md

and so become visible on the web at

https://docs.simplicity-lang.org/documentation/stdlib


## Files

`stdlib.json`

This is a JSON file inspired by the jets documentation (`elements.json`)
from the [SimplicityHL codegen](https://github.com/BlockstreamResearch/SimplicityHL/tree/master/codegen).

Please keep this up-to-date with the standard library content itself
by documenting all stdlib functions and their content here.

It is assumed that "sections" in this file are *contiguous*, so please
put documentation for all library functions related to the same topic
(normally present in the same stdlib `.simf` file) together within
the JSON file.


`stdlib.md.py`

This Python script can be used to translate from `stdlib.json` to
`stdlib.md`, including some provided header and footer text. The format
of `elements.json` is very close to `stdlib.json`, and the format of
`jets.md` is very close to `stdlib.md`.


## Workflow changes

Feel free to replace this with other tools and workflows, as long as
they document the entire standard library and generate useful Markdown
for the developer docs site!

Feel free to add deployment hooks that rebuild the Markdown file and
cause a PR or commit on the developer documentation repository when the
`stdlib.json` file changes.
