# TO DO STUFF

- [ ] write d.ts for socket (rust bindings)
- [ ] write wrapper code for bindings
- [ ] test bindings between node thread and rust threads

- [x] Create manager (resolved with gulp)
  - [x] Rust auto builder https://stackabuse.com/executing-shell-commands-with-node-js/

On socket initialization, check user session by JWT and register to UDP channel.
IP and port will recognize the user.
if jwt token expires in few minutes, refresh token.
