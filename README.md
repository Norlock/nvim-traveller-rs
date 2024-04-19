# Nvim traveller port
This is a port of the nivm-traveller plugin written in Lua. However I'm not a particular fan of scripted languages so I used neo-api-rs for this project. 

> [!NOTE]
> This is in active development so sometimes the `neo-api-rs` crate will point to my own location but you can toggle the comment in Cargo.toml to apply the github version (https://github.com/norlock/neo-api-rs)


## Roadmap
- [ ] Find a good way how to deal with autocmds together with async

## Startup (dev settings)
- Run `./prepare.sh` to setup the symlink to the lua directory
- Run `Cargo build --release` to recreate bin
- Add to plugins (e.g. lazy): `{ dir = '/path/to/nvim-traveller-rs' }`
- Add keymap: 

```lua
local nvim_traveller = require('nvim-traveller-rs')

vim.keymap.set('n', '<leader>i', nvim_traveller.open_navigation, {})
```
