default: check lint fmt test

@check:
        echo "[+] Checking"
        cargo check

@lint:
        echo "[+] Linting"
        cargo clippy --workspace --all-targets --all-features -- -Dwarnings

@fmt:
        echo "[+] Formatting"
        cargo fmt --all --check

@test:
        echo "[+] Testing"
        cargo nextest r --no-tests pass

@commit-msg:
        sh ./just-scripts/commit-msg
