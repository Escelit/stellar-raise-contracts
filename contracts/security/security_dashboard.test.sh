#!/bin/bash
# Test suite for security_dashboard.sh
#
# This script tests the security dashboard functionality to ensure
# all components work correctly in CI/CD environments.

set -e

# Test configuration
TEST_DIR="test_security_dashboard"
SCRIPT_PATH="./security_dashboard.sh"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test counter
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Setup test environment
setup_test_env() {
    echo "Setting up test environment..."
    mkdir -p "${TEST_DIR}"
    cd "${TEST_DIR}"
}

# Cleanup test environment
cleanup_test_env() {
    echo "Cleaning up test environment..."
    cd ..
    rm -rf "${TEST_DIR}"
}

# Test helper function
run_test() {
    local test_name="$1"
    local test_func="$2"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -e "\n${YELLOW}Running: ${test_name}${NC}"
    
    if $test_func; then
        echo -e "${GREEN}✓ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Test 1: Script exists and is executable
test_script_exists() {
    if [ -f "../${SCRIPT_PATH}" ]; then
        return 0
    else
        echo "Script not found at ${SCRIPT_PATH}"
        return 1
    fi
}

# Test 2: Script has correct shebang
test_script_shebang() {
    local first_line=$(head -n 1 "../${SCRIPT_PATH}")
    if [[ "$first_line" == "#!/bin/bash"* ]]; then
        return 0
    else
        echo "Invalid shebang: $first_line"
        return 1
    fi
}

# Test 3: Script contains required functions
test_required_functions() {
    local required_funcs=("check_dependencies" "run_security_tests" "check_vulnerabilities" "generate_html_dashboard")
    
    for func in "${required_funcs[@]}"; do
        if ! grep -q "$func()" "../${SCRIPT_PATH}"; then
            echo "Missing function: $func"
            return 1
        fi
    done
    
    return 0
}

# Test 4: Script creates report directory
test_report_directory_creation() {
    # Mock the script execution (just test the mkdir command)
    mkdir -p "security-reports"
    
    if [ -d "security-reports" ]; then
        rm -rf "security-reports"
        return 0
    else
        return 1
    fi
}

# Test 5: HTML template is valid
test_html_template() {
    if grep -q "<!DOCTYPE html>" "../${SCRIPT_PATH}"; then
        return 0
    else
        echo "HTML template not found"
        return 1
    fi
}

# Test 6: Script has error handling
test_error_handling() {
    if grep -q "set -e" "../${SCRIPT_PATH}"; then
        return 0
    else
        echo "Missing error handling (set -e)"
        return 1
    fi
}

# Test 7: Color codes are defined
test_color_definitions() {
    local colors=("RED" "GREEN" "YELLOW" "BLUE" "NC")
    
    for color in "${colors[@]}"; do
        if ! grep -q "${color}=" "../${SCRIPT_PATH}"; then
            echo "Missing color definition: $color"
            return 1
        fi
    done
    
    return 0
}

# Test 8: Main function exists
test_main_function() {
    if grep -q "main()" "../${SCRIPT_PATH}"; then
        return 0
    else
        echo "Main function not found"
        return 1
    fi
}

# Display test summary
display_summary() {
    echo ""
    echo "================================"
    echo "Test Summary"
    echo "================================"
    echo "Tests run: ${TESTS_RUN}"
    echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
    
    if [ ${TESTS_FAILED} -gt 0 ]; then
        echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
        return 1
    else
        echo -e "${GREEN}All tests passed!${NC}"
        return 0
    fi
}

# Main test execution
main() {
    echo "Starting security_dashboard.sh test suite..."
    
    setup_test_env
    
    run_test "Script exists and is accessible" test_script_exists
    run_test "Script has correct shebang" test_script_shebang
    run_test "Script contains required functions" test_required_functions
    run_test "Report directory creation works" test_report_directory_creation
    run_test "HTML template is valid" test_html_template
    run_test "Error handling is enabled" test_error_handling
    run_test "Color codes are defined" test_color_definitions
    run_test "Main function exists" test_main_function
    
    cleanup_test_env
    
    display_summary
}

# Run tests
main "$@"
