#!/usr/bin/env python3
import re

# Read the file
with open('contracts/crowdfund/src/test.rs', 'r') as f:
    content = f.read()

# Fix multiline initialize calls with 6 parameters (auto_extension_threshold)
# This pattern matches:
# client.initialize(
#     &creator,
#     &token_address,
#     &goal,
#     &deadline,
#     &min_contribution,
#     &Some(auto_extension_threshold),
# );
pattern = r'client\.initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&deadline,\s*&min_contribution,\s*&Some\(auto_extension_threshold\),\s*\)'
replacement = r'client.initialize(&creator, &creator, &token_address, &goal, &deadline, &min_contribution, &None, &Some(auto_extension_threshold), &None)'
content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)

# Fix initialize with &None at the end (6 params)
pattern2 = r'client\.initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&deadline,\s*&min_contribution,\s*&None\s*\)'
replacement2 = r'client.initialize(&creator, &creator, &token_address, &goal, &deadline, &min_contribution, &None, &None, &None)'
content = re.sub(pattern2, replacement2, content, flags=re.MULTILINE | re.DOTALL)

# Fix initialize with deadline variable names
pattern3 = r'client\.initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&past_deadline,\s*&1_000,\s*&None,\s*&None,\s*&None\s*\)'
replacement3 = r'client.initialize(&creator, &creator, &token_address, &goal, &past_deadline, &1_000, &None, &None, &None)'
content = re.sub(pattern3, replacement3, content)

# Fix try_initialize with past_deadline
pattern4 = r'client\.try_initialize\(\s*&creator,\s*&token_address,\s*&goal,\s*&past_deadline,\s*&1_000,\s*&None,\s*&None,\s*&None\s*\)'
replacement4 = r'client.try_initialize(&creator, &creator, &token_address, &goal, &past_deadline, &1_000, &None, &None, &None)'
content = re.sub(pattern4, replacement4, content)

# Write back
with open('contracts/crowdfund/src/test.rs', 'w') as f:
    f.write(content)

print("Fixed more initialize patterns")
