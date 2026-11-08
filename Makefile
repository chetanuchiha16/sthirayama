.PHONY: all help build clean test \
        engine1 engine2 engine3 \
        raft1 raft2 raft3 \
        clean-data

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build & Test:"
	@echo "  make build          - Build both Rust engine and Go Raft binaries"
	@echo "  make test           - Run Rust and Go tests"
	@echo "  make test-engine    - Run Rust engine tests"
	@echo "  make test-raft      - Run Go Raft tests"
	@echo ""
	@echo "Run Rust Engines:"
	@echo "  make engine1        - Run Rust engine 1 on 127.0.0.1:50051 (./db1)"
	@echo "  make engine2        - Run Rust engine 2 on 127.0.0.1:50052 (./db2)"
	@echo "  make engine3        - Run Rust engine 3 on 127.0.0.1:50053 (./db3)"
	@echo ""
	@echo "Run Raft Nodes:"
	@echo "  make raft1          - Run Raft Node 1 (Bootstrap leader) on HTTP:8001, Raft:7001"
	@echo "  make raft2          - Run Raft Node 2 on HTTP:8002, Raft:7002 (joins Node 1)"
	@echo "  make raft3          - Run Raft Node 3 on HTTP:8003, Raft:7003 (joins Node 1)"
	@echo ""
	@echo "Clean:"
	@echo "  make clean-data     - Wipe data directories (data-*, db*)"
	@echo "  make clean          - Clean build artifacts and data directories"

# ----------------------------------------------------
# Build & Test
# ----------------------------------------------------
build: build-engine build-raft

build-engine:
	cargo build --manifest-path engine/Cargo.toml

build-raft:
	cd raft && go build -o bin/sthirayama-raft cmd/main.go

test: test-engine test-raft

test-engine:
	cargo test --manifest-path engine/Cargo.toml

test-raft:
	cd raft && go test -v ./...

# ----------------------------------------------------
# Rust Engines
# ----------------------------------------------------
engine1:
	cargo run --manifest-path engine/Cargo.toml -- ./db1 50051

engine2:
	cargo run --manifest-path engine/Cargo.toml -- ./db2 50052

engine3:
	cargo run --manifest-path engine/Cargo.toml -- ./db3 50053

# ----------------------------------------------------
# Go Raft Nodes
# ----------------------------------------------------
raft1:
	cd raft && go run cmd/main.go \
		--id=node1 \
		--engine=127.0.0.1:50051 \
		--raft=127.0.0.1:7001 \
		--http=127.0.0.1:8001 \
		--data=./data-1 \
		--bootstrap=true

raft2:
	cd raft && go run cmd/main.go \
		--id=node2 \
		--engine=127.0.0.1:50052 \
		--raft=127.0.0.1:7002 \
		--http=127.0.0.1:8002 \
		--data=./data-2 \
		--join=127.0.0.1:8001

raft3:
	cd raft && go run cmd/main.go \
		--id=node3 \
		--engine=127.0.0.1:50053 \
		--raft=127.0.0.1:7003 \
		--http=127.0.0.1:8003 \
		--data=./data-3 \
		--join=127.0.0.1:8001

# ----------------------------------------------------
# Cleanup
# ----------------------------------------------------
clean-data:
	rm -rf raft/data-* db1 db2 db3 ./data-* ./main

clean: clean-data
	cargo clean --manifest-path engine/Cargo.toml
	rm -rf raft/bin
