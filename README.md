https://x.com/PeckShieldAlert/status/1877641906102129092
The objective is to research and analyse how an implementation of attempting to find on chain bugs with "zk-bounties" could look like technically.
## Introduction
One of the biggest issues with crypto nowadays is that exploits happen very often and most of them steal thousands or millions of dollars. ==This is very costly for any protocol and very risky for customers.==

==Software is not exempt of *bugs* and since it is development in the crypto industry tends to be fast paced with huge risks==, ==it usually means that there are interested agents looking for *bugs* and exploits to obtain profit. Hence, there is a big demand for novel use cases that can somehow mitigate this issues.==

==Typically, *bounties* and *audits* are done, but many times this is not enough.== 
*Bounties* typically carry some friction, since sometimes there is some ambiguity over found exploits, sometimes *bounty hunters* are not paid at all. [Find sources] 
*Audits* in the other hand can find potential bugs but may miss some relevant issue. They are also done periodically and are quite expensive.

Ideally, we would like to have some sort of protocol that levels the playing ground both for exploiters and *whitehats*. When en exploiter finds a bug, it only requires the creation of a transaction in some novel way such that after the transaction is executed, the funds are effectively stolen, the exploiters are sophisticated actors which are continually monitoring new projects and smart contracts. 
==If we could build a protocol where *whitehats* (that is, "ethical exploiters") could also do the same thing, but require a payout for said bug instead of whatever funds the contract holds then this would seem to simplify their task massively.==

## Objective
The objective is to create a *permissionless* protocol by using ZKPs (Zero Knowledge Proofs) to prove that an exploit was found without revealing the exploit in question. This also allows some other interesting properties:
1. Depending on the type of exploit (which can be proven to be in *some* category) an appropriate reward can be given **after** the proof is verified (thus, this allows crediting a *bounty* immediately after finding the exploit without revealing it).
2. After an exploit is found, an automatic defense mechanism can be triggered by the smart contract code, for example, freezing the contract until the exploit can be safely resolved by (for example) upgrading the smart contract. This ensures that once an exploit is found, nothing severe can occur.

