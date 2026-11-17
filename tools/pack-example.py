#!/usr/bin/env python3
"""Packs training-material/doubles-to-go into site/training/arc42/example/doubles-to-go.zip (deterministic).
Also copies arc42.md next to it so people can read the document without unzipping."""
import os, zipfile, shutil
SRC = 'training-material/doubles-to-go'
OUT = 'site/training/arc42/example'
os.makedirs(OUT, exist_ok=True)
files = []
for root, dirs, names in os.walk(SRC):
    dirs[:] = sorted(d for d in dirs if d not in ('node_modules', '.git'))
    for n in sorted(names):
        if n.endswith('.db') or n == '.DS_Store': continue
        files.append(os.path.join(root, n))
with zipfile.ZipFile(os.path.join(OUT, 'doubles-to-go.zip'), 'w', zipfile.ZIP_DEFLATED) as z:
    for f in files:
        zi = zipfile.ZipInfo(os.path.relpath(f, 'training-material'), date_time=(2026, 11, 3, 0, 0, 0))
        zi.compress_type = zipfile.ZIP_DEFLATED
        zi.external_attr = 0o644 << 16
        z.writestr(zi, open(f, 'rb').read())
shutil.copyfile(os.path.join(SRC, 'arc42.md'), os.path.join(OUT, 'arc42.md'))
print(f'packed {len(files)} files')
