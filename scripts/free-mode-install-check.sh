#!/bin/bash
# Check FREE MODE installation completeness

set -e

REPO_ROOT="$(pwd)"
ISSUES=()
WARNINGS=()

echo "🔍 FREE MODE Installation Check"
echo "=================================="
echo ""

# Check 1: .env.example has FREE_MODE section
echo -n "Checking .env.example... "
if grep -q "FREE MODE CORE" "$REPO_ROOT/.env.example"; then
    echo "✅"
else
    echo "❌"
    ISSUES+=(".env.example missing FREE_MODE section")
fi

# Check 2: litellm.config.yaml exists
echo -n "Checking litellm.config.yaml... "
if [ -f "$REPO_ROOT/free-mode/litellm.config.yaml" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/litellm.config.yaml not found")
fi

# Check 3: docker-compose.free-mode.yml exists
echo -n "Checking docker-compose.free-mode.yml... "
if [ -f "$REPO_ROOT/docker-compose.free-mode.yml" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("docker-compose.free-mode.yml not found")
fi

# Check 4: providers.json exists
echo -n "Checking providers.json... "
if [ -f "$REPO_ROOT/free-mode/providers.json" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/providers.json not found")
fi

# Check 5: Free mode README exists
echo -n "Checking free-mode/README.md... "
if [ -f "$REPO_ROOT/free-mode/README.md" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/README.md not found")
fi

# Check 6: SECURITY.md exists
echo -n "Checking free-mode/SECURITY.md... "
if [ -f "$REPO_ROOT/free-mode/SECURITY.md" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/SECURITY.md not found")
fi

# Check 7: Python client modules
echo -n "Checking free_mode/__init__.py... "
if [ -f "$REPO_ROOT/free_mode/__init__.py" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free_mode/__init__.py not found")
fi

# Check 8: Test scripts exist
echo -n "Checking test scripts... "
if [ -f "$REPO_ROOT/free-mode/scripts/test-provider.py" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/scripts/test-provider.py not found")
fi

# Check 9: Start/stop scripts exist
echo -n "Checking start/stop scripts... "
if [ -f "$REPO_ROOT/free-mode/scripts/start-free-mode.sh" ] && \
   [ -f "$REPO_ROOT/free-mode/scripts/stop-free-mode.sh" ]; then
    echo "✅"
else
    echo "❌"
    ISSUES+=("free-mode/scripts/start-free-mode.sh or stop-free-mode.sh not found")
fi

# Check 10: Docker available
echo -n "Checking Docker... "
if command -v docker &> /dev/null; then
    echo "✅"
else
    echo "⚠️"
    WARNINGS+=("Docker not installed — proxy cannot start")
fi

# Summary
echo ""
echo "=================================="
if [ ${#ISSUES[@]} -eq 0 ]; then
    echo "✅ All checks passed!"
else
    echo "❌ Found ${#ISSUES[@]} issue(s):"
    for issue in "${ISSUES[@]}"; do
        echo "  - $issue"
    done
fi

if [ ${#WARNINGS[@]} -gt 0 ]; then
    echo ""
    echo "⚠️ Warnings:"
    for warning in "${WARNINGS[@]}"; do
        echo "  - $warning"
    done
fi

echo ""
echo "Next steps:"
echo "1. cp .env.example .env"
echo "2. Edit .env and add at least one provider API key"
echo "3. bash free-mode/scripts/start-free-mode.sh"
echo "4. export FREE_MODE=true"
echo "5. hermes 'test'"
