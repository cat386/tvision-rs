"""Статические TrueType-шрифты для PDF.

Chromium встраивает вариативные шрифты (Lora) и OpenType с контурами CFF
(Inter .otf) как Type3 — отдельной копией на каждой странице: PDF
раздувается в разы, а текст хуже копируется. Поэтому перед сборкой
делаем из них обычные TrueType-начертания (fontTools) и подключаем через
@font-face. Готовые файлы кешируются в tools/guide-pdf/.fonts/.
"""

import glob
import os

HERE = os.path.dirname(os.path.abspath(__file__))
CACHE = os.path.join(HERE, ".fonts")

# Где искать исходники (Linux; на других системах — поправьте пути).
SEARCH = ["/usr/share/fonts", os.path.expanduser("~/.local/share/fonts"), os.path.expanduser("~/Library/Fonts"), "/Library/Fonts"]


def find(name):
    for root in SEARCH:
        hits = glob.glob(os.path.join(root, "**", name), recursive=True)
        if hits:
            return hits[0]
    return None


def instance(src, dst, wght):
    """Вариативный шрифт → одно начертание."""
    from fontTools.ttLib import TTFont
    from fontTools.varLib import instancer

    f = instancer.instantiateVariableFont(TTFont(src), {"wght": wght})
    f.save(dst)


def otf_to_ttf(src, dst):
    """OpenType CFF → TrueType (кубические кривые → квадратичные)."""
    from fontTools.pens.cu2quPen import Cu2QuPen
    from fontTools.pens.ttGlyphPen import TTGlyphPen
    from fontTools.ttLib import TTFont, newTable

    font = TTFont(src)
    order = font.getGlyphOrder()
    gs = font.getGlyphSet()
    glyf = newTable("glyf")
    glyf.glyphOrder = order
    glyf.glyphs = {}
    for name in order:
        pen = TTGlyphPen(gs)
        gs[name].draw(Cu2QuPen(pen, 1.0, reverse_direction=True))
        glyf.glyphs[name] = pen.glyph()
    font["loca"] = newTable("loca")
    font["glyf"] = glyf
    maxp = newTable("maxp")
    maxp.tableVersion = 0x00010000
    for k in ("maxZones", "maxTwilightPoints", "maxStorage", "maxFunctionDefs", "maxInstructionDefs",
              "maxStackElements", "maxSizeOfInstructions", "maxComponentElements"):
        setattr(maxp, k, 0)
    maxp.maxZones = 1
    font["maxp"] = maxp
    post = font["post"]
    post.formatType = 2.0
    post.extraNames = []
    post.mapping = {}
    post.glyphOrder = order
    for t in ("CFF ", "VORG"):
        if t in font:
            del font[t]
    font["head"].glyphDataFormat = 0
    font.sfntVersion = "\x00\x01\x00\x00"
    font.save(dst)


# семейство в CSS, вес, стиль, функция, исходник, параметр
FACES = [
    ("BookSerif", 400, "normal", instance, "Lora-Variable.ttf", 400),
    ("BookSerif", 700, "normal", instance, "Lora-Variable.ttf", 700),
    ("BookSerif", 400, "italic", instance, "Lora-Italic-Variable.ttf", 400),
    ("BookSerif", 700, "italic", instance, "Lora-Italic-Variable.ttf", 700),
    ("BookSans", 300, "normal", otf_to_ttf, "Inter-Light.otf", None),
    ("BookSans", 400, "normal", otf_to_ttf, "Inter-Regular.otf", None),
    ("BookSans", 600, "normal", otf_to_ttf, "Inter-SemiBold.otf", None),
    ("BookSans", 700, "normal", otf_to_ttf, "Inter-Bold.otf", None),
    ("BookSans", 800, "normal", otf_to_ttf, "Inter-ExtraBold.otf", None),
]


def font_css():
    """@font-face для найденных шрифтов (недостающие — пропускаются,
    тогда CSS возьмёт запасные DejaVu)."""
    os.makedirs(CACHE, exist_ok=True)
    rules = []
    for family, weight, style, fn, src_name, arg in FACES:
        dst = os.path.join(CACHE, f"{family}-{weight}-{style}.ttf")
        if not os.path.exists(dst):
            src = find(src_name)
            if not src:
                print(f"шрифт {src_name} не найден — будет запасной")
                continue
            if arg is None:
                fn(src, dst)
            else:
                fn(src, dst, arg)
        rules.append(f'@font-face {{ font-family: "{family}"; font-weight: {weight}; font-style: {style}; '
                     f'src: url("file://{dst}") format("truetype"); }}')
    return "\n".join(rules)
