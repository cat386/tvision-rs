#!/usr/bin/env python3
"""Сборка руководства пользователя (docs/guide/*.md) в один PDF.

    python3 tools/guide-pdf/build.py [выходной.pdf]

Нужны: Python 3 с пакетами markdown, pygments, pypdf, playwright
(и браузер Chromium для playwright). По умолчанию пишет
docs/tvision-rs-guide.pdf.

Как устроено: Markdown → один HTML (вёрстка для печати в style.css) →
PDF через Chromium. Номера страниц в оглавлении и указателе ставятся
вторым проходом: с обложки на каждый заголовок ведёт невидимая ссылка,
поэтому Chromium создаёт для него именованное назначение, и по нему
видно, на какой странице заголовок оказался. Шрифты — статические
TrueType (fonts.py), сжатие — qpdf.
"""

import datetime
import html
import os
import re
import subprocess
import sys
import tempfile

import markdown

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fonts import font_css  # noqa: E402
from pypdf import PdfReader

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
GUIDE = os.path.join(ROOT, "docs", "guide")
HERE = os.path.dirname(os.path.abspath(__file__))
REPO_URL = "https://github.com/cat386/tvision-rs"

# ----------------------------------------------------------------------
# Порядок книги
# ----------------------------------------------------------------------

LESSONS = sorted(f for f in os.listdir(GUIDE) if re.match(r"(0\d|1\d)-.*\.md$", f))
REFERENCE = sorted(f for f in os.listdir(GUIDE) if re.match(r"[23]\d-.*\.md$", f))


def slug(text):
    """Якорь как у GitHub: строчные, без знаков, пробелы → дефисы."""
    t = re.sub(r"<[^>]+>", "", text)
    t = html.unescape(t).strip().lower()
    t = re.sub(r"[^\w\- ]", "", t)
    return t.replace(" ", "-")


def file_id(name):
    return "f-" + name[:-3]


# ----------------------------------------------------------------------
# Markdown → HTML одного файла
# ----------------------------------------------------------------------

MD = markdown.Markdown(
    extensions=["tables", "fenced_code", "codehilite", "sane_lists"],
    extension_configs={"codehilite": {"guess_lang": False, "css_class": "hl"}},
)


def prepare_md(text):
    # ```rust,ignore → ```rust ; блоки ```text и ```toml остаются
    text = re.sub(r"^```(\w+),\w+", r"```\1", text, flags=re.M)
    # Пустая строка перед списками и таблицами после абзаца (markdown-у она нужна).
    # (Нумерованный список — только с «1. »: строка «100. Имена…» внутри
    # абзаца списком не становится.)
    text = re.sub(r"(?m)^([^\n*|#>\-\d ].*)\n(\* |\| |1\. )", r"\1\n\n\2", text)
    return text


def convert(name, text, marks):
    """HTML файла: якоря заголовков с префиксом файла, ссылки внутри
    книги; заголовки 1–3 уровня записываются в `marks` (для оглавления,
    закладок и номеров страниц)."""
    MD.reset()
    body = MD.convert(prepare_md(text))
    fid = file_id(name)

    def head(m):
        level, inner = m.group(1), m.group(2)
        anchor = fid if level == "1" else f"{fid}--{slug(inner)}"
        if level in "123":
            key = f"M{len(marks):04d}"
            marks.append((key, anchor, int(level), re.sub(r"<[^>]+>", "", html.unescape(inner))))
        return f'<h{level} id="{anchor}">{inner}</h{level}>'

    body = re.sub(r"<h([1-6])>(.*?)</h\1>", head, body)

    def link(m):
        href = html.unescape(m.group(1))
        if href.startswith(("http://", "https://", "mailto:")):
            return f'href="{html.escape(href)}"'
        path, _, frag = href.partition("#")
        if not path:
            target = f"{fid}--{frag}" if frag else fid
            return f'href="#{target}"'
        if "/" not in path and path.endswith(".md") and os.path.exists(os.path.join(GUIDE, path)):
            if path == "README.md":
                # Оглавление руководства в книге — разделы «Введение» и «Указатель».
                return f'href="#{"f-README--" + frag if frag else "f-README"}"'
            target = file_id(path) + (f"--{frag}" if frag else "")
            return f'href="#{target}"'
        # Всё остальное (главы docs/*.md, примеры) — ссылка на GitHub.
        full = os.path.normpath(os.path.join("docs/guide", path))
        return f'href="{REPO_URL}/blob/main/{full}{"#" + frag if frag else ""}"'

    body = re.sub(r'href="([^"]+)"', link, body)
    # Таблицы: шире 3 столбцов — помельче.
    body = re.sub(r"<table>(\s*<thead>\s*<tr>(?:\s*<th[^>]*>.*?</th>){4,})", r'<table class="wide">\1', body, flags=re.S)
    # Короткие блоки кода не разрываем между страницами.
    def pre(m):
        lines = m.group(0).count("\n")
        cls = "keep" if lines <= 30 else "long"
        return m.group(0).replace('<div class="hl">', f'<div class="hl {cls}">', 1)

    body = re.sub(r'<div class="hl">.*?</div>', pre, body, flags=re.S)
    return body


