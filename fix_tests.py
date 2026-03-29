#!/usr/bin/env python3
import re

# Read the file
with open('contracts/crowdfund/src/test.rs', 'r') as f:
    content = f.read()

# Pattern 1: Fix initialize calls with 4 parameters (old style)
# client.initialize(&creator, &token_address, &goal, &deadline)
pattern1 = r'client\.initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&deadline\s*\)'
replacement1 = r'client.initialize(&creator, &creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None)'
content = re.sub(pattern1, replacement1, content)

# Pattern 2: Fix initialize calls with 8 parameters (missing admin)
# client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None, &None, &None)
pattern2 = r'client\.initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&deadline,\s*&min_contribution,\s*&None,\s*&None,\s*&None\s*\)'
replacement2 = r'client.initialize(&creator, &creator, &token_address, &goal, &deadline, &min_contribution, &None, &None, &None)'
content = re.sub(pattern2, replacement2, content)

# Pattern 3: Fix try_initialize calls with 8 parameters
pattern3 = r'client\.try_initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&deadline,\s*&min_contribution,\s*&None,\s*&None,\s*&None\s*\)'
replacement3 = r'client.try_initialize(&creator, &creator, &token_address, &goal, &deadline, &min_contribution, &None, &None, &None)'
content = re.sub(pattern3, replacement3, content)

# Write back
with open('contracts/crowdfund/src/test.rs', 'w') as f:
    f.write(content)

print("Fixed initialize calls")
