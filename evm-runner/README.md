# EVM-Runner

The EVM-Runner crate is a standalone component which serves as the program that simulates transactions in the context of the EVM (Ethereum Virtual Machine).

In order to run simulations over certain contracts, a set of fixed addresses is used to set what is known as the **context state**. This state allows the generalization of certain **templates** and **arbitrary contracts** such that when simulating a transaction, and later performing a zero knowledge proof, this simulation can be done only on certain contracts without having to change this program at all.

As such, the following addresses are reserved:
- Address `0x7A46E70000000000000000000000000000000000` is reserved for the `Target` contract, whichever it may be.
- Address `0xCA11E40000000000000000000000000000000000` is reserved for the `Caller`, which **must** be an EOA account.
- Address `0xE4C2000000000000000000000000000000000000` is reserved for the `ContextTemplateERC20` contract.
- From `0x10000000000000000000000000000000000000aa` to `0x10000000000000000000000000000000000000ff`:
  - Here we define any arbitrary contract that the prover may want to deploy to the global state. This gives room to simulate with plenty of contracts
