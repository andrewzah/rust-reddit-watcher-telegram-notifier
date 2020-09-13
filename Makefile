IMAGE_NAME := andrewzah/rust-reddit-watcher-telegram-notifier

.phony: build

build:
	cargo build

release:
	cargo build --release

make clean:
	cargo clean

make clippy:
	cargo clippy

make docker: docker-build docker-push

make docker-build: clippy release
	docker build . -t "${IMAGE_NAME}:latest"

make docker-push:
	docker push "${IMAGE_NAME}"
