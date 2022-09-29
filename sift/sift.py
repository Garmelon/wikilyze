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

    info = {"id": page.id, "title": page.title}

    if page.redirect:
        info["redirect"] = page.redirect
    else:
        [revision] = list(page)  # Every page has exactly one revision

        length = len(revision.text)
        info["length"] = length

        # Parsing may fail for articles with length 0
        if length > 0:
            links = []
            for link in wtp.parse(revision.text).wikilinks:
                start, end = link.span
                links.append((link.title, start, end))
        info["links"] = links

    print(json.dumps(info, check_circular=False, separators=(",", ":")))


def main():
    dump = mwxml.Dump.from_file(sys.stdin)
    for page in dump.pages:
        process_page(page)
