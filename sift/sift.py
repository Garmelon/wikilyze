import json
import re
import sys
from multiprocessing import Pool

import mwxml  # https://pythonhosted.org/mwxml/
import wikitextparser as wtp  # https://github.com/5j9/wikitextparser#readme

# A link can have two important properties:
# 1. It can be inside a bigger structure (e.g. an infobox template)
# 2. It can be inside parentheses
#
# The first link that is neither in parentheses nor part of any template is
# considered the first link of the article for the purposes of the
# Philosophy Game.
#
# The parentheses "(" and ")" are only recognized outside of certain
# components like templates.
#
# https://en.wikipedia.org/wiki/Wikipedia:Getting_to_Philosophy
# https://diff.wikimedia.org/2018/04/20/why-it-took-a-long-time-to-build-that-tiny-link-preview-on-wikipedia/
# https://www.mediawiki.org/wiki/Page_Previews/API_Specification


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def advance_delims(delims, to):
    delta = 0

    while delims:
        i, opening = delims[-1]
        if i <= to:
            delims.pop()
            delta += 1 if opening else -1
        else:
            break

    return delta


def find_structures(page):
    # These elements count as "structures". Within them, parentheses are ignored
    # and links count as "in a structure".
    structures = []
    structures.extend(i.span for i in page.comments)
    structures.extend(i.span for i in page.external_links)
    # In disambiguation pages, <onlyinclude> tags wrap all links.
    structures.extend(i.span for i in page.get_tags() if i.name != "onlyinclude")
    structures.extend(i.span for i in page.tables)
    structures.extend(i.span for i in page.templates)

    structure_delims = []
    structure_delims.extend((s, True) for s, _ in structures)
    structure_delims.extend((e, False) for _, e in structures)
    structure_delims.sort()
    return structure_delims


def find_parens(page, structure_delims):
    structure_delims = list(reversed(sorted(structure_delims)))

    open_structures = 0
    paren_delims = []

    for m in re.finditer(r"\(|\)", page.string):
        start, end = m.span()

        open_structures += advance_delims(structure_delims, start)
        if open_structures != 0:
            continue

        opening = m.group(0) == "("
        pos = start if opening else end
        paren_delims.append((pos, opening))

    return paren_delims


def fix_parens(paren_delims):
    # First, remove closing parens that close nonexistent opening parens.
    open_parens = 0
    paren_delims_2 = []
    for i, opening in paren_delims:
        if opening:
            open_parens += 1
            paren_delims_2.append((i, opening))
        elif open_parens > 0:
            open_parens -= 1
            paren_delims_2.append((i, opening))

    # Then, remove opening parens that would never be closed.
    open_parens = 0
    paren_delims_3 = []
    for i, opening in reversed(paren_delims_2):
        if not opening:
            open_parens += 1
            paren_delims_3.append((i, opening))
        elif open_parens > 0:
            open_parens -= 1
            paren_delims_3.append((i, opening))
    paren_delims_3.reverse()

    return paren_delims_3


def format_link(link, in_structure, in_parens):
    title = link.title.strip()
    start, end = link.span
    flags = in_structure << 1 | in_parens
    return (title, start, end - start, flags)


def find_links(page, structure_delims, paren_delims):
    structure_delims = list(reversed(sorted(structure_delims)))
    paren_delims = list(reversed(sorted(paren_delims)))

    open_structures = 0
    open_parens = 0
    links = []

    for link in page.wikilinks:
        start, end = link.span
        open_structures += advance_delims(structure_delims, start)
        open_parens += advance_delims(paren_delims, start)
        in_structure = open_structures > 0 or link.parent() is not None
        in_parens = open_parens > 0
        links.append(format_link(link, in_structure, in_parens))

    return links


def pair_parens(paren_delims):
    starts = []
    spans = []
    for i, opening in paren_delims:
        if opening:
            starts.append(i)
        else:
            spans.append((starts.pop(), i))
    return spans


def process_xmldump_page(page):
    # https://pythonhosted.org/mwxml/iteration.html#mwxml.Page
    if page.namespace != 0:
        return
    eprint(f"{page.id:8} - {page.title!r}")

    [revision] = list(page)  # Every page has exactly one revision
    text = revision.text or ""
    parsed = wtp.parse(text)
    structure_delims = find_structures(parsed)
    paren_delims = find_parens(parsed, structure_delims)
    paren_delims_fixed = fix_parens(paren_delims)
    links = find_links(parsed, structure_delims, paren_delims_fixed)

    info = {
        "id": page.id,
        "title": page.title,
        "length": len(text),
        "links": links,
    }

    if page.redirect:
        info["redirect"] = page.redirect

    print(json.dumps(info, check_circular=False, separators=(",", ":")))


# Page info as simple tuples
def simple_pages(input):
    dump = mwxml.Dump.from_file(sys.stdin)
    articles = 0
    for i, page in enumerate(dump.pages):
        if (i + 1) % 1000 == 0:
            # Yeah, the articles are usually off by one
            eprint(f"{i+1:8} pages, {articles:8} articles, at pid {page.id:8}")

        if page.namespace != 0:
            continue

        articles += 1
        [revision] = list(page)  # Every page has exactly one revision
        yield page.id, page.title, revision.text or "", page.redirect

    eprint(f"{articles} articles total")


def process_simple_page(info):
    pid, title, text, redirect = info
    # eprint(f"{pid:8} - {title!r}")

    parsed = wtp.parse(text)
    structure_delims = find_structures(parsed)
    paren_delims = find_parens(parsed, structure_delims)
    paren_delims_fixed = fix_parens(paren_delims)
    links = find_links(parsed, structure_delims, paren_delims_fixed)

    info = {
        "id": pid,
        "title": title,
        "length": len(text),
        "links": links,
    }

    if redirect:
        info["redirect"] = redirect

    return json.dumps(info, check_circular=False, separators=(",", ":"))


def main():
    with Pool() as p:
        for dump in p.imap(process_simple_page, simple_pages(sys.stdin)):
            print(dump)
