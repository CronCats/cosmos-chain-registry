&nbsp;

<div align="center">
<img width="300px" src="https://github.com/CronCats/croncat-rs/raw/main/croncat.png" />
</div>

&nbsp;

---

# cosmos-chain-registry

A Rust API for getting chain information from the [Cosmos Chain Registry](https://github.com/cosmos/chain-registry).

## Example:

```rust
use cosmos_chain_registry::ChainRegistry;

let registry = ChainRegistry::from_remote().unwrap();
let chain_info = registry.get_by_chain_id("juno-1").unwrap();

assert_eq!(info.chain_name, "juno");
assert_eq!(info.chain_id, "juno-1");
assert_eq!(info.pretty_name, "Juno");
```
