.DEFAULT_GOAL := dev

.PHONY: dev build check clean install

install:
	npm install

dev: install
	npm run dev

build: install
	npm run build

check: install
	npm run check

clean:
	rm -rf dist/ src-tauri/target/ node_modules/.vite
