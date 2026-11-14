#!/usr/bin/env python3
"""Produce the mod's two deterministic files in target/:

- `jei-<version>.hnmod`: the native bundle (`horizon-native.json` and
  `mixins/jei/`, its `.rs` and `.toml` sources only, as the launcher takes).
- `jei-<version>.hmod`: the content half, the GUI sprites under
  `assets/jei/textures/gui/` (JEI 1.12's, MIT; see NOTICE). A native bundle
  may not carry them, so they ship beside it.
"""
import hashlib
import json
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / 'mixins/jei'
CONTENT = ROOT / 'content/jei'
STAMP = (2026, 1, 1, 0, 0, 0)


def manifest():
    mixin = tomllib.loads((BUNDLE / 'horizon-mixin.toml').read_text())
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    content = tomllib.loads((CONTENT / 'horizon-mod.toml').read_text())
    version = mixin['mod']['version']
    assert version == content['version'], 'the native and content versions disagree'
    assert mixin['mod']['id'] == content['id'], 'the native and content ids disagree'
    rev = mixin['compatibility']['horizon_revision']
    assert compat['horizon_revision'] == rev, 'compatibility.json and the bundle pin different commits'
    meta = dict(schema=1, id=mixin['mod']['id'], version=version, game=compat['game'], horizon=rev,
                mixins=compat['horizon_mixins_revision'])
    if 'hooks' in compat:
        meta['hooks'] = compat['hooks']
    return meta


def tree(base: Path, prefix: str, allowed) -> dict:
    out = {}
    for path in sorted(base.rglob('*')):
        rel = path.relative_to(base)
        if path.is_file() and 'target' not in rel.parts:
            assert allowed(path), f'unexpected file {path}'
            out[f'{prefix}{rel.as_posix()}'] = path.read_bytes()
    return out


def write(out: Path, files: dict) -> dict:
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(files.items()):
            info = zipfile.ZipInfo(name, STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    data = out.read_bytes()
    return dict(file=str(out), size=len(data), sha256=hashlib.sha256(data).hexdigest())


def main():
    meta = manifest()
    native = {'horizon-native.json': (json.dumps(meta, indent=2, sort_keys=True) + '\n').encode()}
    native.update(tree(BUNDLE, 'mixins/jei/', lambda p: p.suffix in ('.rs', '.toml')))
    content = tree(CONTENT, '', lambda p: p.suffix in ('.png', '.toml', '.txt', '.json'))
    stem = f"{meta['id']}-{meta['version']}"
    print(json.dumps(dict(native=write(ROOT / 'target' / f'{stem}.hnmod', native),
                          content=write(ROOT / 'target' / f'{stem}.hmod', content)), indent=2))


if __name__ == '__main__':
    main()
