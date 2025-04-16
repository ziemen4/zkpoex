# ----------------------------------------------------------------------------
# Justfile for zkpoex - zero-knowledge proof of exploit
# -----------------------------------------------------------------------------
set dotenv-load

# RPC URLs and Bonsai API URL  
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
	@echo "\n"

# -----------------------------------------------------------------------------
# Run EVM tests
# -----------------------------------------------------------------------------
test-evm: compile-contract
	cargo test -p evm-runner -- --nocapture --test-threads=1

# -----------------------------------------------------------------------------
# Run verify test
#
# Parameters:
#  network - Network identifier ("testnet", or "mainnet")
# -----------------------------------------------------------------------------
test-verify network: compile-contract
	sh -c ' \
	  if [ -z "$WALLET_PRIV_KEY" ]; then \
	    echo "❌ WALLET_PRIV_KEY is required if you want to verify.\n"; \
	    exit 1; \
	  fi; \
	  if [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="{{HOLESKY_RPC_URL}}"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="{{MAINNET_RPC_URL}}"; \
	  else \
	    echo "⚠️ Network is unknown, also you can not use local network for this test. ETH_RPC_URL not set.\n"; \
	    exit 1; \
	  fi; \
	  echo "ETH_RPC_URL: $ETH_RPC_URL \n"; \
	  cargo test -p host -- --nocapture \
	'
# -----------------------------------------------------------------------------
# Deploy the verifier contract
#
# Parameters:
#   context_state  - Path to the context state JSON file
#   program_spec   - Path to the program specification JSON file
#   network        - Network identifier ("testnet", or "mainnet")
# -----------------------------------------------------------------------------
deploy-verifier context_state program_spec network: ascii-art compile-contract
	@echo "============================================================"
	@echo "🚀 Starting verifier deploy"
	@echo " - Context State: {{context_state}}"
	@echo " - Program Specification: {{program_spec}}"
	@echo " - Network: {{network}}"
	@echo "============================================================"

	sh -c ' \
	  if [ -z "$WALLET_PRIV_KEY" ]; then \
	    echo "❌ WALLET_PRIV_KEY is required if you want to deploy VerifierContract.\n"; \
	    exit 1; \
	  fi; \
	  if [ "{{network}}" = "testnet" ]; then \
	    export ETH_RPC_URL="{{HOLESKY_RPC_URL}}"; \
	    export VERIFIER_ADDRESS="{{VERIFIER_ADDRESS_HOLESKY}}"; \
	  elif [ "{{network}}" = "mainnet" ]; then \
	    export ETH_RPC_URL="{{MAINNET_RPC_URL}}"; \
	    export VERIFIER_ADDRESS="{{VERIFIER_ADDRESS_MAINNET}}"; \
	  else \
	    echo "⚠️ Network is unknown, also you can not use local network for this test. ETH_RPC_URL not set, no RiscZeroVerifierRouter for this network.\n"; \
	    exit 1; \
	  fi; \
	  echo "ETH_RPC_URL: $ETH_RPC_URL"; \
	  echo "VERIFIER_ADDRESS: $VERIFIER_ADDRESS \n"; \
	  cargo run --release -p sc-owner -- \
	    --private-key "$WALLET_PRIV_KEY" \
	    --risc0-verifier-contract-address $VERIFIER_ADDRESS \
	    --context-state "{{context_state}}" \
	    --program-spec "{{program_spec}}" \
	'
	@echo "============================================================"
	@echo "✅ Verifier Contract deployed successfully!"
	@echo "============================================================"
	@echo "\n"

# -----------------------------------------------------------------------------
# Unified proving recipe
#
# Parameters:
#   function       - Function signature (e.g. "withdraw(uint256)")
#   params         - Function parameters (e.g. "1001")
#   context_state  - Path to the context state JSON file
#   program_spec   - Path to the program specification JSON file
#   value          - Value to be passed to the function (in wei)
#   network        - Network identifier ("local", "testnet", or "mainnet")
#   bonsai         - (Optional) "true" for Bonsai proving, "false" for local proving.
#                    Defaults to "false" if not provided.
# -----------------------------------------------------------------------------
prove function params context_state program_spec value network bonsai="false":
	@echo "============================================================"
	@echo "🚀 Starting exploit proving"
	@echo " - Function: {{function}}"
	@echo " - Params: {{params}}"
	@echo " - Context State: {{context_state}}"
	@echo " - Program Specification: {{program_spec}}"
	@echo " - Value: {{value}} wei" 
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
	  	if [ -z "$BONSAI_API_KEY" ]; then \
			echo "❌ BONSAI_API_KEY is required for Bonsai proving but was not set.\n"; \
			exit 1; \
		fi; \
	    echo "Using Bonsai for proving \n"; \
	    export RISC0_DEV_MODE=0; \
	    export BONSAI_API_KEY="$BONSAI_API_KEY"; \
	    export BONSAI_API_URL="{{BONSAI_API_URL}}"; \
	  else \
	    echo "Using local proving \n"; \
	    export RISC0_DEV_MODE=1; \
	  fi; \
	  cargo run --release -p host -- \
	    --function "{{function}}" \
	    --params "{{params}}" \
	    --context-state "{{context_state}}" \
	    --program-spec "{{program_spec}}" \
		--value "{{value}}" \
	'
	@echo "============================================================"
	@echo "✅ Exploit verified successfully!"
	@echo "============================================================"
	@echo "\n"

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
example-basic-vulnerable-prove network bonsai="false" value="0": ascii-art compile-contract
	@echo "============================================================"
	@echo "⚙️  Just command for basic vulnerable contract "
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'value' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "exploit(bool)" "true" \
		"./shared/examples/basic-vulnerable/context_state.json" \
		"./shared/examples/basic-vulnerable/program_spec.json" \
		"{{value}}" "{{network}}" "{{bonsai}}"


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
#	value   - (Optional) Value to be passed to the function (default: "0")
# -----------------------------------------------------------------------------
example-over-under-flow-prove network bonsai="false" value="0": ascii-art compile-contract
	@echo "============================================================"
	@echo "⚙️  Just command for over-under flow contract"
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'value' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "withdraw(uint256)" "1001" \
		"./shared/examples/over-under-flow/context_state.json" \
		"./shared/examples/over-under-flow/program_spec.json" \
		"{{value}}" "{{network}}" "{{bonsai}}"

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
example-reentrancy-prove network bonsai="false": ascii-art compile-contract
	@echo "============================================================"
	@echo "⚙️  Just command for reentrancy contract"
	@echo "⚙️  $ just prove 'function' 'params' 'context_state' 'program_spec' 'value' 'network' 'bonsai'"
	@echo "============================================================"
	just prove "attack(uint256)" "1000000000000000000" \
		"./shared/examples/reentrancy/context_state.json" \
		"./shared/examples/reentrancy/program_spec.json" \
		"10000000000000000000" \
		"{{network}}" "{{bonsai}}"

