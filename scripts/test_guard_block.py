#!/usr/bin/env python3
"""Test the runtime guard for required skills.

This script temporarily marks a nonexistent skill as required in
`skills/SKILL_REGISTRY.json`, invokes a dangerous tool via the registry,
prints the result, and restores the original registry file.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))
REG = ROOT.joinpath('skills', 'SKILL_REGISTRY.json')

bak = REG.read_text(encoding='utf-8')
try:
    data = json.loads(bak)
except Exception:
    print('Failed to parse existing SKILL_REGISTRY.json')
    sys.exit(1)

data['__missing_required_test__'] = {
    'path': 'skills/__missing_required_test__',
    'description': 'Temporary test',
    'required': True,
    'enabled': True,
}

REG.write_text(json.dumps(data, indent=4), encoding='utf-8')

try:
    # Call the registry dispatch for a dangerous tool
    from model_tools import handle_function_call

    result = handle_function_call('write_file', {'path': 'dummy.txt', 'content': 'x'})
    print('Dispatch result:', result)
finally:
    REG.write_text(bak, encoding='utf-8')
