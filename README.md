# DragonDNS

> [!CAUTION]
> A rewrite is currently going on over on the [rewrite](https://github.com/viktorpopp/dragon-dns/tree/rewrite) branch.

DragonDNS is an self-hosted Dynamic DNS service that can be configured on the server itself and the Cloudflare
Dashboard in the DNS entry comments.

## Why?

Why build this when [favonia/cloudflare-ddns](https://github.com/favonia/cloudflare-ddns) exists? Two reasons:
simplicity and load balancing.

Favonias DDNS requires creating DNS records in Cloudflare and editing the Docker Compose File on your server. DragonDNS
handles everything in the dashboard so no changes on the server is needed.

It also enables basic load balancing by using multiple DNS records, letting clients pick one at random without a
dedicated load balancer.
