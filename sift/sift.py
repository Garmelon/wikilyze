import sys
from pathlib import Path

import mwxml  # https://pythonhosted.org/mwxml/


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def process_page(page):
    # https://pythonhosted.org/mwxml/iteration.html#mwxml.Page
    eprint(f"{page.id:8} - {page.title}")
    if len(list(page)) != 1:
        eprint(f"{page.id:8} - {page.title} - {len(list(page))}")


def main():
    dump = mwxml.Dump.from_file(sys.stdin)
    for page in dump.pages:
        process_page(page)
