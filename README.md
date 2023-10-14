# Chaiwala
[![](https://img.shields.io/crates/v/chaiwala)](https://crates.io/crates/chaiwala)
[![](https://img.shields.io/docsrs/chaiwala)](https://docs.rs/chaiwala)
[![](https://img.shields.io/github/license/kanekoshoyu/chaiwala)](https://github.com/kanekoshoyu/chaiwala/blob/master/LICENSE)  

Chaiwaka is a service layer for Kucoin Arbitrage, along with Continuous Deployment

### Introduction
Having the perfect algorithm and software architecture is not enough for algo-trading. A low latency network environment is needed to properly place order, which highlights the need of deployment to the cloud.  
As of now, Kucoin API has the lowest latency at AWS east japan, which suggests deployment over ECS or similar services using docker. To facilitate the effective remote debug reports and performance reports, this repo was set up to experiment hosting a webserver in event-driven async rust.

### Features to be Included
| Feature                                      | API         | Status    |
| -------------------------------------------- | ----------- | --------- |
| System warning report                        | Discord bot | Available |
| Arbitrage performance report via Discord bot | Discord bot | Available |
| Release build                                | Docker      | Pending   |
| AWS Continuous Deployment                    | Docker      | Pending   |
| Remote request process                       | REST        | Available |
| Process management                           | REST        | Available |
| Arbitrage broadcast                          | WebSocket   | Pending   |


### Discord Server
[Join my Discord](https://discord.gg/q3j5MYdwnm)
