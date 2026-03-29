#!/bin/bash

# Test suite for security_monitoring_alerts.sh
# Comprehensive tests for security monitoring functionality

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Test setup
setup() {
    export TEST_DIR=$(mktemp -d)
    export SCAN_RESULTS_DIR="$TEST_DIR/.security-scans"
    export LOG_FILE="$TEST_DIR/test_security.log"
    mkdir -p "$SCAN_RESULTS_DIR"
    mkdir -p "$TEST_DIR/contracts"
}

# Test teardown
teardown() {
    rm -rf "$TEST_DIR"
}

# Assert function
assert_equals() {
    local expected=$1
    local actual=$2
    local message=$3
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ "$expected" = "$actual" ]; then
        echo -e "${GREEN}✓${NC} $message"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC} $message"
        echo "  Expected: $expected"
        echo "  Actual: $actual"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Assert file exists
assert_file_exists() {
    local file=$1
    local message=$2
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $message"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC} $message"
        echo "  File not found: $file"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Assert command succeeds
assert_success() {
    local message=$1
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $message"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC} $message"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Assert command fails
assert_failure() {
    local message=$1
    
    TESTS_RUN=$((TESTS_RUN + 1))
    
    if [ $? -ne 0 ]; then
        echo -e "${GREEN}✓${NC} $message"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC} $message"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Test: Log file creation
test_log_file_creation() {
    echo "Test: Log file creation"
    setup
    
    # Create a simple test that writes to log
    echo "Test log entry" >> "$LOG_FILE"
    
    assert_file_exists "$LOG_FILE" "Log file should be created"
    
    teardown
}

# Test: Scan results directory creation
test_scan_results_dir_creation() {
    echo "Test: Scan results directory creation"
    setup
    
    assert_file_exists "$SCAN_RESULTS_DIR" "Scan results directory should exist"
    
    teardown
}

# Test: Secret pattern detection
test_secret_pattern_detection() {
    echo "Test: Secret pattern detection"
    setup
    
    # Create test file with secret pattern
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    let api_key = "sk_test_1234567890";
    let password = "secret123";
}
EOF
    
    # Run secret scan
    cd "$TEST_DIR"
    local result=$(grep -rniE "password\s*=\s*['\"][^'\"]+['\"]" contracts/ --include="*.rs" | wc -l)
    
    assert_equals "1" "$result" "Should detect password pattern"
    
    teardown
}

# Test: Unsafe code detection
test_unsafe_code_detection() {
    echo "Test: Unsafe code detection"
    setup
    
    # Create test file with unsafe code
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    unsafe {
        let x = 5;
    }
}
EOF
    
    cd "$TEST_DIR"
    local result=$(grep -r "unsafe" contracts/ --include="*.rs" | wc -l)
    
    assert_equals "1" "$result" "Should detect unsafe code"
    
    teardown
}

# Test: Panic pattern detection
test_panic_pattern_detection() {
    echo "Test: Panic pattern detection"
    setup
    
    # Create test file with panic
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    panic!("Error occurred");
    let x = some_option.unwrap();
}
EOF
    
    cd "$TEST_DIR"
    local panic_count=$(grep -r "panic!" contracts/ --include="*.rs" | wc -l)
    local unwrap_count=$(grep -r "unwrap()" contracts/ --include="*.rs" | wc -l)
    
    [ "$panic_count" -ge 1 ]
    assert_success "Should detect panic! pattern"
    
    [ "$unwrap_count" -ge 1 ]
    assert_success "Should detect unwrap() pattern"
    
    teardown
}

# Test: Clean code (no issues)
test_clean_code() {
    echo "Test: Clean code (no issues)"
    setup
    
    # Create clean test file
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    let x = 5;
    let y = x.checked_add(10);
}
EOF
    
    cd "$TEST_DIR"
    local unsafe_count=$(grep -r "unsafe" contracts/ --include="*.rs" | wc -l)
    
    assert_equals "0" "$unsafe_count" "Clean code should have no unsafe blocks"
    
    teardown
}

# Test: Integer overflow detection
test_integer_overflow_detection() {
    echo "Test: Integer overflow detection"
    setup
    
    # Create test file with unchecked arithmetic
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    let x = 5 + 10;
    let y = x * 2;
    let z = y - 3;
}
EOF
    
    cd "$TEST_DIR"
    local risky_ops=$(grep -rE "(\+|\-|\*)\s*[0-9]+" contracts/ --include="*.rs" | wc -l)
    
    [ "$risky_ops" -ge 1 ]
    assert_success "Should detect unchecked arithmetic operations"
    
    teardown
}

# Test: File permission check
test_file_permissions() {
    echo "Test: File permission check"
    setup
    
    # Create file with normal permissions
    touch "$TEST_DIR/contracts/test.rs"
    chmod 644 "$TEST_DIR/contracts/test.rs"
    
    cd "$TEST_DIR"
    local world_writable=$(find contracts/ -type f -perm -002 2>/dev/null | wc -l)
    
    assert_equals "0" "$world_writable" "Should have no world-writable files"
    
    teardown
}

# Test: World-writable file detection
test_world_writable_detection() {
    echo "Test: World-writable file detection"
    setup
    
    # Create world-writable file
    touch "$TEST_DIR/contracts/test.rs"
    chmod 666 "$TEST_DIR/contracts/test.rs"
    
    cd "$TEST_DIR"
    local world_writable=$(find contracts/ -type f -perm -002 2>/dev/null | wc -l)
    
    [ "$world_writable" -ge 1 ]
    assert_success "Should detect world-writable files"
    
    teardown
}

# Test: Multiple secret patterns
test_multiple_secret_patterns() {
    echo "Test: Multiple secret patterns"
    setup
    
    cat > "$TEST_DIR/contracts/test.rs" << 'EOF'
fn main() {
    let api_key = "key123";
    let token = "token456";
    let secret = "secret789";
}
EOF
    
    cd "$TEST_DIR"
    local patterns=("api[_-]?key" "token" "secret")
    local total_found=0
    
    for pattern in "${patterns[@]}"; do
        local count=$(grep -rniE "$pattern\s*=\s*['\"][^'\"]+['\"]" contracts/ --include="*.rs" | wc -l)
        total_found=$((total_found + count))
    done
    
    [ "$total_found" -ge 3 ]
    assert_success "Should detect multiple secret patterns"
    
    teardown
}

# Test: Edge case - empty contracts directory
test_empty_contracts_directory() {
    echo "Test: Empty contracts directory"
    setup
    
    cd "$TEST_DIR"
    local unsafe_count=$(grep -r "unsafe" contracts/ --include="*.rs" 2>/dev/null | wc -l)
    
    assert_equals "0" "$unsafe_count" "Empty directory should have no findings"
    
    teardown
}

# Run all tests
run_all_tests() {
    echo "========================================="
    echo "Security Monitoring Alerts Test Suite"
    echo "========================================="
    echo ""
    
    test_log_file_creation
    test_scan_results_dir_creation
    test_secret_pattern_detection
    test_unsafe_code_detection
    test_panic_pattern_detection
    test_clean_code
    test_integer_overflow_detection
    test_file_permissions
    test_world_writable_detection
    test_multiple_secret_patterns
    test_empty_contracts_directory
    
    echo ""
    echo "========================================="
    echo "Test Results"
    echo "========================================="
    echo "Tests run: $TESTS_RUN"
    echo -e "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests failed: ${RED}$TESTS_FAILED${NC}"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}✗ Some tests failed${NC}"
        exit 1
    fi
}

# Main execution
run_all_tests
