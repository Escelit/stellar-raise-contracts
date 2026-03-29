# TODO: Implement refund_single token transfer security/logging improvements (closes #320)

## Steps (in order):

- [x] 2. Edit `contracts/crowdfund/src/refund_single_token.rs`: Add amount > 0 check, debug event, NatSpec.
- [x] 3. Edit `contracts/crowdfund/src/lib.rs`: Remove duplicate transfer, update callers/NatSpec.
- [x] 4. Created `contracts/crowdfund/src/refund_single_token_security_tests.rs`: zero-skip + debug event tests.
- [x] 5. Edit `contracts/crowdfund/refund_single_token.md`: Add zero-opt + logging sections.
- [ ] 6. Run `cargo test -p crowdfund refund_single -- --nocapture`
- [ ] 7. Commit: `git commit -m "feat: add logging/bounds to refund_single token transfer logic for security with tests/docs (closes #320)"`
- [ ] 8. Create PR: `gh pr create --title "feat: ... (closes #320)" --body "..."`

**Current: Step 1 pending**

# TODO: Extract constants from WASM build pipeline caching for Testing (#314)

## Steps:


- [ ] 2. Clean git state (reset HEAD, add ., commit)
- [ ] 3. Checkout new branch feature/extract-constants-from-wasm-build-pipeline-caching-for-testing
- [ ] 4. Create wasm_build_pipeline.tsx with constants
- [ ] 5. Create wasm_build_pipeline.test.tsx with tests
- [ ] 6. Create wasm_build_pipeline.md docs
- [ ] 7. Run tests (npm test, cargo test, coverage >95%)
- [ ] 8. Commit changes
- [ ] 9. Push branch
- [ ] 10. Create PR to upstream main
