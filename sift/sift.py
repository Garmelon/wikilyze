import json
import sys
from pathlib import Path

import mwxml  # https://pythonhosted.org/mwxml/
import wikitextparser as wtp  # https://github.com/5j9/wikitextparser#readme


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def process_page(page):
    # https://pythonhosted.org/mwxml/iteration.html#mwxml.Page
    eprint(f"{page.id:8} - {page.title}")

    [revision] = list(page)  # Every page has exactly one revision
    parsed = wtp.parse(revision.text)

    links = []
    for link in parsed.wikilinks:
        start, end = link.span
        links.append({"to": link.title, "start": start, "end": end})

    info = {"id": page.id, "title": page.title, "links": links}
    print(json.dumps(info, check_circular=False, separators=(",", ":")))


def main():
    dump = mwxml.Dump.from_file(sys.stdin)
    for page in dump.pages:
        process_page(page)