help: ascii-art
	@echo "\033[37mzkpoex\033[0m is a Rust-based toolkit for generating zero-knowledge proofs of EVM exploits."
	@echo "Built on top of \033[36mrisc0\033[0m, it enables verifiable exploit attestation without leaking details."
	@echo "This CLI is powered by \033[36mjust\033[0m and provides streamlined commands for testing, proving and deploying."
	@echo ""
	@echo "\033[90mUSAGE:\033[0m"
	@echo "  just <COMMAND> [ARGS]"
	@echo ""
	@echo "\033[90mCOMMANDS:\033[0m"
	@echo "  compile-contract                  Compile all Solidity contracts"
	@echo "  test-evm                          Run tests in the evm-runner crate"
	@echo "  test-verify                       Run test for proof verification (local/testnet/mainnet)"
	@echo "  deploy-verifier                   Deploy the verifier"
	@echo "  prove                             Run the unified proving command"
	@echo "  example-basic-vulnerable-prove    Run proof for the BasicVulnerable exploit example"
	@echo "  example-over-under-flow-prove     Run proof for Over/Under Flow exploit example"
	@echo "  example-reentrancy-prove          Run proof for the Reentrancy exploit example"
	@echo ""
	@echo "\033[90mPROVING USAGE:\033[0m"
	@echo "  just prove <function> <params> <context_state> <program_spec> <value> <network> <bonsai?>"
	@echo ""
	@echo "\033[90mNETWORKS:\033[0m"
	@echo "  local, testnet (holesky), mainnet"
	@echo ""
	@echo "\033[90mVERSION:\033[0m"
	@echo "  \033[3;37mv0.1.0\033[0m"
	@echo ""
	
ascii-art:
	@echo "\033[90m============================================================\033[0m"
	@echo "\033[36m       zkpoex - zero-knowledge proof of exploit\033[0m"
	@echo "\033[90m============================================================\033[0m"
	@echo ""; sleep 0.1
	@echo "\033[36m           __                                               					\033[0m"; sleep 0.1
	@echo "\033[36m          | ∑∑                                              					\033[0m"; sleep 0.1
	@echo "\033[36m ________ | ∑∑   __   ______    ______    ______   __    __ 					\033[0m"; sleep 0.1
	@echo "\033[36m|        \\| ∑∑  /  \\ /      \\  /      \\  /      \\ |  \\  /  \\ 			\033[0m"; sleep 0.1
	@echo "\033[36m \\∑∑∑∑∑∑∑∑| ∑∑_/  ∑∑|  ∑∑∑∑∑∑\\|  ∑∑∑∑∑∑\\|  ∑∑∑∑∑∑\\ \\∑∑\\/∑∑ 			\033[0m"; sleep 0.1
	@echo "\033[36m  /    ∑∑ | ∑∑   ∑∑ | ∑∑  | ∑∑| ∑∑  | ∑∑| ∑∑    ∑∑/  /∑∑∑					\033[0m"; sleep 0.1
	@echo "\033[36m /  ∑∑∑∑_ | ∑∑∑∑∑∑\\ | ∑∑__/ ∑∑| ∑∑__/ ∑∑| ∑∑∑∑∑∑∑|  / ∑∑∑					\033[0m"; sleep 0.1
	@echo "\033[36m|  ∑∑    \\| ∑∑  \\∑∑\\| ∑∑    ∑∑ \\∑∑    ∑∑ \\∑∑     \\ /∑∑/ \\∑∑ 			\033[0m"; sleep 0.1
	@echo "\033[36m \\∑∑∑∑∑∑∑∑ \\∑∑   \\∑∑| ∑∑∑∑∑∑∑   \\∑∑∑∑∑∑   \\∑∑∑∑∑∑ \\∑∑/   \\∑∑			\033[0m"; sleep 0.1
	@echo "\033[36m                    | ∑∑                                    					\033[0m"; sleep 0.1
	@echo "\033[36m                    | ∑∑                                    					\033[0m"; sleep 0.1
	@echo "\033[36m                     \\∑∑ \033[3;37mv0.1.0\033[0m			                \033[0m"; sleep 0.1
	@echo "\n"; sleep 0.1 
	@echo "\033[3;90mDeveloped by: galexela & ziemann\033[0m            \033[3;90m Powered by Risc0			\033[0m" ; sleep 0.1
	@echo "\n"; sleep 0.1