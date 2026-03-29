#!/usr/bin/env python3
import re

# Read the file
with open('contracts/crowdfund/src/test.rs', 'r') as f:
    lines = f.readlines()

# Track seen test functions
seen_tests = set()
output_lines = []
skip_until = -1
i = 0

while i < len(lines):
    line = lines[i]
    
    # Check if this is a test function definition
    if i > skip_until:
        match = re.match(r'^fn (test_\w+)\(\)', line)
        if match:
            test_name = match.group(1)
            if test_name in seen_tests:
                # This is a duplicate, skip until we find the next function or test
                print(f"Removing duplicate: {test_name} at line {i+1}")
                # Find the end of this function (next 'fn ' or end of file)
                j = i + 1
                brace_count = 0
                found_opening = False
                while j < len(lines):
                    if '{' in lines[j]:
                        brace_count += lines[j].count('{')
                        found_opening = True
                    if '}' in lines[j]:
                        brace_count -= lines[j].count('}')
                    if found_opening and brace_count == 0:
                        skip_until = j
                        break
                    j += 1
                i += 1
                continue
            else:
                seen_tests.add(test_name)
    
    output_lines.append(line)
    i += 1

# Write back
with open('contracts/crowdfund/src/test.rs', 'w') as f:
    f.writelines(output_lines)

print(f"Kept {len(seen_tests)} unique tests")
