compile-contract:
    @echo "============================================================"
    @echo "🚀 Starting compilation..."
    @echo "============================================================"
    solc-select install 0.8.20 && \
    solc-select use 0.8.20 && \
    for file in contracts/src/examples/*.sol; do \
        solc --abi \
             --bin \
             --bin-runtime \
             --optimize \
             --overwrite \
             --evm-version shanghai \
             --output-dir bytecode \
             $file; \
    done

    for file in contracts/src/*.sol; do \
        solc --abi \
             --bin \
             --bin-runtime \
             --optimize \
             --overwrite \
             --evm-version shanghai \
             --output-dir bytecode \
             $file; \
    done

    solc --storage-layout \
         --optimize \
         --overwrite \
         --evm-version shanghai \
         contracts/src/context/ContextTemplateERC20.sol > contracts/out/ContextTemplateERC20_layout.json

    @echo "============================================================"
    @echo "✅ Compilation completed successfully!"
    @echo "============================================================"


test-evm: compile-contract
    cargo test -p evm-runner -- --nocapture

deploy-verifier network: compile-contract
    sh -c ' \
      if [ "{{network}}" = "testnet" ]; then \
        export ETH_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"; \
        echo "test ETH_RPC_URL: $ETH_RPC_URL"; \
      elif [ "{{network}}" = "mainnet" ]; then \
        export ETH_RPC_URL="https://ethereum-rpc.publicnode.com"; \
        echo "main ETH_RPC_URL: $ETH_RPC_URL"; \
      else \
        echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
        export ETH_RPC_URL=""; \
      fi; \
      echo "ETH_RPC_URL: $ETH_RPC_URL"; \
      cargo run --release -p sc-owner -- --no-capture   \
    '

prove function params context_state program_spec network: compile-contract
    @echo "============================================================"
    @echo "🚀 Starting exploit remote proving using Bonsai:"
    @echo "  - Function: {{function}}"
    @echo "  - Params: {{params}}"
    @echo "  - Context State: {{context_state}}"
    @echo "  - Program Specification: {{program_spec}}"
    @echo "  - Network: {{network}}"
    @echo "============================================================"
    
    RISC0_DEV_MODE=1 RUST_LOG=full RISC0_INFO=1 RUST_BACKTRACE=1 \

    # Set up the ETH_RPC_URL based on the network and run the cargo command
    sh -c ' \
      if [ "{{network}}" = "local" ]; then \
        export ETH_RPC_URL="http://localhost:8545"; \
        echo "local ETH_RPC_URL: $ETH_RPC_URL"; \
      elif [ "{{network}}" = "testnet" ]; then \
        export ETH_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"; \
        echo "test ETH_RPC_URL: $ETH_RPC_URL"; \
      elif [ "{{network}}" = "mainnet" ]; then \
        export ETH_RPC_URL="https://ethereum-rpc.publicnode.com"; \
        echo "main ETH_RPC_URL: $ETH_RPC_URL"; \
      else \
        echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
        export ETH_RPC_URL=""; \
      fi; \
      echo "ETH_RPC_URL: $ETH_RPC_URL"; \
      cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --context-state "{{context_state}}" \
        --program-spec "{{program_spec}}" \
    '

    @echo "============================================================"
    @echo "✅ Exploit verified successfully!"
    @echo "============================================================"

prove-bonsai function params context_state program_spec network: compile-contract
    @echo "============================================================"
    @echo "🚀 Starting exploit remote proving using Bonsai:"
    @echo "  - Function: {{function}}"
    @echo "  - Params: {{params}}"
    @echo "  - Context State: {{context_state}}"
    @echo "  - Program Specification: {{program_spec}}"
    @echo "  - Network: {{network}}"
    @echo "============================================================"

    RISC0_DEV_MODE=0 RUST_LOG=full RUST_BACKTRACE=1 \
    BONSAI_API_KEY=J8ZXydQGyGMWvK8BVXa92Juxi0u2eZl8MpH0v632 BONSAI_API_URL=https://api.bonsai.xyz/ \
    
    # Set up the ETH_RPC_URL based on the network and run the cargo command
    sh -c ' \
      if [ "{{network}}" = "local" ]; then \
        export ETH_RPC_URL="http://localhost:8545"; \
        echo "local ETH_RPC_URL: $ETH_RPC_URL"; \
      elif [ "{{network}}" = "testnet" ]; then \
        export ETH_RPC_URL="https://ethereum-holesky-rpc.publicnode.com"; \
        echo "test ETH_RPC_URL: $ETH_RPC_URL"; \
      elif [ "{{network}}" = "mainnet" ]; then \
        export ETH_RPC_URL="https://ethereum-rpc.publicnode.com"; \
        echo "main ETH_RPC_URL: $ETH_RPC_URL"; \
      else \
        echo "⚠️ Network is unknown, ETH_RPC_URL not set"; \
        export ETH_RPC_URL=""; \
      fi; \
      echo "ETH_RPC_URL: $ETH_RPC_URL"; \
      RUST_BACKTRACE=1 cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --context-state "{{context_state}}" \
        --program-spec "{{program_spec}}" \
    '

    @echo "============================================================"
    @echo "✅ Exploit verified successfully!"
    @echo "============================================================"

