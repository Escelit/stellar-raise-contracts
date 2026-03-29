#!/usr/bin/env python3
import re

# Read the file
with open('contracts/crowdfund/src/test.rs', 'r') as f:
    content = f.read()

# Fix contribute calls with 3 parameters - remove the third parameter (&None)
# Pattern: contribute(&contributor, &amount, &None)
content = re.sub(r'\.contribute\((&[^,]+), (&[^,]+), &None\)', r'.contribute(\1, \2)', content)

# Fix try_contribute calls with 3 parameters
content = re.sub(r'\.try_contribute\((&[^,]+), (&[^,]+), &None\)', r'.try_contribute(\1, \2)', content)

# Write back
with open('contracts/crowdfund/src/test.rs', 'w') as f:
    f.write(content)

print("Fixed contribute calls")
