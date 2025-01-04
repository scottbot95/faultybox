# Room API

### Auth

[//]: # (Don't actually need this I think...)
[//]: # (1. Client creates a "clientId" when joining a new room. Stored in cookie &#40;`faulty_client_id`?&#41; )
When joining a room the server will authenticate a client by:
1. Check if clientId is already in the room
   1. If so, expect `Faulty-Room-Token` header to be valid (matches hmac with local secret) otherwise reject with 401
   2. If connection accepted, kick previous connection if it existed
   3. If new client, generate a `Faulty-Room-Token` for them.
      1. Client then needs to store the token to present any time it re-connects

### Create
