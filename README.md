<p align="center">
  <br>
  <br>
  <picture>
    <img alt="Puddle" src="./puddle.jpg" height="200">
  </picture>
  <br>
  <br>
</p>

# Puddle
Puddle is a CLI tool to interact with the [Raindrop.io API](https://developer.raindrop.io/).

# Installation
## Homebrew
`brew tap howenyap/tap`
`brew install puddle`

## Cargo
`cargo install puddle-cli`

# Setup
- Run `puddle setup`
- Head to raindrop.io [integrations](https://app.raindrop.io/settings/integrations) and create a new app
- Use `https://oauthdebugger.com/debug` for the redirect URI
- Copy your `Client ID` and `Client secret` into the CLI inputs
- Try running `puddle me`, you should see your account details and you're all set!

## Environment Variables
`puddle` can also load configuration from environment variables instead of `config.toml`.

Supported variables:
- `PUDDLE_CLIENT_ID`
- `PUDDLE_CLIENT_SECRET`
- `PUDDLE_REDIRECT_URI`
- `PUDDLE_ACCESS_TOKEN`
- `PUDDLE_REFRESH_TOKEN`

# Development
Make sure you have [Rust](https://rust-lang.org/tools/install/) installed.

## Pre-commit Hooks
- Install [prek](https://github.com/j178/prek)
- Install the local Git hooks `prek install`
- Manual run with `prek run`
