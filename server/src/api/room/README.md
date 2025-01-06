# Room API

### Creating Room
1. `POST /api/room/create/{gameId}`
   1. Receives a room token which contains a `roomId` and a client ID

### Joining Room
1. `POST /api/room/join/{roomId}`
   1. Receives a room token which contains a `roomId` and a client ID

### Connecting
1. Form websocket to `/api/room/connect` with the room token in the `Room-Token` header

Reconnecting is the same, just re-use the same room token and you will be treated as the same player


## TODO

- [ ] Boot other connections if a new one comes in with the same token
