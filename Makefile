.DEFAULT_GOAL := dev

.PHONY: dev build check test smoke clean install ci

install:
	npm install

dev: install
	npm run dev

build: install
	npm run build

check: install
	npm run check

test: install
	npm run test

smoke: install
	npm run smoke

# Run all checks an agent should run before pushing
ci: check test smoke build

clean:
	rm -rf dist/ src-tauri/target/ node_modules/.vite