==To enhance point 2==, we provide an optional output that allows the proving party to encrypt the vulnerability details along with the proof. The SC owners can define this property as mandatory, thus allowing proving parties that detail the vulnerability (with encryption) by providing some key for said encryption. This key is issued by the SC owners and can be rotated if needed, the public key to encrypt data is published (in the protocol's contract) so that it can be used by auditors.
Setting up this optional parameter allows the protocol to be sure that the witness used as an exploit can be shared with them in a safe way (as long as the decryption key is safe), thus allowing liquidity to flow immediately to auditors without additional friction. This also ensures that exploiters reveal the exploit so that the protocol can work out a fix.

## Proposal

##### Preliminaries
Before properly defining the protocol, we will make some definitions that will be used later to explain the protocol.
###### Fixed Condition
A fixed condition $C_F(s): s \rightarrow \{0, 1\}$ takes as input a state $s$ and outputs TRUE or FALSE if the condition is met or not respectively. Conditions are defined as an operation that compares some value of the state with some variable. For example $C_{F,1}(s) = s.a > 0$ is the condition number one which is TRUE if the variable $a$ that belongs to state $s$ is a positive number. [[ZKPoEX Notes#1.]]

**Relative Condition**
A relative condition $C_R(s, s'): s \times s \rightarrow \{0, 1\}$ takes as input two states $s$ and $s'$ and outputs TRUE or FALSE if the condition is met or not respectively. These conditions are defined as operations that compare the previous state $s$ and the current state $s'$. For example $C_{R,1}(s) = s.a > s'.a$ is the condition number one which is TRUE if the variable $a$ that belongs to the state $s$ is greater than the variable $a$ that belongs to state $s'$.

**Taxonomy of conditions**
We define a fixed condition $C_F$ as $(k_s, op, v)$ where:
- $k_s$: State key
- $op$: An operation, for now: $(=, \neq, \gt, \geq, \lt, \leq$)
- $v$: The expected value (for now a primitive type)

We also define a relative condition $C_R$ as $(k_{s}, op, k_{s'})$ where:
- $k_s$: State key
- $op$: An operation
- $k_{s'}$: End state key

The objective of conditions is to verify that:
- $C_F: s[k_s]$ $op$ $v$ is **true**
- $C_R: s[k_s]$ $op$ $s'[k_{s'}]$ is **true**

**Method**
A method $m(i, s): s \rightarrow s'$ takes as input a set of bytes $i$ and a state $s$ and outputs another state $s'$
A **pure** method is one where $s'=s$.
We state that conditions are related to methods with a $1$ to $N$ cardinality, that is every method can have up to $N$ conditions acting on the state $s$ (fixed conditions) or $s$ and $s'$ (relative conditions). Importantly, there are no conditions without an associated method.

**Specification**
A **valid specification** $S$ is the set of tuples $(m, c_m)$ (methods and conditions over said method) where we expect that after applying $m$ the condition $c_m$ is always true, that is:
- $S: \{(m, c_m) \mid s' = m(i, s) \land c_m(s, s') = 1\}$

An **invalid specification** $S$ is the set of tuples $(m, c_m)$ where we expect that after applying any $m$ at least one condition $c_m$ is false, that is:
- $S: \{(m, c_m) \mid s' = m(i, s) \land \exists c_m(s, s') = 0\}$

We use the shorthand $(*, c)$ to define a condition applied to all methods (such conditions are called **universal**).
##### Protocol
The objective of the protocol is to provide a framework and standardisation so that proving exploits can be done without ambiguity.
In particular, we will define the specification of a program as the set of conditions $S = \{C_1, C_2, ..., C_n\}$. As mentioned earlier this specification defines all states that are valid for a program $P$.

In particular, there are two things that could indicate that the specification is not followed:
1. After applying some method $m(i', s)$ we obtain a state $s'$ where some condition $C_i$ evaluates to FALSE.
2. There exists some condition $C_j$ such that $j \neq i$, which is not part of the protocol but **should** be.

These two cases must be handled separately, we will now explain how each of them would work under the protocol being presented.
###### Finding an exploit state
==We say that the state $s_E$ is an exploit state if when evaluated by any condition of the program $C_i$ it ==evaluates to false.==
==That is that the set of exploit states $S_E = \{s_E\}$ are the states where any condition evaluates to false.==

==Assume== that we can execute some method $m$ with some input $i_E$ and some state $s \in S_V$ where $S_V$ is the set of all **valid states** (states not in the set of exploit states) [[ZKPoEX Notes#2.]]

If we prove that $s$ is the current state of the program $P$ [[ZKPoEX Notes#3]] and that applying $m$ with input $i_E$ leads to some state $s'$ where $C_i(s') = 0$ for some $i$, then we have effectively proven an exploit on the specification of the program. In this case, we call $i_E$ the *exploit input*.

Since we just want to **prove** that this exploit exists and we don't wish to execute it on the program $P$ itself, we have to **simulate** $P$ and run $m$ on it. After the execution, we can output a SNARK proof $\pi$ which should be verified by the actual program *onchain*.
To prevent $i_E$ from being leaked, the input exploit will be our witness $w$ and a zk-SNARK will be used instead. 

With this proof, the prover asserts the following:
*I know a witness $w = i_E$ to some method $m$ such that $m(i_E, s) = s'$ and $C_i(s, s') = 0$ for some condition in the specification of $P$. [[ZKPoEX Notes#4]]*

TODO: Here explain how after the verification is done, then the contract can do automatic things
1. Since category of exploit can be determined a priori, then give funds for category of vulnerability
2. If vulnerability is very high, then somehow have rights to pause the contract

TODO: There is a possibility for the program to decide that the witness $w$ should be part of the pubic output $y$ BUT encrypted. In that case, the proof must also ensure that the witness $w$ that is used for the exploit is encrypted and is part of the output $y$, hence after proving the program $P$learns $Enc(w)$ and can decrypt (since the pk used for encrypting is also part of the pub input $x$)

TODO: Recall that anyone could steal the proof $\pi$ and claim the funds for themselves, we should ensure that some key is used in the proof so that the msg.sender of the proof (or whoever it claims to be) is signed in the proof.

###### Finding a new condition
If the specification is missing some condition $C_j$ where $j \neq i$ for any $i$ in the set of conditions, then this means that the specification is wrong, since there exists some condition $C_j$ that was unaccounted for.

In this case, there is no way to prove that the state $s'$ used to evaluate said condition actually belongs to $S_E$ at all. It may be that a malicious party is trying to trick the program $P$. Therefore, we require a way to prevent DoS attacks for this use case.

If a new condition is found, the protocol will require the following:
1. The proving party must lock collateral funds to be defined by the program $P$ to be able to interact with the contract to add a condition $C_j$
2. The witness $w$ that was used to show that there exists a method $m$ such that $m(w,s) = s'$ and $C_j(s') = 0$ is automatically **decrypted** after approximately $T$ time.

The first condition, ensures that ==DoSing== the program $P$ is mitigated as much as possible since it would require locking funds for multiple hours [[ZKPoEX Notes#5]]
The second condition, ensures that if the witness **is actually an exploit**, then a malicious party can use said exploit in the program $P$ to obtain some benefit! Therefore, the program $P$ is incentivized to add this newly found condition $C_j$ to the specification.

When a new condition is added to the specification, it first verifies if there are any new condition claims. If there are, then it rewards the proving party with funds. Unfortunately, we cannot know *a priori* what the effects of the conditions would be (so here the program $P$ must act faithfully and assign the proper amount of funds). We can guarantee the lowest amount for the proving party, but not necessarily much more than that. [[ZKPoEX Notes#6]]

**Add**: [[ZKPoEX R&D#**Research about obtaining the encryption key after some time/compute**]]
## Implementation

To implement said protocol we are going to be using some tools:
1. As our zkVM we will be using [Risc0](https://github.com/risc0/risc0)
2. As our Ethereum VM interpreter we will be using [Rust EVM](https://github.com/rust-ethereum/evm)
4. As for the chain, any L2 should do, **but** since we are using an EVM interpreter, we can only do verification on Type 1 chains [[ZKPoEX Notes#7]]

As the basis for our protocol we will be extending [zkPoEX](https://github.com/zkoranges/zkPoEX) which is a PoC implementation of this protocol:
- It uses Risc0 as a zkVM
- It uses SputnikVM (which is an older version of Rust EVM) as its EVM interpreter.

There are some changes to be done regarding their implementation:
- The logic for the program must change since right now the current implementation of zkPoEX is setting up some trivial exploit.
- ==This scheme is not trustless, it involved a third party. Thus the implementation just hashed the witness and gave it as public input. There are some messages exchanged between the bounty hunter and the DAO and some escrow acts as a middleman, but with this protocol we want to ensure that there is no third party.==

To comply with the protocol described above we will be attempting to implement said protocol with both parties, the *bounty hunter* or **prover** and the program (a smart contract onchain) or **verifier**.
###### Prover
The prover will look for a set of programs $P$ currently deployed on any chain and attempt to find some exploit so that a call to some method $m$ with some input $i_E$ leads to a state $s'$ such that $m(i_E, s) = s'$ and $C_i(s, s') = 0$ for any $i$ 

Once the prover finds this, it must create a proof of this fact. In order to do this, the prover must follow some steps:
0. As an initial step, a [guest program](https://dev.risczero.com/api/zkvm/guest-code-101) must be compiled to produce an [ELF binary](https://dev.risczero.com/terminology#elf-binary). This guest program will be a program that runs the EVM for some bytecode $b$ and some calldata $c$. More importantly, an [imageID](https://dev.risczero.com/terminology#image-id) which identifies said binary is stored in the verifier contract *on chain* which binds the verifier to **the guest program** that the prover will execute and prove.
1. As a first step, the prover compiles the smart contract in question and **[executes](https://dev.risczero.com/terminology#execute)** the[[ZKPoEx#Guest program]], obtaining an [execution trace](https://dev.risczero.com/terminology#execution-trace). This execution trace is then used to [prove](https://dev.risczero.com/terminology#prove) that the execution was correct, thus ensuring that the ELF binary was faithfully executed according to the rules of the RISC-V instruction set architecture.
2. Once the prover is done with the execution and proving, a [receipt](https://dev.risczero.com/terminology#receipt), which contains a [receipt claim](https://dev.risczero.com/terminology#receipt-claim) (public output) and a [seal](https://dev.risczero.com/terminology#seal) (the validity proof) is generated so that it can be sent to the verifier.

###### Verifier
As stated in the [[ZKPoEx#Prover]] section, the verifier is instantiated with an imageID, which binds it to a specific guest program (in this case, the program which executes some contract on an EVM interpreter and verifies if it fulfills a predetermined set of conditions, or specification, for said contract)

The verifier, apart from verifying the validity of the proof, must ensure some other properties related to the public input to fully be certain that the guest program was ran on the smart contract in question.
In particular, the public output consists of:
1. The encrypted calldata $Enc(c)$
2. The hash of the program's specification used $SHA3(S)$
3. The hash of the bytecode used $SHA3(b)$

It is important to then verify that:
1. The current specification of the smart contract $S$ when hashed is equal to the **claimed** specification in the receipt claim.
2. The current bytecode of the smart contract $b$ when hashed is equal to the **claimed** bytecode in the receipt claim.

With this verifications we ensure that, the actual bytecode that was used in the guest program is in fact the smart contract **and** that the specification which it proves to *exploit* is also the current defined specification ==on it.==
If any of these fail, it means that the proof was not created for the actual smart contract or that it was created under an outdated specification of it.

###### On chain protocol
Ideally, we would want said protocol to be on top of existing smart contract applications to prevent said applications from having to update. Thus we require the setup of a framework which can enable this functionality.

In order to do that, we must ensure that the deployed contract has the following set of functionalities:
1. In all cases, the proof $\pi$ must be relayed to the verifier contract and the claimed bytecode and specification must be checked against the actual bytecode and specification.
2. After that, we have divergent paths depending on the use case
	1. **Finding an exploit**
			In this case, since an exploit is found and verified correctly, then we can release an amount of locked funds to the sender of the proof. [[ZKPoEX Notes#8]]
	2. **Finding a new condition**
		1. In this case, since an exploit is not found but rather a new condition is being proposed to the set of specifications, the sender and the encrypted witness must be saved until the condition is added to the specification. When added, the sender receives an amount of locked funds.
		2. At the time of verifying, the sender must also send a collateral in whatever the contract requires to mitigate possible DoS attacks over the smart contract.

When upgrading the set of contracts that define the program $P$, developers **must** upgrade the bytecode $b$ and specification $S$ accordingly. When this contract is initialized it emits a special event called *ZKPoEXInitialized* which can be filtered by *whitehats* to know that some set of contracts are available to prove exploits.

[[ZKPoEX Notes#9]]
###### Guest program
3. The guest program will perform the following operations:
	1. Read the inputs:
		2. The calldata to call the contract: $c$
		3. The initial state: $s$
			1. Containing the contract's address and bytecode $(a, b)$
			2. Containing the caller $(a'[, b'])$ (could be EOA or contract).
			3. **(Optionally)** Other contract addresses and bytecodes $((a_1, b_1), ..., (a_m, b_m))$
		5. The program's specification: $S$ (a set of conditions)
		6. The public key of the protocol $pk$
		7. **(Optionally)** A new condition: $C_j$
	3. Prove that the initial state $s$ is **valid**:
		1. It contains the contract's bytecode $b$ and the rest of the bytecodes optionally
		2. For all conditions in the specification $S$, (including $C_j$ in case it exists) prove that for all $i$, $C_i(s) = 1$
	4. Call the contract (which is represented here by the bytecode) with $c$.
	5. The execution will produce a state $s'$. Here there are two things to do:
		1. Iterate through the conditions in $S$ and show that $C_i(s, s') = 0$ for any $i$ (**Finding an exploit**)
		2. If this does is not true for any $i$ and a condition $C_j$ was sent as input then show that $C_j(s, s') = 0$ **(Finding a new condition)**
		3. If none of these were discovered, then panic.
	6. In case there is a plausible exploit in the contract:
		1. If an exploit was found, then we encrypt the calldata $c$ with the public key $pk$ of the protocol $Enc(pk, c)$
		2. If a new condition was found, then still not sure [[ZKPoEX R&D#Research about obtaining the encryption key after some time/compute]]
	7. At last, the guest program must write private outputs to the [host](https://dev.risczero.com/terminology#host) and public outputs to the [journal](https://dev.risczero.com/terminology#journal) 
		1. Private inputs
			1. The calldata of the contract $c$
			2. The initial state $s$****
		2. Public inputs (can be seen as outputs, but for the zkVM are inputs)
			1. The public key of the protocol: $pk$
			2. The encrypted calldata $Enc(c)$
			3. The hash of the program's specification used $SHA3(S)$
			4. The hash of the bytecode used $SHA3(b)$
			5. **(Optionally)** The hash of other used addresses and bytecodes $SHA3((a_1, b_1) || ... || (a_m, b_m))$

## A simple example (PoC)

We shall now describe a very simple example with only one condition:
- The balance of the contract must be greater than 0 for any state transition

With that in mind we will first setup the program specification:
$S = \{ ((a$.$b, \gt, 0), m)\}$
Where
$a$: Contract address
$b$: Balance key on the Account
$m$: The contract's method associated to the condition, in this case: ```exploit(bool)```

This is what we call a composed key, since from the state trie view, we must first access the contract and then the balance.

The contract will have two methods:
1. ```function exploit(bool _exploit) public;```
2. ```function supposedly_no_exploit(uint256 _number) public;```

The PoC is designed to show the two possible cases:
1. **(Finding an exploit state)** Calling the method ```exploit``` and effectively showing that the end state $s'$ does not comply with the program specification $S$
2. **(Finding a new condition)** Calling the method ```supposedly_no_exploit``` and showing that if there existed a condition $C_j$ (where currently $C_j \notin S$), then the program specification would not comply with the end state $s'$












