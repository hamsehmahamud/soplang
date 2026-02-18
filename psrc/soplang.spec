# -*- mode: python ; coding: utf-8 -*-
# Run from project root: pyinstaller psrc/soplang.spec

import os
import platform
block_cipher = None

# Resolve project root (parent of psrc/) when spec is in psrc/soplang.spec
_spec_path = os.path.abspath(SPEC if globals().get('SPEC') else __file__)
_spec_dir = os.path.dirname(_spec_path)
_project_root = os.path.dirname(_spec_dir)

icon_file = os.path.join(_project_root, 'windows', 'soplang_icon.ico')
if not os.path.exists(icon_file):
    print(f"Warning: Icon file {icon_file} not found. The executable will use a default icon.")
    icon_file = None

a = Analysis(
    [os.path.join(_project_root, 'main.py')],
    pathex=[_project_root],
    binaries=[],
    datas=[(os.path.join(_project_root, 'psrc'), 'psrc')],
    hiddenimports=[
        'psrc.core',
        'psrc.runtime',
        'psrc.stdlib',
        'psrc.utils',
        'colorama',
        'prompt_toolkit',
        'prompt_toolkit.clipboard',
        'prompt_toolkit.completion',
        'prompt_toolkit.filters',
        'prompt_toolkit.history',
        'prompt_toolkit.key_binding',
        'prompt_toolkit.layout',
        'prompt_toolkit.lexers',
        'prompt_toolkit.styles',
        'prompt_toolkit.shortcuts',
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='soplang',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=icon_file,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='soplang',
)
