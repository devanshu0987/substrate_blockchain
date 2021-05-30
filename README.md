# Auction Marketplace for Instagram

- This stores the substrate blockchain for Auction
- Includes 2 pallets
  - Auction pallet : Stores auction related data
  - BalancesAndOwnership pallet : Stores balance and ownership of resource data
    - resource is any thing for which we want to maintain ownership details. For example : Images, Videos etc.
    - we only store hash for the resource
- [Custom Pallets implemented Link](https://github.com/devanshu0987/substrate_blockchain/tree/main/pallets)

# How to run

- First open [https://23.100.29.42](https://23.100.29.42)
  - You will get certificate error. Select proceed and then you will get nginx error
  - At this point, your IP is whitelisted by the server and it will allow you to access the project
  - Otherwise, you will get error that cant connect to websocket in console.
- Blockchain is hosted at "wss://23.100.29.42". Open this in Polkadot portal. Go here are open this in [Polkadot apps portal](https://polkadot.js.org/apps/#/explorer)

# Frontend

- The frontend is hosted at [Link](https://github.com/devanshu0987/node-express-boilerplate)
- Read how to run portion
