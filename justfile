compile-contract:
    for file in contracts/src/*.sol; do \
        solc --bin-runtime \
            --optimize \
            --overwrite \
            --evm-version istanbul \
            --output-dir bytecode \
            $file; \
        done

    solc --storage-layout --optimize --overwrite --evm-version istanbul contracts/src/ContextTemplateERC20.sol > contracts/out/ContextTemplateERC20_layout.json

test-evm: compile-contract
    cargo test -p evm-runner -- --nocapture

prove function params conditions contract_bytecode: compile-contract
    @echo "Running with function={{function}}, params={{params}}, conditions={{conditions}}, contract_bytecode={{contract_bytecode}}"
    RISC0_DEV_MODE=true RUST_LOG=full RUST_BACKTRACE=1 \
    cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --conditions "{{conditions}}" \
        --contract-bytecode "{{contract_bytecode}}"

prove-bonsai function params conditions contract_bytecode: compile-contract
    @echo "Running with function={{function}}, params={{params}}, conditions={{conditions}}, contract_bytecode={{contract_bytecode}}"
    RISC0_DEV_MODE=false RUST_LOG=full RUST_BACKTRACE=1 \
    BONSAI_API_KEY=J8ZXydQGyGMWvK8BVXa92Juxi0u2eZl8MpH0v632 BONSAI_API_URL=https://api.bonsai.xyz/ \
    cargo run --release -p host -- \
        --function "{{function}}" \
        --params "{{params}}" \
        --conditions "{{conditions}}" \
        --contract-bytecode "{{contract_bytecode}}"

