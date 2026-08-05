#!/bin/bash
# Strix Batch Security Testing - Convenient CLI wrapper
# Usage: ./run-batch-test.sh [options]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Defaults
WORKERS=3
SCAN_MODE="standard"
OUTPUT_DIR="./security-scans"
SAVE_REPORT=false
SHOW_HELP=false
TARGETS=()
TARGET_FILE=""
USE_GITHUB_ORG=""
USE_GITHUB_USER=""

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(dirname "$SCRIPT_DIR")"

print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}  🔐 Strix Batch Security Testing                                ${BLUE}║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════╝${NC}"
}

print_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --target URL              Add a target (GitHub repo or URL)"
    echo "  -f, --target-file FILE        Read targets from file (one per line)"
    echo "  -o, --org ORG                 Scan all repos in GitHub organization"
    echo "  -u, --user USER               Scan all repos of GitHub user"
    echo "  -w, --workers N               Number of parallel agents (default: 3)"
    echo "  -m, --scan-mode MODE          Scan depth: quick|standard|full (default: standard)"
    echo "  -d, --output-dir DIR          Output directory (default: ./security-scans)"
    echo "  -r, --save-report             Save JSON report"
    echo "  -h, --help                    Show this help message"
    echo ""
    echo "Examples:"
    echo "  # Test single repo"
    echo "  $0 --target https://github.com/org/repo"
    echo ""
    echo "  # Test multiple repos"
    echo "  $0 -t https://github.com/org/app1 -t https://github.com/org/app2"
    echo ""
    echo "  # Test all org repos"
    echo "  $0 --org mycompany --workers 3"
    echo ""
    echo "  # Test from file"
    echo "  $0 --target-file targets.txt --scan-mode quick --save-report"
}

check_prerequisites() {
    # Check Strix
    if ! command -v strix &> /dev/null; then
        echo -e "${RED}✗ Strix not found${NC}"
        echo "  Install with: curl -sSL https://strix.ai/install | bash"
        exit 1
    fi

    # Check Python
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}✗ Python 3 not found${NC}"
        exit 1
    fi

    # Check Docker
    if ! docker ps &> /dev/null; then
        echo -e "${YELLOW}⚠ Docker is not running${NC}"
        echo "  Starting Docker..."
        docker daemon 2>/dev/null || (
            echo -e "${YELLOW}  Note: You may need to start Docker manually${NC}"
        )
    fi

    # Check LLM API key
    if [ -z "$LLM_API_KEY" ]; then
        echo -e "${RED}✗ LLM_API_KEY environment variable not set${NC}"
        echo "  Set with: export LLM_API_KEY='your-key'"
        exit 1
    fi

    # Check STRIX_LLM
    if [ -z "$STRIX_LLM" ]; then
        echo -e "${YELLOW}⚠ STRIX_LLM not set, using default: openai/gpt-5.4${NC}"
        export STRIX_LLM="openai/gpt-5.4"
    fi

    echo -e "${GREEN}✓ Prerequisites check passed${NC}"
}

print_config() {
    echo -e "\n${BLUE}Configuration:${NC}"
    echo "  LLM Provider:      $STRIX_LLM"
    echo "  Scan Mode:         $SCAN_MODE"
    echo "  Parallel Workers:  $WORKERS"
    echo "  Output Directory:  $OUTPUT_DIR"
    echo "  Save Report:       $SAVE_REPORT"

    if [ ${#TARGETS[@]} -gt 0 ]; then
        echo "  Targets:"
        for target in "${TARGETS[@]}"; do
            echo "    - $target"
        done
    fi
}

run_batch_test() {
    if [ ${#TARGETS[@]} -eq 0 ] && [ -z "$TARGET_FILE" ] && \
       [ -z "$USE_GITHUB_ORG" ] && [ -z "$USE_GITHUB_USER" ]; then
        echo -e "${RED}✗ No targets specified${NC}"
        print_help
        exit 1
    fi

    check_prerequisites
    print_config

    echo -e "\n${BLUE}Starting security tests...${NC}\n"

    # Build command
    cmd=("python3" "$SCRIPT_DIR/batch_security_test.py")

    if [ ${#TARGETS[@]} -gt 0 ]; then
        cmd+=(--targets "${TARGETS[@]}")
    fi

    if [ -n "$TARGET_FILE" ]; then
        cmd+=(--target-file "$TARGET_FILE")
    fi

    cmd+=(
        --output-dir "$OUTPUT_DIR"
        --workers "$WORKERS"
        --scan-mode "$SCAN_MODE"
        --llm-provider "$STRIX_LLM"
    )

    if [ "$SAVE_REPORT" = true ]; then
        cmd+=(--save-report)
    fi

    # Run tests
    "${cmd[@]}"

    # Show results summary
    if [ -f "$OUTPUT_DIR/batch-report.json" ] && command -v jq &> /dev/null; then
        echo -e "\n${BLUE}=== Scan Summary ===${NC}"
        jq '.batch_summary' "$OUTPUT_DIR/batch-report.json"
    fi

    echo -e "\n${GREEN}✓ Security testing completed${NC}"
    echo -e "  Results: ${BLUE}$OUTPUT_DIR${NC}"
}

run_github_org_test() {
    if [ -z "$USE_GITHUB_ORG" ]; then
        echo -e "${RED}✗ GitHub organization not specified${NC}"
        exit 1
    fi

    check_prerequisites

    echo -e "${BLUE}Starting GitHub organization scan...${NC}\n"

    python3 "$SCRIPT_DIR/github_multi_agent_test.py" \
        --org "$USE_GITHUB_ORG" \
        --workers "$WORKERS" \
        --output-dir "$OUTPUT_DIR"

    echo -e "\n${GREEN}✓ GitHub organization scan completed${NC}"
    echo -e "  Results: ${BLUE}$OUTPUT_DIR${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--target)
            TARGETS+=("$2")
            shift 2
            ;;
        -f|--target-file)
            TARGET_FILE="$2"
            shift 2
            ;;
        -o|--org)
            USE_GITHUB_ORG="$2"
            shift 2
            ;;
        -u|--user)
            USE_GITHUB_USER="$2"
            shift 2
            ;;
        -w|--workers)
            WORKERS="$2"
            shift 2
            ;;
        -m|--scan-mode)
            SCAN_MODE="$2"
            shift 2
            ;;
        -d|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -r|--save-report)
            SAVE_REPORT=true
            shift
            ;;
        -h|--help)
            print_header
            print_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            print_help
            exit 1
            ;;
    esac
done

# Main execution
print_header

if [ -n "$USE_GITHUB_ORG" ] || [ -n "$USE_GITHUB_USER" ]; then
    if [ -n "$USE_GITHUB_ORG" ]; then
        run_github_org_test
    fi
else
    run_batch_test
fi

exit $?
