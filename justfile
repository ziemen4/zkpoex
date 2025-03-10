compile-contract:
    solc-select use 0.8.20 && \
    for file in contracts/src/examples/*.sol; do \
        solc --bin \
             --optimize \
             --overwrite \
             --evm-version cancun \
             --output-dir bytecode \
             $file; \
    done

    for file in contracts/src/examples/*.sol; do \
        solc --abi \
             --bin \
             --optimize \
             --overwrite \
             --evm-version cancun \
             --output-dir bytecode \
             $file; \
    done


    for file in contracts/src/*.sol; do \
        solc --bin \
             --optimize \
             --overwrite \
             --evm-version cancun \
             --output-dir bytecode \
             $file; \
    done

    solc --storage-layout \
         --optimize \
         --overwrite \
         --evm-version cancun \
         contracts/src/ContextTemplateERC20.sol > contracts/out/ContextTemplateERC20_layout.json


test-evm: compile-contract
    cargo test -p evm-runner -- --nocapture

prove function params conditions contract_bytecode network abi: compile-contract
    @echo "Running with function={{function}}, params={{params}}, conditions={{conditions}}, contract_bytecode={{contract_bytecode}}, network={{network}}, abi={{abi}}"
    RISC0_DEV_MODE=true RUST_LOG=full RUST_BACKTRACE=1 \
    cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --conditions "{{conditions}}" \
        --contract-bytecode "{{contract_bytecode}}" \
        --network "{{network}}" \
        --abi "{{abi}}"

prove-bonsai function params conditions contract_bytecode network abi: compile-contract
    @echo "Running with function={{function}}, params={{params}}, conditions={{conditions}}, contract_bytecode={{contract_bytecode}}, network={{network}}, abi={{abi}}"
    RISC0_DEV_MODE=false RUST_LOG=full RUST_BACKTRACE=1 \
    BONSAI_API_KEY=J8ZXydQGyGMWvK8BVXa92Juxi0u2eZl8MpH0v632 BONSAI_API_URL=https://api.bonsai.xyz/ \
    
    # Set the ETH_RPC_URL based on the network
    if [ "{{network}}" = "testnet" ]; then \
        export ETH_RPC_URL=https://ethereum-holesky-rpc.publicnode.com; \
    elif [ "{{network}}" = "mainnet" ]; then \
        export ETH_RPC_URL=https://ethereum-rpc.publicnode.com; \
    fi

    cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --conditions "{{conditions}}" \
        --contract-bytecode "{{contract_bytecode}}" \
        --network "{{network}}" \
        --abi "{{abi}}"