# ----------------------------------------------------------------------
# Разделы книги
# ----------------------------------------------------------------------

def read(name):
    with open(os.path.join(GUIDE, name), encoding="utf-8") as f:
        return f.read()


def split_readme():
    """README руководства → «Введение» (до справочника включительно, без
    указателя) и «Указатель» (классы + модули констант)."""
    text = read("README.md")
    text = re.sub(r"(?m)^> Всё руководство одним файлом.*?\n\n", "", text, flags=re.S)
    intro, rest = text.split("### Указатель классов", 1)
    index, tail = rest.split("## Что ещё почитать", 1)
    intro = intro.replace("# Руководство пользователя tvision-rs", "# Введение", 1)
    intro += "\n## Что ещё почитать" + tail
    index = "# Указатель классов\n" + index.replace("### Модули констант", "## Модули констант")
    return intro, index


def build_html(pages=None):
    """pages: якорь → номер страницы (после первого прохода)."""
    marks = []
    intro, index = split_readme()
    parts = []

    parts.append(f'<section class="chapter intro">{convert("README.md", intro, marks)}</section>')
    parts.append('<section class="part"><div class="part-no">Часть 1</div><div class="part-title">Уроки</div>'
                 '<p>По порядку, от пустой программы до приложения с данными. '
                 'К каждому уроку — готовая программа <code>cargo run --example …</code>.</p></section>')
    for f in LESSONS:
        parts.append(f'<section class="chapter">{convert(f, read(f), marks)}</section>')
    parts.append('<section class="part"><div class="part-no">Часть 2</div><div class="part-title">Справочник</div>'
                 '<p>Каждый класс библиотеки: что это, когда нужен, как создать, '
                 'что важно знать, главные методы и пример.</p></section>')
    for f in REFERENCE:
        parts.append(f'<section class="chapter">{convert(f, read(f), marks)}</section>')
    idx = convert("index.md", index, marks)  # якоря f-index…, чтобы не совпасть с введением
    parts.append(f'<section class="chapter index">{idx}</section>')

    body = "\n".join(parts)
    pages = pages or {}

    # Номера страниц у ссылок указателя: «Button · с. 57».
    def idx_pages(m):
        sec = m.group(0)

        def a(mm):
            target = mm.group(1)
            pg = pages.get(target)
            return mm.group(0) + (f'<span class="pg">{pg}</span>' if pg else '<span class="pg">000</span>')

        return re.sub(r'<a href="#([^"]+)">.*?</a>', a, sec, flags=re.S)

    body = re.sub(r'<section class="chapter index">.*?</section>', idx_pages, body, flags=re.S)

    toc = build_toc(marks, pages)
    today = datetime.date.today()
    rev = subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    with open(os.path.join(HERE, "style.css"), encoding="utf-8") as f:
        css = f.read()
    cover = f"""
<section class="cover">
  <div class="cover-top">
    <div class="cover-kicker">Библиотека текстового интерфейса для Rust</div>
    <h1 class="cover-title">tvision-rs</h1>
    <div class="cover-sub">Руководство пользователя</div>
  </div>
  <pre class="cover-screen">  Файл  Контакт
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░░╔═[■]═════════ Контакты ═════════════1═[↑]═╗░░░░░░░
░░║ Имя             │ Телефон          │ ★   ▲░░░░░░░
░░║─────────────────┼──────────────────┼─────■░░░░░░░
░░║ Анна Петрова    │ +7 900 111-22-33 │ ★   ▒░░░░░░░
░░║ Борис Смирнов   │ +7 900 222-33-44 │     ▒░░░░░░░
░░║ Вера Иванова    │ +7 900 333-44-55 │ ★   ▼░░░░░░░
░░╚═══════════════════════════════════════════╝░░░░░░░
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
 Ins Добавить  F4 Изменить  Del Удалить  Alt-X Выход  </pre>
  <div class="dest-links">{"".join(f'<a href="#{a}">.</a>' for _, a, _, _ in marks)}</div>
  <div class="cover-bottom">
    <div>Уроки для начинающих · справочник по каждому классу · данные в JSON и SQLite</div>
    <div class="cover-meta">{today.strftime("%d.%m.%Y")} · ревизия {rev} · {REPO_URL.replace("https://", "")}</div>
  </div>
</section>"""

    return f"""<!doctype html>
<html lang="ru"><head><meta charset="utf-8">
<title>tvision-rs — Руководство пользователя</title>
<style>{font_css()}
{css}</style></head>
<body>
{cover}
{toc}
{body}
</body></html>""", marks


