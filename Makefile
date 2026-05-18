LOCAL_PREFIX ?= $(HOME)/.local
DEMO_DIR ?= $(LOCAL_PREFIX)/share/rusticle/examples
BINARY := target/release/rusticle

# ── IMPORTANT ───────────────────────────────────────────
# NEVER use `cargo install`. It puts binaries in ~/.cargo/bin
# which pollutes PATH precedence. Use `make install` only.
# All binaries go to ~/.local/bin.
# ─────────────────────────────────────────────────────────

.PHONY: all build release test install install-demos uninstall clean demo

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --no-fail-fast

$(BINARY): release

install: $(BINARY) install-demos
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 $(BINARY) $(LOCAL_PREFIX)/bin/rusticle
	@echo "  ✅ rusticle → $(LOCAL_PREFIX)/bin/rusticle"

install-demos:
	install -d $(DEMO_DIR)
	install -m 644 examples/*.tcl $(DEMO_DIR)/
	@echo "  ✅ demos → $(DEMO_DIR)/"

uninstall:
	rm -f $(LOCAL_PREFIX)/bin/rusticle
	rm -rf $(LOCAL_PREFIX)/share/rusticle
	@echo "  ✅ uninstalled"

clean:
	cargo clean

demo:
	@for f in examples/*.tcl; do \
		echo "\n══════ $$f ══════"; \
		cargo run -- "$$f" || true; \
	done
