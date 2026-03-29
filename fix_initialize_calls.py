#!/usr/bin/env python3
import re

# Read the file
with open('contracts/crowdfund/src/test.rs', 'r') as f:
    content = f.read()

# Pattern 1: Single-line initialize calls with 8 args
# client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None)
pattern1 = r'client\.initialize\(&creator, &token_address, &goal, &deadline, &(\d+|min_contribution), &None, &None, &None\)'
replacement1 = r'client.initialize(&admin, &creator, &token_address, &goal, &deadline, &\1, &None, &None, &None, &None)'
content = re.sub(pattern1, replacement1, content)

# Pattern 2: Multi-line initialize calls starting with client.initialize(
# Need to handle these more carefully
lines = content.split('\n')
new_lines = []
i = 0
while i < len(lines):
    line = lines[i]
    
    # Check if this is a multi-line initialize call
    if 'client.initialize(' in line or 'client.try_initialize(' in line:
        # Collect all lines until we find the closing )
        call_lines = [line]
        j = i + 1
        paren_count = line.count('(') - line.count(')')
        
        while j < len(lines) and paren_count > 0:
            call_lines.append(lines[j])
            paren_count += lines[j].count('(') - lines[j].count(')')
            j += 1
        
        # Join the call lines
        full_call = '\n'.join(call_lines)
        
        # Check if it starts with &creator (old format)
        if re.search(r'initialize\(\s*&creator,', full_call):
            # Insert &admin, before &creator
            full_call = re.sub(r'(initialize\()\s*(&creator,)', r'\1\n        &admin,\n        \2', full_call)
        
        # Split back into lines and add to new_lines
        new_lines.extend(full_call.split('\n'))
        i = j
    else:
        new_lines.append(line)
        i += 1

content = '\n'.join(new_lines)

# Write back
with open('contracts/crowdfund/src/test.rs', 'w') as f:
    f.write(content)

print("Fixed initialize calls")
