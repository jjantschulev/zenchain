# ZenChain

The final boss of blockchains. Mine your own zencoins now!!!

### How it works

well, this is a learning project where i try to make a blockchain to learn more about rust, cryptography, sockets and blockchain concepts.

Guaranteed to be free of bugs. (might contain spiders so tread carefully)

### how to use this

you need to generate a keypair first:

```
cargo run --bin client keys generate <name>
```

then you can start mining:

```
cargo run --release --bin node -- --port 8888
```

If you want your node to be visible to other nodes you can add it to `nodes.txt` and submit a pull request on GH.
Or convice other node runners to include your node through another communication channel.

### Wen Mint | AirDrop | Merge | etc...

idk go find a fish in the river and ask it. probably knows better than me...
