test-colors:
	@echo "=== Direct ANSI test ===" && printf '\x1b[32mgreen\x1b[0m\n' && echo "=== Table test ===" && ./target/release/reap port 6463 2>&1 | od -An -tx1 | grep -E '1b|3[0-4]' | head -3
