```mermaid
flowchart LR

subgraph you[Your PC]
  obs([OBS]) -->|rtmp| pcs([PeCa Starter]) -->|rtmp| pst([PeerCastStation])
  pcs -->|http| pst
end

pst -->|pcp| yp([YP])

lpc([Listener's PeerCast]) -->|http| yp
subgraph listener[Listener's PC]
  player([Listener's Player]) --> lpc
  lpc -->|pcp| pst
end
```
