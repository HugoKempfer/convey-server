# Convey - Universal and efficient file transfer

There are for sure plenty of awesome and efficient ways to achieve file transfer *but* (this repository wouldn't exist otherwise), here is my proposal.



## Overview of current technologies

I will try to make a non-exhaustive but representative list of how files are transfered nowadays.

### Cloud based

Lots of people are enjoying an "instantaneous" and contextualized way to share files via communication apps. This could be any personal/company messenger or collaborative platform. I'm also including in this category every cloud file storage provider.

Pro:

- widely adopted
- sequential upload/download that doesn't require peers to be up during the whole process
- generally using HTTP available in most places

Cons:

- fills servers with data that might not need to be stored a long period
- most providers does not provide end to end encryption
- (very) limited capacity with free plans
- HTTP resumability is not its strongest strength

### Client/Server

Some skilled user can setup servers that enable secured transfer like OpenSSH. That enable to use a wide ecosystem of tools, like scp, rsync etc.

Pro:

- secured, private, convenient, automatable

Cons:

- lots of burden to setup (port forwarding, account management...)
- those protocols often needs to be extensively configured because they permit more than simple file transfer

### Peer to Peer

Pro:

- efficient, decentralized and scalable
- particularly adapted for large files
- easily resumable

Cons:

- hard to setup for quick transfer and casual users
- often associated with illegal data transfer by most users

### Offline

Good old cold storage.

Pro:

- very eco-friendly
- privacy
- large file transfer

Cons: obvious



## What constraints convey needs to meet?

Here are the core pillars that I want convey to be based on:

- **Respect privacy and be secured** -> end to end encryption, open source code
- **Be decentralized** -> let's stop using someone's else computers! no data stored on third party servers, data should be conveyed in a P2P way
- **Be usable anywhere** -> no platform should be omitted, nowadays we have all technologies to include everyone :)
- **Be user friendly** -> anyone should be able to use it with minimum skills -> focus on ergonomics, GUI, internationalization
- **Free of charge, open and unlimited** -> no subscriptions, no transfer limits 



This seems to be obvious qualities that everyone wants, but I struggle to find a tools meeting all of these..

This also might seems to be an unreachable goal by some individual, *but* I think we have the matter to make it :)



## What is convey?

This will be a tool Desktop/Mobile/Web/CLI application that will rely on Bittorrent protocol with some superset tooling to transfer any kind of files.

A custom relay server will connect two or more peers together with a full-duplex TCP/WS/WebRTC (TCP will be preferred whenever possible) connexion based on a common file sharing session.

This session will be granted access with a common "key", this key can be under the form of PAKE, QR Code, local peer discovery or even later, with account system.

Once the users are in the file sharing session, they will share a symmetric key that will be used to encrypt all files E2E. The relay server will never get this previous key.

Then the initial seeder will compress, encrypt and upload a torrent file on a custom tracker. Then the transfer can be done via the Bittorrent protocol.



The goal of convey is to automatize and secure the whole torrenting process, the end user will not even necessarily be aware of the underlying used protocol.



## References

- https://github.com/webtorrent/bittorrent-tracker
- https://github.com/webtorrent/webtorrent-hybrid
- https://github.com/schollz/croc
- https://github.com/magic-wormhole/magic-wormhole
- https://file.pizza/
- https://www.sharedrop.io/