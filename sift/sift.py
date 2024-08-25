import json
import re
import sys

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


def find_spans(page):
    # Within any of these spans, parentheses are ignored.
    spans = []
    spans.extend(i.span for i in page.comments)
    spans.extend(i.span for i in page.external_links)
    spans.extend(i.span for i in page.get_tags())  # Usually <ref>
    spans.extend(i.span for i in page.tables)
    spans.extend(i.span for i in page.templates)
    spans.extend(i.span for i in page.wikilinks)

    span_delims = []
    span_delims.extend((s, True) for s, _ in spans)
    span_delims.extend((e, False) for _, e in spans)
    span_delims.sort()
    return span_delims


def find_parens(page, span_delims):
    span_delims = list(reversed(sorted(span_delims)))

    open_spans = 0
    paren_delims = []

    for m in re.finditer(r"\(|\)", page.string):
        start, end = m.span()

        open_spans += advance_delims(span_delims, start)
        if open_spans != 0:
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
        else:
            eprint(f"(removed weird closing paren at {i})")

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
        else:
            eprint(f"(removed weird opening paren at {i})")
    paren_delims_3.reverse()

    return paren_delims_3


def format_link(link, in_parens, in_structure):
    title = link.title.strip()
    start, end = link.span
    flags = in_structure << 1 | in_parens
    return (title, start, end - start, flags)


def find_links(page, paren_delims):
    paren_delims = list(reversed(sorted(paren_delims)))

    open_parens = 0
    links = []

    for link in page.wikilinks:
        start, end = link.span
        open_parens += advance_delims(paren_delims, start)
        in_parens = open_parens != 0
        in_structure = link.parent() is not None
        links.append(format_link(link, in_parens, in_structure))

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
    span_delims = find_spans(parsed)
    paren_delims = find_parens(parsed, span_delims)
    paren_delims_fixed = fix_parens(paren_delims)
    links = find_links(parsed, paren_delims_fixed)

    info = {
        "id": page.id,
        "title": page.title,
        "length": len(text),
        "links": links,
    }

    if page.redirect:
        info["redirect"] = page.redirect

    print(json.dumps(info, check_circular=False, separators=(",", ":")))


def main():
    dump = mwxml.Dump.from_file(sys.stdin)
    for page in dump.pages:
        process_xmldump_page(page)
