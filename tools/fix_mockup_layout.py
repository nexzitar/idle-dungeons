#!/usr/bin/env python3
"""Convert mockup_layout.rs: spawn(NodeBundle{...}) then ButtonBundle blocks, then text/images."""
import pathlib
import sys

import convert_ui_bundles as c


def rewrite_spawn_nodebundles(s: str) -> str:
    key = ".spawn(NodeBundle"
    out = []
    i = 0
    while True:
        j = s.find(key, i)
        if j == -1:
            out.append(s[i:])
            break
        out.append(s[i:j])
        br = s.find("{", j)
        try:
            end = c.brace_end(s, br)
        except ValueError:
            out.append(s[j:])
            break
        inner = s[br + 1 : end].strip()
        conv = c.convert_nb(inner)
        if conv is None:
            out.append(s[j : end + 1])
            i = end + 1
            continue
        out.append(".spawn((\n            ")
        out.append(conv)
        out.append("\n        ))")
        i = end + 1
        if i < len(s) and s[i] == ")":
            i += 1
    return "".join(out)


def main():
    path = pathlib.Path(sys.argv[1])
    text = path.read_text()
    text = rewrite_spawn_nodebundles(text)
    text = c.replace_bundles(text, "NodeBundle", c.convert_nb)
    text = c.replace_bundles(text, "ButtonBundle", c.convert_bb)
    text = c.convert_text_bundles(text)
    text = c.convert_image_bundles(text)
    path.write_text(text)
    print("patched", path)


if __name__ == "__main__":
    main()
