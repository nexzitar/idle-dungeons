#!/usr/bin/env python3
"""Convert NodeBundle { style: Style {..}, ... } / ButtonBundle blocks in this repo to Bevy 0.18 components."""
import pathlib
import re
import sys


def brace_end(s: str, i: int) -> int:
    d = 0
    for k in range(i, len(s)):
        if s[k] == "{":
            d += 1
        elif s[k] == "}":
            d -= 1
            if d == 0:
                return k
    raise ValueError("unbalanced")


def split_commas(s: str):
    out = []
    buf = []
    d_brace = 0
    d_paren = 0
    for ch in s:
        if ch == "{":
            d_brace += 1
            buf.append(ch)
        elif ch == "}":
            d_brace -= 1
            buf.append(ch)
        elif ch == "(":
            d_paren += 1
            buf.append(ch)
        elif ch == ")":
            d_paren -= 1
            buf.append(ch)
        elif ch == "," and d_brace == 0 and d_paren == 0:
            t = "".join(buf).strip()
            if t and t != "..default()":
                out.append(t)
            buf = []
        else:
            buf.append(ch)
    tail = "".join(buf).strip()
    if tail and tail != "..default()":
        out.append(tail)
    return out


def parse_kv_fields(inner: str):
    chunks = split_commas(inner)
    kv = {}
    for ch in chunks:
        if ":" not in ch:
            continue
        k, v = ch.split(":", 1)
        kv[k.strip()] = v.strip()
    return kv


def convert_nb(inner: str) -> str | None:
    kv = parse_kv_fields(inner)
    if "style" not in kv:
        return None
    st = kv["style"]
    if not st.startswith("Style"):
        return None
    sb = st.find("{")
    if sb < 0:
        return None
    se = brace_end(st, sb)
    body = st[sb + 1 : se].strip()
    if "box_sizing" not in body:
        body = "box_sizing: BoxSizing::BorderBox,\n                " + body
    parts = [f"Node {{\n                {body}\n            }}"]
    if "background_color" in kv:
        parts.append(f"BackgroundColor({kv['background_color']})")
    if "border_color" in kv:
        bc = kv["border_color"]
        if bc.startswith("BorderColor(") and bc.endswith(")"):
            inner_bc = bc[len("BorderColor(") : -1]
            parts.append(f"BorderColor::from({inner_bc})")
        else:
            parts.append(f"BorderColor::from({bc})")
    if "focus_policy" in kv:
        fp = kv["focus_policy"]
        parts.append(fp)
    if "visibility" in kv:
        parts.append(kv["visibility"])
    return ",\n            ".join(parts)


def convert_bb(inner: str) -> str | None:
    kv = parse_kv_fields(inner)
    if "style" not in kv:
        return None
    st = kv["style"]
    if not st.startswith("Style"):
        return None
    sb = st.find("{")
    se = brace_end(st, sb)
    body = st[sb + 1 : se].strip()
    if "box_sizing" not in body:
        body = "box_sizing: BoxSizing::BorderBox,\n                " + body
    parts = [
        f"Node {{\n                {body}\n            }}",
        "Button",
    ]
    if "background_color" in kv:
        parts.append(f"BackgroundColor({kv['background_color']})")
    if "border_color" in kv:
        bc = kv["border_color"]
        if bc.startswith("BorderColor(") and bc.endswith(")"):
            inner_x = bc[len("BorderColor(") : -1]
            parts.append(f"BorderColor::from({inner_x})")
        else:
            parts.append(f"BorderColor::from({bc})")
    return ",\n            ".join(parts)


def replace_bundles(s: str, name: str, conv) -> str:
    token = name
    out = []
    i = 0
    while True:
        j = s.find(token, i)
        if j == -1:
            out.append(s[i:])
            break
        out.append(s[i:j])
        br = s.find("{", j)
        if br < 0:
            out.append(s[j:])
            break
        end = brace_end(s, br)
        inner = s[br + 1 : end].strip()
        rep = conv(inner)
        if rep is None:
            out.append(s[j : end + 1])
        else:
            out.append(rep)
        i = end + 1
    return "".join(out)


def convert_text_bundles(s: str) -> str:

    def repl(m):
        expr = m.group(1).strip()
        fs = m.group(2).strip()
        col = m.group(3).strip()
        return f"(\n                Text::new({expr}),\n                TextFont::from_font_size({fs}),\n                TextColor({col}),\n            )"

    pat = re.compile(
        r"TextBundle::from_section\(\s*(.*?)\s*,\s*TextStyle\s*\{\s*font_size:\s*([^,]+),\s*color:\s*([^,]+),[^}]*\}\s*,?\s*\)",
        re.DOTALL,
    )
    return pat.sub(repl, s)


def convert_image_bundles(s: str) -> str:
    pat2 = re.compile(
        r"ImageBundle\s*\{\s*style:\s*Style\s*\{([\s\S]*?)\}\s*,\s*image:\s*UiImage::new\(([^)]+)\)\s*\.\s*with_color\(([\s\S]*?)\)\s*,?\s*\.\.default\(\)\s*\}",
        re.DOTALL,
    )

    def repl2(m):
        body = m.group(1).strip()
        handle = m.group(2).strip()
        col = m.group(3).strip()
        if "box_sizing" not in body:
            body = "box_sizing: BoxSizing::BorderBox,\n                " + body
        return (
            "(\n                Node {\n                "
            + body
            + "\n            },\n                ImageNode {\n                image: "
            + handle
            + ",\n                color: "
            + col
            + ",\n                ..default()\n            }\n            )"
        )

    s = pat2.sub(repl2, s)

    pat = re.compile(
        r"ImageBundle\s*\{\s*style:\s*Style\s*\{([\s\S]*?)\}\s*,\s*image:\s*UiImage::new\(([\s\S]*?)\)\s*,?\s*\.\.default\(\)\s*\}",
        re.DOTALL,
    )

    def repl(m):
        body = m.group(1).strip()
        handle = m.group(2).strip()
        if "box_sizing" not in body:
            body = "box_sizing: BoxSizing::BorderBox,\n                " + body
        return (
            "(\n                Node {\n                "
            + body
            + "\n            },\n                ImageNode {\n                image: "
            + handle
            + ",\n                color: Color::WHITE,\n                ..default()\n            }\n            )"
        )

    return pat.sub(repl, s)


def main():
    for path in sys.argv[1:]:
        p = pathlib.Path(path)
        t = p.read_text()
        t2 = replace_bundles(t, "NodeBundle", convert_nb)
        t2 = replace_bundles(t2, "ButtonBundle", convert_bb)
        t2 = convert_text_bundles(t2)
        t2 = convert_image_bundles(t2)
        if t2 != t:
            p.write_text(t2)
            print("OK", p)


if __name__ == "__main__":
    main()
