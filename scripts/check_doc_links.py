#!/usr/bin/env python3
"""scripts/check_doc_links.py -- thin shim post-P60 SIMPLIFY-08.

The canonical home is quality/gates/docs-build/link-resolution.py. This
shim survives for one merge cycle per OP-5 reversibility (any hidden
caller surfaces). P63 SIMPLIFY-12 audits whether to delete or keep.
"""
import os
import sys

target = os.path.join(
    os.path.dirname(os.path.realpath(__file__)),
    "..",
    "quality",
    "gates",
    "docs-build",
    "link-resolution.py",
)
os.execv(sys.executable, [sys.executable, target] + sys.argv[1:])
