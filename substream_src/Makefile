DSN ?= clickhouse://default:@127.0.0.1:9000/virtual_agents_on_solana
ENDPOINT ?= mainnet.sol.streamingfast.io:443

.PHONY: protogen
protogen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: run
run: build
	substreams run substreams.yaml $(if $(MODULE),$(MODULE),map_filter_transactions) $(if $(START_BLOCK),-s $(START_BLOCK)) $(if $(STOP_BLOCK),-t $(STOP_BLOCK)) $(if $(PARAMS),-p '$(PARAMS)')

.PHONY: package
package: build
	substreams pack substreams.yaml

.PHONY: stream
stream: build
	substreams run -e mainnet.sol.streamingfast.io:443 substreams.yaml map_block_without_votes -s 321751296 -t +1 --debug-modules-output map_block_without_votes

.PHONY: stream_db
stream_db: build
	substreams run -e mainnet.sol.streamingfast.io:443 substreams.yaml db_out -s 321100949

.PHONY: create_db
create_db:
	substreams-sink-sql setup "clickhouse://default:@127.0.0.1:9000/virtual_agents_on_solana" manifest/sink_solana.yaml

.PHONY: sink_db_out
sink_db_out:
	substreams-sink-sql run -e mainnet.sol.streamingfast.io:443 --flush-interval 1 "clickhouse://default:@127.0.0.1:9000/solana" manifest/sink_solana.yaml 319985132:333124600 --undo-buffer-size 10"	

