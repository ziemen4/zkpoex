# Global Bonsai settings (update the API key as needed)
set dotenv-load
BONSAI_API_KEY := env("BONSAI_API_KEY")
BONSAI_API_URL := "https://api.bonsai.xyz/"

# -----------------------------------------------------------------------------
# Compile Solidity Contracts
# -----------------------------------------------------------------------------
compile-contract:
	@echo "============================================================"
	@echo "🚀 Starting compilation..."
	@echo "============================================================"
	solc-select install 0.8.20 && \
	solc-select use 0.8.20 && \
	for file in contracts/src/examples/*.sol; do \
		solc --abi --bin --bin-runtime --optimize --overwrite --evm-version shanghai --output-dir bytecode $$file; \
	done

	for file in contracts/src/*.sol; do \
		solc --abi --bin --bin-runtime --optimize --overwrite --evm-version shanghai --output-dir bytecode $$file; \
	done

	solc --storage-layout --optimize --overwrite --evm-version shanghai contracts/src/context/ContextTemplateERC20.sol > contracts/out/ContextTemplateERC20_layout.json

	@echo "============================================================"
	@echo "✅ Compilation completed successfully!"
	@echo "============================================================"

# -----------------------------------------------------------------------------
# Run EVM tests
# -----------------------------------------------------------------------------
test-evm: compile-contract
	cargo test -p evm-runner -- --nocapture

# -----------------------------------------------------------------------------
# Deploy the verifier contract (network parameter required)
# -----------------------------------------------------------------------------
deploy-verifier network: compile-contract
	sh -c ' \
	  if [ "{{network}}" = "local" ]; then \
	    export ETH_RPC_URL="http://localhost:8545"; \
	  elif [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="https://ethereum-rpc.publicnode.com"; \
	  else \
	    echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
	    export ETH_RPC_URL=""; \
	  fi; \
	  echo "ETH_RPC_URL: $$ETH_RPC_URL"; \
	  cargo run --release -p sc-owner -- --no-capture   \
	'

# -----------------------------------------------------------------------------
# Unified proving recipe
#
# Parameters:
#   function       - Function signature (e.g. "withdraw(uint256)")
#   params         - Function parameters (e.g. "1001")
#   context_state  - Path to the context state JSON file
#   program_spec   - Path to the program specification JSON file
#   network        - Network identifier ("local", "testnet", or "mainnet")
#   bonsai         - (Optional) "true" for Bonsai proving, "false" for local proving.
#                    Defaults to "false" if not provided.
# -----------------------------------------------------------------------------
prove function params context_state program_spec network bonsai="false": compile-contract
	@echo "============================================================"
	@echo "🚀 Starting exploit proving"
	@echo " - Function: {{function}}"
	@echo " - Params: {{params}}"
	@echo " - Context State: {{context_state}}"
	@echo " - Program Specification: {{program_spec}}"
	@echo " - Network: {{network}}"
	@echo " - Bonsai: {{bonsai}}"
	@echo "============================================================"
	sh -c ' \
	  if [ "{{network}}" = "local" ]; then \
	    export ETH_RPC_URL="http://localhost:8545"; \
	  elif [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="https://ethereum-rpc.publicnode.com"; \
	  else \
	    echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
	    export ETH_RPC_URL=""; \
	  fi; \
	  echo "ETH_RPC_URL: $$ETH_RPC_URL"; \
	  if [ "{{bonsai}}" = "true" ]; then \
	    echo "Using Bonsai for proving"; \
	    export RISC0_DEV_MODE=0; \
	    export BONSAI_API_KEY="{{BONSAI_API_KEY}}"; \
	    export BONSAI_API_URL="{{BONSAI_API_URL}}"; \
	  else \
	    echo "Using local proving"; \
	    export RISC0_DEV_MODE=1; \
	  fi; \
	  cargo run --release -p host -- \
	    --function "{{function}}" \
	    --params "{{params}}" \
	    --context-state "{{context_state}}" \
	    --program-spec "{{program_spec}}" \
	'
	@echo "============================================================"
	@echo "✅ Exploit verified successfully!"
	@echo "============================================================"

# -----------------------------------------------------------------------------
# Example: Over-Under Flow Proving
#
# This recipe wraps the unified prove command with hardcoded parameters for the
# "withdraw(uint256)" function, context state, and program specification.
#
# Parameters:
#   network  - Network identifier ("local", "testnet", or "mainnet")
#   bonsai   - (Optional) "true" for Bonsai proving, "false" for local proving.
#               Defaults to "false" if not provided.
# -----------------------------------------------------------------------------
example-over-under-flow-prove network bonsai="false": compile-contract
	@echo "============================================================"
	@echo "🚀 Starting exploit proving for over-under flow"
	@echo " - Function: withdraw(uint256)"
	@echo " - Params: 1001"
	@echo " - Context State: ./shared/examples/over-under-flow/context_state.json"
	@echo " - Program Specification: ./shared/examples/over-under-flow/program_spec.json"
	@echo " - Network: {{network}}"
	@echo " - Bonsai: {{bonsai}}"
	@echo "============================================================"
	just prove "withdraw(uint256)" "1001" "./shared/examples/over-under-flow/context_state.json" "./shared/examples/over-under-flow/program_spec.json" "{{network}}" "{{bonsai}}"
