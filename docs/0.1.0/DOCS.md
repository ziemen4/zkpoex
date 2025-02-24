## Introduction
One of the biggest issues with crypto nowadays is that exploits happen very often and most of them steal thousands or millions of dollars.

Software is not exempt of *bugs* and since development in the crypto industry tends to be fast paced with huge risks, it usually means that there are interested agents looking for *bugs* and exploits to obtain profit. Hence, there is a big demand for novel use cases that can somehow mitigate these issues.

*Bounties* typically carry some friction, since sometimes there is some ambiguity over found exploits, sometimes *bounty hunters* are not paid at all.
> "[...] it’s important that hunters receive their rewards promptly and reliably. Delays in payments or a lack of trust in the reward process can be problematic. Additionally, issues related to financial privacy, such as sharing bank card details, need to be addressed."  
> _[Source: "A Comprehensive Survey of Bug Bounties and Vulnerability Reward Programs"](https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=10409942)

*Audits* in the other hand can find potential bugs but may miss some relevant issue. They are also done periodically and are quite expensive.

Ideally, we would like to have some sort of protocol that levels the playing ground both for exploiters and *whitehats*. When en exploiter finds a bug, it only requires the creation of a transaction in some novel way such that after the transaction is executed, the funds are effectively stolen, the exploiters are sophisticated actors which are continually monitoring new projects and smart contracts. 
If we could build a protocol where *whitehats* (that is, "ethical exploiters") could also obtain a profit, albeit potentially smaller, in a *trustless* way, it would serve as a massive improvement.

## Objective
The objective is to create a *permissionless* protocol by using ZKPs (Zero Knowledge Proofs) to prove that an exploit was found without revealing the exploit in question. This also allows some other interesting properties:
1. Depending on the type of exploit (which can be proven to be in *some* category) an appropriate reward can be given **after** the proof is verified (thus, this allows crediting a *bounty* immediately after finding the exploit without revealing it, atomically).
2. After an exploit is found, an automatic defense mechanism can be triggered by the smart contract code, for example, freezing the contract until the exploit can be safely resolved by (for example) upgrading the smart contract. This ensures that once an exploit is found, an immediate action can be taken.

## Proposal

##### Preliminaries
Before properly defining the protocol, we will make some definitions that will be used later to explain the protocol.

###### State
The state $s$ is defined as a map where each key stores a value. It represents the current state of a blockchain and can be implemented in various manners in practice. For example, Ethereum uses a Merkle Patricia Trie to store the state.
In regards to our definitions, we can treat it as a function $s: \{0, 1\}^{log_2(A)} \rightarrow \{0, 1\}^{log_2(V)}$ where $A = 2^{160}$ is the number of all possible addresses and $V = 2^{256}$ is the number of all possible values.

###### Fixed Condition
A fixed condition over a state $s$ can be defined as:
$F_{s}(a, v): \{0, 1\}^{log_2(A)} \times \{0, 1\}^{log_2(V)} \rightarrow \{0, 1\}$ 
It takes as input an address $a$ and a value $v$ and outputs ```true``` if the value stored in the address $a$ in state $s$ matches the input $v$ or```false``` otherwise. 
For example $F_{s}(a,0) = s(a) > 0$ is the condition number one which is ```true``` if the value stored in address $a$ that belongs to state $s$ is a positive number.

**Relative Condition**
A relative condition over states $s$ and $s;$ can be defined as:
$R_{s,s'}(a, a'):  \{0, 1\}^{log_2(A)} \times \{0, 1\}^{log_2(A)} \rightarrow \{0, 1\}$ 
It takes as input an address $a$ from state $s$ and an address $a'$ from state $s'$ and outputs ```true``` if the value stored in address $a$ in state $s$ is equal to the value stored in address $a'$ in state $s'$ or ```false``` otherwise.
For example $R_{s,s'}(a, a') = s(a) > s'(a')$ is the condition which is ```true``` if the value stored at address $a$ in $s$ is greater than the value stored at address $a'$ in $s'$.

**Taxonomy of conditions**
We define a fixed condition $F$ as $(a_s, op, v)$ where:
- $a_s$: State address
- $op$: An operation, for now: $(=, \neq, \gt, \geq, \lt, \leq$)
- $v$: The expected value (for now a primitive type)

