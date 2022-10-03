import json
import sys
from pathlib import Path

import mwxml  # https://pythonhosted.org/mwxml/
import wikitextparser as wtp  # https://github.com/5j9/wikitextparser#readme


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def process_page(page):
    # https://pythonhosted.org/mwxml/iteration.html#mwxml.Page
    print(f"{page.namespace:4}: {page.id:8} - {page.title}")

    [revision] = list(page)  # Every page has exactly one revision
    text = revision.text or ""

    links = []
    for link in wtp.parse(text).wikilinks:
        start, end = link.span
        links.append((link.title, start, end))

    info = {
        "ns": page.namespace,
        "id": page.id,
        "title": page.title,
        "length": len(text),
        "links": links,
    }

    if page.redirect:
        assert len(links) == 1
        assert links[0] == page.redirect
        info["redirect"] = page.redirect

    print(json.dumps(info, check_circular=False, separators=(",", ":")))


def main():
    dump = mwxml.Dump.from_file(sys.stdin)
    for page in dump.pages:
        process_page(page)
