#!/bin/bash
# Test Coverage Script for Aether Matrix Bot
# 
# Prerequisites:
#   - Linux: cargo install cargo-tarpaulin
#   - macOS: cargo install cargo-llvm-cov
#
# Usage:
#   ./scripts/coverage.sh              # Run coverage and generate report
#   ./scripts/coverage.sh html         # Generate HTML report only
#   ./scripts/coverage.sh lcov         # Generate LCOV report for CI
#   ./scripts/coverage.sh clean        # Clean coverage artifacts

set -e

# Configuration
TOOL=""
REPORT_DIR="target/coverage"
HTML_DIR="target/coverage/html"

# Detect platform and available tool
detect_tool() {
    if command -v cargo-tarpaulin &> /dev/null; then
        TOOL="tarpaulin"
        echo "✓ Using cargo-tarpaulin (Linux)"
    elif command -v cargo-llvm-cov &> /dev/null; then
        TOOL="llvm-cov"
        echo "✓ Using cargo-llvm-cov (cross-platform)"
    else
        echo "✗ No coverage tool found!"
        echo ""
        echo "Install one of the following:"
        echo "  Linux:   cargo install cargo-tarpaulin"
        echo "  macOS:   cargo install cargo-llvm-cov"
        echo "  Any:     cargo install grcov (requires --instrument-coverage)"
        echo ""
        exit 1
    fi
}

# Run coverage with tarpaulin
run_tarpaulin() {
    local mode="${1:-all}"
    
    case "$mode" in
        html)
            echo "Generating HTML coverage report..."
            cargo tarpaulin --out Html --output-dir "$HTML_DIR"
            echo "✓ HTML report: $HTML_DIR/index.html"
            ;;
        lcov)
            echo "Generating LCOV report..."
            cargo tarpaulin --out Lcov --output-dir "$REPORT_DIR"
            echo "✓ LCOV report: $REPORT_DIR/lcov.info"
            ;;
        xml)
            echo "Generating XML report..."
            cargo tarpaulin --out Xml --output-dir "$REPORT_DIR"
            echo "✓ XML report: $REPORT_DIR/coverage.xml"
            ;;
        all|*)
            echo "Running full coverage suite..."
            cargo tarpaulin --out Html --out Lcov --output-dir "$REPORT_DIR"
            echo "✓ Reports generated in $REPORT_DIR/"
            ;;
    esac
}

# Run coverage with llvm-cov
run_llvm_cov() {
    local mode="${1:-all}"
    
    case "$mode" in
        html)
            echo "Generating HTML coverage report..."
            cargo llvm-cov --html --output-dir "$HTML_DIR"
            echo "✓ HTML report: $HTML_DIR/index.html"
            ;;
        lcov)
            echo "Generating LCOV report..."
            cargo llvm-cov --lcov --output-path "$REPORT_DIR/lcov.info"
            echo "✓ LCOV report: $REPORT_DIR/lcov.info"
            ;;
        text)
            echo "Generating text summary..."
            cargo llvm-cov
            ;;
        all|*)
            echo "Running full coverage suite..."
            cargo llvm-cov --html --lcov --output-dir "$REPORT_DIR"
            echo "✓ Reports generated in $REPORT_DIR/"
            ;;
    esac
}

# Clean coverage artifacts
clean() {
    echo "Cleaning coverage artifacts..."
    rm -rf "$REPORT_DIR"
    rm -rf "target/llvm-cov-target"
    rm -rf "target/debug/deps/*.gcno"
    rm -rf "target/debug/deps/*.gcda"
    echo "✓ Cleaned"
}

# Main
main() {
    local action="${1:-run}"
    
    case "$action" in
        clean)
            clean
            ;;
        html|lcov|xml|text)
            detect_tool
            if [ "$TOOL" = "tarpaulin" ]; then
                run_tarpaulin "$action"
            else
                run_llvm_cov "$action"
            fi
            ;;
        run|all|*)
            detect_tool
            if [ "$TOOL" = "tarpaulin" ]; then
                run_tarpaulin "all"
            else
                run_llvm_cov "all"
            fi
            ;;
    esac
}

main "$@"
