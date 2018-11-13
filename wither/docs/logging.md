### logging
This crate uses the [rust standard logging facade](https://docs.rs/log/), and integrating it with another logging framework is usually quite simple. You can very quickly and easily get up and running with [env_logger](https://docs.rs/env_logger/latest/env_logger/) or pretty much any other standard logging framework.

If you are using something a bit more exotic, like slog, check out the [slog-rs/stdlog](https://docs.rs/slog-stdlog/) create for easy integration.
