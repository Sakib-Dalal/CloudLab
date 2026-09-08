.PHONY: build dev test images desktop install
build:
	./build.sh
dev:
	npm run dev
test:
	npm run check
	cargo test --locked -p cloudlab
	cargo build --locked -p cloudlab
	python3 tests/integration.py
images:
	./scripts/build-images.sh
desktop:
	npm run desktop:build
install:
	./install.sh
