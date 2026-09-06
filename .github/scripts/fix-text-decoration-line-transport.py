from pathlib import Path

path = Path('.github/scripts/apply-text-decoration-line-candidate.py')
text = path.read_text()
old = '''needle = '        font_variant_numeric_observations,\\n        overscroll_behavior_x_observations,'
count = value.count(needle)
if count != 3:
    raise SystemExit(f'tuple carry: expected 3 anchors, found {count}')
value = value.replace(
    needle,
    '        font_variant_numeric_observations,\\n        text_decoration_line_observations,\\n        overscroll_behavior_x_observations,',
)
'''
new = '''needle = '        font_variant_numeric_observations,\\n        overscroll_behavior_x_observations,'
nested_needle = '            font_variant_numeric_observations,\\n            overscroll_behavior_x_observations,'
count = value.count(needle)
nested_count = value.count(nested_needle)
if count + nested_count != 3:
    raise SystemExit(
        f'tuple carry: expected 3 anchors, found {count} outer + {nested_count} nested'
    )
value = value.replace(
    needle,
    '        font_variant_numeric_observations,\\n        text_decoration_line_observations,\\n        overscroll_behavior_x_observations,',
)
value = value.replace(
    nested_needle,
    '            font_variant_numeric_observations,\\n            text_decoration_line_observations,\\n            overscroll_behavior_x_observations,',
)
'''
if text.count(old) != 1:
    raise SystemExit(f'expected one tuple block, found {text.count(old)}')
path.write_text(text.replace(old, new, 1))
