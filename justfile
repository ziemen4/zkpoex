# Global Bonsai settings (update the API key as needed)
set dotenv-load

BONSAI_API_KEY := env("BONSAI_API_KEY")
WALLET_PRIV_KEY := env("WALLET_PRIV_KEY")
ANVIL_RPC_URL := "http://localhost:8545"
SEPOLIA_RPC_URL := "https://ethereum-sepolia-rpc.publicnode.com"
HOLESKY_RPC_URL := "https://ethereum-holesky-rpc.publicnode.com"
MAINNET_RPC_URL := "https://ethereum-rpc.publicnode.com"
BONSAI_API_URL := "https://api.bonsai.xyz/"

# Risc0 Verifier Contract Address from https://dev.risczero.com/api/blockchain-integration/contracts/verifier (RiscZeroVerifierRouter)
VERIFIER_ADDRESS_MAINNET := "0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319"
VERIFIER_ADDRESS_HOLESKY := "0xf70aBAb028Eb6F4100A24B203E113D94E87DE93C"
VERIFIER_ADDRESS_SEPOLIA := "0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187"

# -----------------------------------------------------------------------------
# Compile Solidity Contracts
# -----------------------------------------------------------------------------
compile-contract:
	@echo "============================================================"
	@echo "🚀 Starting compilation..."
	@echo "============================================================"
	solc-select install 0.8.20 && \
	solc-select use 0.8.20 && \
	find contracts/src/ -type f -name '*.sol' -exec solc --abi \
		--bin \
		--bin-runtime \
		--optimize \
		--overwrite \
		--evm-version shanghai \
		--output-dir bytecode \
		{} \;

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
# Run sc-owner tests
# -----------------------------------------------------------------------------
test-sc-owner network: compile-contract
	sh -c ' \
	  if [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="{{HOLESKY_RPC_URL}}"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="{{MAINNET_RPC_URL}}"; \
	  else \
	    echo "⚠️  Network is unknown, ETH_RPC_URL not set"; \
	    export ETH_RPC_URL=""; \
	  fi; \
	  echo "ETH_RPC_URL: $ETH_RPC_URL"; \
	  cargo test -p sc-owner -- --nocapture \
	'
# -----------------------------------------------------------------------------
# Deploy the verifier contract (network parameter required)
# -----------------------------------------------------------------------------
deploy-verifier context_state program_spec network: compile-contract
	@echo "============================================================"
	@echo "🚀 Starting verifier deploy"
	@echo " - Context State: {{context_state}}"
	@echo " - Program Specification: {{program_spec}}"
	@echo " - Network: {{network}}"
	@echo "============================================================"

	sh -c ' \
	  if [ "{{network}}" = "local" ]; then \
	    export ETH_RPC_URL="{{ANVIL_RPC_URL}}"; \
	    export VERIFIER_ADDRESS=""; \
	  elif [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="{{HOLESKY_RPC_URL}}"; \
	    export VERIFIER_ADDRESS="{{VERIFIER_ADDRESS_HOLESKY}}"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="{{MAINNET_RPC_URL}}"; \
	    export VERIFIER_ADDRESS="{{VERIFIER_ADDRESS_MAINNET}}"; \
	  else \
	    echo "⚠️  Network is unknown, ETH_RPC_URL not set"; \
	    export ETH_RPC_URL=""; \
	    export VERIFIER_ADDRESS=""; \
	  fi; \
	  echo "ETH_RPC_URL: $ETH_RPC_URL"; \
	  echo "VERIFIER_ADDRESS: $VERIFIER_ADDRESS"; \
	  cargo run --release -p sc-owner -- \
	    --private-key "{{WALLET_PRIV_KEY}}" \
	    --risc0-verifier-contract-address $VERIFIER_ADDRESS \
	    --context-state "{{context_state}}" \
	    --program-spec "{{program_spec}}" \
	'
	@echo "============================================================"
	@echo "✅ Verifier Contract deployed successfully!"
	@echo "============================================================"

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
prove function params context_state program_spec network bonsai="false":
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
	    export ETH_RPC_URL="{{ANVIL_RPC_URL}}"; \
	  elif [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="{{HOLESKY_RPC_URL}}"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="{{MAINNET_RPC_URL}}"; \
	  else \
	    echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
	    export ETH_RPC_URL=""; \
	  fi; \
	  echo "ETH_RPC_URL: $ETH_RPC_URL"; \
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
# Example: Basic Vulnerable Proving
#
# This recipe wraps the unified prove command with hardcoded parameters for the
# "exploit(bool)" function, context state, and program specification.
#
# Parameters:
#   network  - Network identifier ("local", "testnet", or "mainnet")
#   bonsai   - (Optional) "true" for Bonsai proving, "false" for local proving.
#               Defaults to "false" if not provided.
# -----------------------------------------------------------------------------
example-basic-vulnerable-prove network bonsai="false": compile-contract
	@echo "============================================================"
	@echo "⚙️  Just command for basic vulnerable contract "
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "exploit(bool)" "true" \
		"./shared/examples/basic-vulnerable/context_state.json" \
		"./shared/examples/basic-vulnerable/program_spec.json" \
		"{{network}}" "{{bonsai}}"


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
	@echo "⚙️  Just command for over-under flow contract"
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "withdraw(uint256)" "1001" \
		"./shared/examples/over-under-flow/context_state.json" \
		"./shared/examples/over-under-flow/program_spec.json" \
		"{{network}}" "{{bonsai}}"

# -----------------------------------------------------------------------------
# Example: Reentrancy Proving
#
# This recipe wraps the unified prove command with hardcoded parameters for the
# "attack(uint256)" function, context state, and program specification.
#
# Parameters:
#   network  - Network identifier ("local", "testnet", or "mainnet")
#   bonsai   - (Optional) "true" for Bonsai proving, "false" for local proving.
#               Defaults to "false" if not provided.
# -----------------------------------------------------------------------------
example-reentrancy-prove network bonsai="false": compile-contract
	@echo "============================================================"
	@echo "⚙️  Just command for reentrancy contract"
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "attack(uint256)" "10000000000000000000" \
		"./shared/examples/reentrancy/context_state.json" \
		"./shared/examples/reentrancy/program_spec.json" \
		"{{network}}" "{{bonsai}}"