We also define a relative condition $R$ as $(a_{s}, op, a_{s'})$ where:
- $a_s$: State address
- $op$: An operation
- $a_{s'}$: End state address

The objective of conditions is to verify that:
- $F: s[a_s]$ $op$ $v$ is **true**
- $R: s[a_s]$ $op$ $s'[a_{s'}]$ is **true**

**Method**
A method $m(i, s): \{0, 1\}^* \times \tau \rightarrow \tau$ takes as input a set of bytes $i$ and a state $s$ and outputs another state $s'$, here $\tau$ is just the set of all possible states.
A **pure** method is one where $s'=s$.
We state that conditions are related to methods with a $1$ to $N$ cardinality, that is every method can have up to $N$ conditions acting on the state $s$ (fixed conditions) or $s$ and $s'$ (relative conditions). Importantly, there are no conditions without an associated method.

**Specification**
A **valid specification** $S$ is the set of tuples $(m, c_m)$ (methods and conditions over said method) where we expect that after applying $m$ the condition $c_m$ is always true, that is:
- $S: \{(m, c_m) \mid s' = m(s, i) \rightarrow c_m = 1 \forall c_m\}$

An **invalid specification** $S$ is the set of tuples $(m, c_m)$ where we expect that after applying any $m$ at least one condition $c_m$ is false, that is:
- $S: \{(m, c_m) \mid s' = m(s, i) \rightarrow \exists c_m = 0\}$

We use the shorthand $(*, c)$ to define a condition applied to all methods (such conditions are called **universal**).
##### Protocol
The objective of the protocol is to provide a framework and standardization so that proving exploits can be done without ambiguity.
In particular, we will define the specification of a program as the set of conditions $S = \{(m, c_m)\}$. As mentioned earlier this specification defines all states that are valid for a program $P$.

In particular, there are two things that could indicate that the specification is not followed:
1. After applying some method $m(i', s)$ we obtain a state $s'$ where some condition $C_i$ evaluates to ```false```.
2. There exists some condition $C_j$ such that $j \neq i$, which is not part of the protocol but **should** be.

These two cases must be handled separately, in this version we will only deal with the first problem, leaving the alternatives to solve the second problem to a future version.
###### Finding an exploit state
We say that a state $s$ is an exploit state when there is an address $a \in s$ such that $c(a, *) = 0$ (or $c(*,a) = 0$)

If there is a method $m$ with input $i_E$ which is called upon a state $s$ which is **valid** (that is, there exists no $a \in s$ such that $c(a,*) = 0$, or $c(*,a) = 0$) then, if the resulting state $s'$ is in an **exploit state**, we say that **an exploit has been found**.

If we can prove that the program $P$ was in the state $s$ and that applying $m$ leads to said state $s'$ then we can effectively prove that the program specification has an exploit. We call the input $i_E$ the *exploit input*.

Since we just want to **prove** that this exploit exists and we don't wish to execute it on the program $P$ itself, we have to **simulate** $P$ and run $m$ on it. After the execution, we can output a SNARK proof $\pi$ which should be verified by the actual program *on chain*.
To prevent $i_E$ from being leaked, the input exploit will be the witness $w$ (as opposed to the public input $x$) and a zk-SNARK will be used instead to ensure the *zero-knowledge* property.

More formally, the zk-SNARK must ensure that:
*I know a witness $w = i_E$ to some method $m$ such that $m(i_E, s) = s'$ and $C(a,*) = 0$ or $C(*,a)=0$ for some condition in the specification of $P$. [[ZKPoEX Notes#4]]*

The verifier residing *on chain* can have arbitrary logic related to what should happen after the verification is succesful.
In particular, each of the tuples defined in the specification could be **categorized** and specific rewards or actions can be taken depending on said category. This is left to future iterations of the project to explore.
## Implementation

To implement said protocol we are going to be using some tools:
1. As our zkVM we will be using [Risc0](https://github.com/risc0/risc0)
2. As our Ethereum VM interpreter we will be using [Rust EVM](https://github.com/rust-ethereum/evm)
3. As for the chain, any L2 should do, **but** since we are using an EVM interpreter, we will only support Type 1 chains

As the basis for our protocol we will be extending [zkPoEX](https://github.com/zkoranges/zkPoEX) which is a PoC implementation of this protocol:
- It uses Risc0 as a zkVM
- It uses SputnikVM (which is an older version of Rust EVM) as its EVM interpreter.

There are some changes to be done regarding their implementation:
- The logic for the program must change since right now the current implementation of zkPoEX is setting up some trivial exploit.
- The proposed scheme is not trustless

To comply with the protocol described above we will be attempting to implement said protocol with both parties, the *bounty hunter* or **prover** and the program (a smart contract *on chain*) or **verifier**.
###### Prover
The prover will look for a set of programs $P$ currently deployed on any chain and attempt to find some exploit so that a call to some method $m$ with some input $i_E$ leads to a state $s'$ such that $m(i_E, s) = s'$ and $c(*, *) = 0$ for any condition $c$

Once the prover finds this, it must create a proof of this fact. In order to do this, the prover must follow some steps:
1. As an prior step, a [guest program](https://dev.risczero.com/api/zkvm/guest-code-101) must be compiled to produce an [ELF binary](https://dev.risczero.com/terminology#elf-binary). This guest program will be a program that runs the EVM for some bytecode $b$ and some calldata $c$. More importantly, an [imageID](https://dev.risczero.com/terminology#image-id) which identifies said binary is stored in the verifier contract *on chain* which binds the verifier to **the guest program** that the prover will execute and prove.
2. As a first step, the prover compiles the smart contract in question and **[executes](https://dev.risczero.com/terminology#execute)** the[[ZKPoEx#Guest program]], obtaining an [execution trace](https://dev.risczero.com/terminology#execution-trace). This execution trace is then used to [prove](https://dev.risczero.com/terminology#prove) that the execution was correct, thus ensuring that the ELF binary was faithfully executed according to the rules of the RISC-V instruction set architecture.
3. Once the prover is done with the execution and proving, a [receipt](https://dev.risczero.com/terminology#receipt), which contains a [receipt claim](https://dev.risczero.com/terminology#receipt-claim) (public output) and a [seal](https://dev.risczero.com/terminology#seal) (the validity proof) is generated so that it can be sent to the verifier.

###### Verifier
As stated in the [[ZKPoEx#Prover]] section, the verifier is instantiated with an imageID, which binds it to a specific guest program (in this case, the program which executes some contract on an EVM interpreter and verifies if it fulfills a predetermined set of conditions, or specification, for said contract)

The verifier, apart from verifying the validity of the proof, must ensure some other properties related to the public input to fully be certain that the guest program was ran on the smart contract in question.
In particular, the public output consists of:
1. The hash of the program's specification used $SHA3(S)$
2. The hash of the target contract bytecode used $SHA3(b)$
3. The hash of any other context data (such as other deployed contracts) 

It is important to then verify that:
1. The current specification of the smart contract $S$ when hashed is equal to the **claimed** specification in the receipt claim.
2. The current bytecode of the smart contract $b$ when hashed is equal to the **claimed** bytecode in the receipt claim.
3. The current bytecode for the contextual data of the smart contract $b_c$ when hashed is equal to the **claimed bytecodes** of the contextual data

With this verifications we ensure that, the actual bytecode that was used in the guest program is in fact the smart contract **and** that the specification which it proves to *exploit* is also the current defined specification.
If any of these fail, it means that the proof was not created for the actual smart contract or that it was created under an outdated specification of it.

![zkpoex image](/images/zkpoex-excalidraw.png)

###### On chain protocol
Ideally, we would want said protocol to be on top of existing smart contract applications to prevent said applications from having to update. Thus we require the setup of a framework which can enable this functionality.

In order to do that, we must ensure that the deployed contract has the following set of functionalities:
1. In all cases, the proof $\pi$ must be relayed to the verifier contract and the claimed bytecode and specification must be checked against the actual bytecode and specification.
2. After that, since an exploit is found and verified correctly, then we can release an amount of locked funds to the sender of the proof.

When upgrading the set of contracts that define the program $P$, developers **must** upgrade the bytecode $b$ and specification $S$ accordingly. When this contract is initialized it emits a special event called *ZKPoEXInitialized* which can be listened to in order to know that some set of contracts are available to prove exploits.
###### Guest program
1. The guest program will perform the following operations:
	1. Read the inputs:
		1. The calldata to call the contract: $i_E$
		2. The initial state: $s$
			1. Containing the contract's address and bytecode $(a, b)$
			2. Containing the caller $(a'[, b'])$
			3. **(Optionally)** Other contract addresses and bytecodes $((a_1, b_1), ..., (a_m, b_m))$
		3. The program's specification: $S$ (a set of conditions)
	2. Prove that the initial state $s$ is **valid**:
		1. It contains the contract's bytecode $b$ and the rest of the bytecodes optionally
		2. For all conditions in the specification $S$, (including $c_j$ in case it exists) prove that for all $i$, $c_i(*,*) = 1$
	3. Call the contract (which is represented here by the bytecode) with $i_E$.
	4. The execution will produce a state $s'$. Here there are two things to do:
		1. Iterate through the conditions in $S$ and show that $c_i(*, *) = 0$ for at least one condition $c_i$
		2. If none of these were discovered, then panic.
	5. At last, the guest program must write private outputs to the [host](https://dev.risczero.com/terminology#host) and public outputs to the [journal](https://dev.risczero.com/terminology#journal) 
		1. Private inputs
			1. The calldata of the contract $i_E$
			2. The initial state $s$
		2. Public inputs (can be seen as outputs, but for the zkVM are inputs)
			1. The hash of the program's specification used $SHA3(S)$
			2. The hash of the bytecode used $SHA3(b)$
			3. **(Optionally)** The hash of other used addresses and bytecodes $SHA3((a_1, b_1) || ... || (a_m, b_m))$












