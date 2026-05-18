PREFIX ?= /usr/local
LOCAL_PREFIX ?= $(HOME)/.local
BINARY := target/release/rusticle

# ── IMPORTANT ───────────────────────────────────────────
# NEVER use `cargo install`. It puts binaries in ~/.cargo/bin
# which pollutes PATH precedence. Use `make install-local`
# for user install or `make install` for system-wide.
# ─────────────────────────────────────────────────────────

.PHONY: all build release test install install-local uninstall uninstall-local clean demo

all: build

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --no-fail-fast

$(BINARY): release

install: $(BINARY)
	install -d $(PREFIX)/bin
	install -m 755 $(BINARY) $(PREFIX)/bin/rusticle
	install -d $(PREFIX)/share/rusticle/examples
	install -m 644 examples/*.tcl $(PREFIX)/share/rusticle/examples/
	@echo "  ✅ rusticle → $(PREFIX)/bin/rusticle"

install-local: $(BINARY)
	install -d $(LOCAL_PREFIX)/bin
	install -m 755 $(BINARY) $(LOCAL_PREFIX)/bin/rusticle
	install -d $(LOCAL_PREFIX)/share/rusticle/examples
	install -m 644 examples/*.tcl $(LOCAL_PREFIX)/share/rusticle/examples/
	@echo "  ✅ rusticle → $(LOCAL_PREFIX)/bin/rusticle"

uninstall:
	rm -f $(PREFIX)/bin/rusticle
	rm -rf $(PREFIX)/share/rusticle

uninstall-local:
	rm -f $(LOCAL_PREFIX)/bin/rusticle
	rm -rf $(LOCAL_PREFIX)/share/rusticle

clean:
	cargo clean

demo:
	@for f in examples/*.tcl; do \
		echo "\n══════ $$f ══════"; \
		cargo run -- "$$f" || true; \
	done
