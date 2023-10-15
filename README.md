# Chaiwala
[![](https://img.shields.io/crates/v/chaiwala)](https://crates.io/crates/chaiwala)
[![](https://img.shields.io/docsrs/chaiwala)](https://docs.rs/chaiwala)
[![](https://img.shields.io/github/license/kanekoshoyu/chaiwala)](https://github.com/kanekoshoyu/chaiwala/blob/master/LICENSE)  

Chaiwala is a service layer for Kucoin Arbitrage, along with Continuous Deployment

## Introduction
Perfect algorithms and software architectures are not enough for algo-trading. A low latency network environment is needed to properly place order, which highlights the need of deployment to the cloud. As of now, Kucoin API has the lowest latency at AWS east japan, which suggests deployment on ECS Fargate using docker.  

### Monitoring via Discord
kucoin_arbitrage's monitor mod is modified as report mod in chaiwala, which sends the MPS counter report to Discord channel in real time.  

### Core Runtime Management via REST
set core's runtime status using GET command i.e.  
Enable: http://localhost:1080/set?status=Running  
Disable: http://localhost:1080/set?status=Idle  

### Docker
Build docker image locally: `docker build . -t local-chaiwala -f ./.deploy/local.dockerfile`  
Run local docker image: `docker run -p 80:1080 local-chaiwala:latest`

## Features to be Included
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
