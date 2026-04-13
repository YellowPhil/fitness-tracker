"""Workout generator service package."""

from __future__ import annotations

import sys
from pathlib import Path


_generated_root = Path(__file__).resolve().parent / "generated"
if _generated_root.is_dir():
    generated_path = str(_generated_root)
    if generated_path not in sys.path:
        sys.path.insert(0, generated_path)