def build_toc(marks, pages):
    rows = []
    for key, anchor, level, title in marks:
        if anchor.startswith("f-index--"):
            continue
        if level == 1:
            cls = "toc1"
        elif level == 2 and (anchor.startswith("f-0") or anchor.startswith("f-1")):
            cls = "toc2"  # разделы уроков — в оглавлении; справочник — только главы
        else:
            continue
        pg = pages.get(anchor, "000")
        rows.append(f'<a class="{cls}" href="#{anchor}"><span class="t">{html.escape(title)}</span><span class="dots"></span><span class="pg">{pg}</span></a>')
    # Части перед первым уроком и первым справочником.
    out = []
    for r in rows:
        if 'href="#f-01-' in r and "toc1" in r:
            out.append('<div class="toc-part">Часть 1. Уроки</div>')
        if 'href="#f-20-' in r and "toc1" in r:
            out.append('<div class="toc-part">Часть 2. Справочник</div>')
        if 'href="#f-index"' in r:
            out.append('<div class="toc-part">Приложение</div>')
        out.append(r)
    return '<section class="toc"><h1 class="toc-title">Содержание</h1>' + "\n".join(out) + "</section>"


# ----------------------------------------------------------------------
# PDF
# ----------------------------------------------------------------------

FOOTER = """<div style="width:100%;font-family:'DejaVu Sans',sans-serif;font-size:7.5pt;color:#8a8f98;
 padding:0 18mm;display:flex;justify-content:space-between;">
 <span>tvision-rs · Руководство пользователя</span><span class="pageNumber"></span></div>"""


def short_ids(html_text):  # → (html, {якорь: короткий})
    """Длинные (кириллические) якоря → короткие a1, a2… : именованные
    назначения в PDF остаются короткими, просмотрщики не ругаются."""
    ids = {}
    for m in re.finditer(r'id="([^"]+)"', html_text):
        ids.setdefault(m.group(1), f"a{len(ids) + 1}")
    broken = sorted({h for h in re.findall(r'href="#([^"]+)"', html_text) if h not in ids})
    if broken:
        print(f"предупреждение: {len(broken)} ссылок внутри книги ведут в никуда: {broken[:8]}")
    html_text = re.sub(r'id="([^"]+)"', lambda m: f'id="{ids[m.group(1)]}"', html_text)
    return re.sub(r'href="#([^"]+)"', lambda m: f'href="#{ids.get(m.group(1), m.group(1))}"', html_text), ids


def render(html_text, out, footer=True):
    from playwright.sync_api import sync_playwright

    with tempfile.NamedTemporaryFile("w", suffix=".html", delete=False, encoding="utf-8") as f:
        f.write(html_text)
        path = f.name
    try:
        with sync_playwright() as p:
            b = p.chromium.launch()
            page = b.new_page()
            page.goto("file://" + path)
            page.evaluate("document.fonts.ready")
            page.pdf(
                path=out,
                prefer_css_page_size=True,  # поля — из @page в style.css (обложка без полей)
                print_background=True,
                display_header_footer=footer,
                header_template="<span></span>",
                footer_template=FOOTER,
                outline=False,  # закладки строим сами (add_outline)
                tagged=True,
            )
            b.close()
    finally:
        os.unlink(path)


