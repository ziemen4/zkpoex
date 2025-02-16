compile-contract:
    solc --bin-runtime \
        --optimize \
        --overwrite \
        --evm-version istanbul \
        --output-dir bytecode \
        contracts/src/TargetContract.sol

    solc --storage-layout --optimize --overwrite --evm-version istanbul contracts/src/ContextERC20.sol > contracts/out/ContextERC20_layout.json

    solc --bin-runtime \
        --optimize \
        --overwrite \
        --evm-version istanbul \
        --output-dir bytecode \
        contracts/src/ContextERC20.sol

test-evm: compile-contract
    cargo test -p evm-runner -- --nocapture

prove: compile-contract
    RUST_BACKTRACE=full cargo run --release -p host

