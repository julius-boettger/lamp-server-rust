# Lamp Server
REST API built with [axum](https://github.com/tokio-rs/axum) to manipulate [supported Govee lamps through its Developer API](https://govee.readme.io/reference/govee-developer-api).

The main feature of this project (which the [Govee Home app](https://play.google.com/store/apps/details?id=com.govee.home) does not support) is its use as a sunrise alarm clock.

### Configuration
To configure Govee API access, **make sure you have a `lamp-server.yaml`** at the path for your operating system below.
- Linux: `$XDG_CONFIG_HOME/` or `$HOME/.config/` 
- MacOS: `$HOME/Library/Application Support/`
- Windows: `%APPDATA%\`

##### Config File Template
```yaml
govee_api_key: "00000000-0000-0000-0000-000000000000"
govee_device: "00:00:00:00:00:00:00:00"
govee_model: "00000"
```