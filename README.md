# Convey Server

This repository holds the session server for the convey app.

See [MOTIVATIONS.md](https://github.com/HugoKempfer/convey-server/blob/main/README.md).

Documentation and specification on this server can be found under the OpenAPI3 format in the `spec/` folder.

## Technologies

The server is written in Rust using [Actix and Actix-web frameworks](https://actix.rs/).

It uses Redis as a cache, so there are no internal states.



## Testing

To test the project there must be an open Redis instance running on your machine. This is used for integration tests. You can use the one provided in `dev/docker-compose.yml` file.

