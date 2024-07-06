DOCKER_BUILD_STATE_FILE := .wg-remote-docker-build

target/release/wg:
	cargo build --release

wg-remote-docker: $(DOCKER_BUILD_STATE_FILE)

$(DOCKER_BUILD_STATE_FILE): Dockerfile scripts/run_server.sh target/release/wg
	docker build -t wg-remote:latest .
	@touch $(DOCKER_BUILD_STATE_FILE)

.PHONY: wg-remote-docker