def find_pages(pdf, marks, idmap):
    """Якорь заголовка → номер страницы: по именованным назначениям PDF
    (Chromium создаёт их для всех целей ссылок, а на каждый заголовок
    ведёт невидимая ссылка с обложки)."""
    reader = PdfReader(pdf)
    dests = reader.named_destinations
    out = {}
    for _, anchor, _, _ in marks:
        name = idmap.get(anchor, "")
        d = dests.get("/" + name) or dests.get(name)
        out[anchor] = reader.get_destination_page_number(d) + 1 if d is not None else None
    return out, reader


def add_outline(pdf, marks, pages):
    """Закладки: Введение, Часть 1 (уроки и их разделы), Часть 2 (главы
    справочника и разделы), Указатель. Плюс заголовок документа."""
    from pypdf import PdfWriter

    w = PdfWriter(clone_from=pdf)
    pg = lambda a: (pages.get(a) or 1) - 1
    w.add_outline_item("Содержание", 1)
    parent = None
    chapter = None
    for key, anchor, level, title in marks:
        if anchor.startswith("f-index--"):
            continue
        if level == 1:
            if anchor == "f-01-first-program":
                parent = w.add_outline_item("Часть 1. Уроки", pg(anchor) - 1)
            elif anchor.startswith("f-20-"):
                parent = w.add_outline_item("Часть 2. Справочник", pg(anchor) - 1)
            elif anchor in ("f-README", "f-index"):
                parent = None
            chapter = w.add_outline_item(title, pg(anchor), parent=parent)
        elif level == 2 and chapter is not None:
            w.add_outline_item(title, pg(anchor), parent=chapter)
    w.page_mode = "/UseOutlines"
    w.add_metadata({"/Title": "tvision-rs — Руководство пользователя", "/Author": "tvision-rs",
                    "/Subject": "Библиотека текстового интерфейса для Rust: уроки и справочник"})
    with open(pdf, "wb") as f:
        w.write(f)


def compress(pdf):
    """Сжать потоками объектов (qpdf, если установлен): файл меньше в 2–3 раза."""
    import shutil

    if not shutil.which("qpdf"):
        print("qpdf не найден — PDF не сжат")
        return
    tmp = pdf + ".min.pdf"
    r = subprocess.run(["qpdf", "--object-streams=generate", "--compress-streams=y", "--recompress-flate",
                        "--compression-level=9", pdf, tmp])
    if r.returncode in (0, 3) and os.path.exists(tmp):  # 3 — предупреждения
        os.replace(tmp, pdf)


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, "docs", "tvision-rs-guide.pdf")
    tmp = out + ".pass.pdf"
    # Проход 1: где оказались заголовки.
    html1, marks = build_html()
    html1, ids1 = short_ids(html1)
    render(html1, tmp)
    pages, _ = find_pages(tmp, marks, ids1)
    missing = [a for a, p in pages.items() if p is None]
    if missing:
        print(f"предупреждение: не найдены страницы {len(missing)} заголовков: {missing[:5]}")
    pages = {a: p for a, p in pages.items() if p}
    # Проход 2: с номерами страниц в оглавлении и указателе.
    html2, marks2 = build_html(pages)
    html2, ids2 = short_ids(html2)
    with open(os.path.join(HERE, "guide.html"), "w", encoding="utf-8") as f:
        f.write(html2)
    render(html2, out)
    pages2, _ = find_pages(out, marks2, ids2)
    moved = [a for a in pages if pages[a] != pages2.get(a)]
    if moved:
        print(f"предупреждение: сдвинулись {len(moved)} заголовков, например {moved[:3]}")
    os.unlink(tmp)
    # Обложку — отдельно, без колонтитула, поверх первой страницы.
    cover_html = re.sub(r"<section class=\"toc\">.*</body>", "</body>", html2, flags=re.S)
    cover_pdf = out + ".cover.pdf"
    render(cover_html, cover_pdf, footer=False)
    from pypdf import PdfWriter

    w = PdfWriter(clone_from=out)
    w.pages[0].merge_page(PdfReader(cover_pdf).pages[0])
    with open(out, "wb") as f:
        w.write(f)
    os.unlink(cover_pdf)
    add_outline(out, marks2, pages2)
    compress(out)
    print(f"{out}: страниц {len(PdfReader(out).pages)}, заголовков {len(marks2)}")


if __name__ == "__main__":
    main()
