# :electric_plug: noti-lsp

**noti-lsp** is a language server for the `noti` layout DSL used by [Noti](https://github.com/noti-rs/noti).

## :star2: Features

| status | feature             |
| :----: | :------------------ |
|   ✅   | Hover               |
|   ✅   | Diagnostics         |
|   ✅   | Completion          |
|   ✅   | Rename              |
|   🚧   | Go to definition    |

## :inbox_tray: Installation

### Build from source

```bash
cargo build --release
```

Binary will be available at: ```./target/release/noti-lsp```

## :wrench: Editor setup

### Neovim (0.11+)

Requirements: [noti.nvim](https://github.com/noti-rs/noti.nvim)

```lua
vim.lsp.config.noti = {
    cmd = { "/absolute/path/to/noti-lsp" },
    filetypes = {
        "noti",
    },
}

vim.lsp.enable("noti")
```

## :bug: Troubleshooting

- Make sure the binary path is correct
- Check LSP logs if the server doesn’t start

## :handshake: Contributing

Interested in improving **Noti**? Here's how to contribute:

1. Fork the repo and create your branch:

   ```bash
   git checkout -b feature/my-improvement
   ```

2. Make your changes and commit them:

   ```bash
   git commit -am "Describe your changes"
   ```

3. Push your changes:

   ```bash
   git push origin feature/my-improvement
   ```

4. Open a Pull Request

> [!NOTE]
> For major changes, please open an issue first to discuss the changes you'd like to make.
