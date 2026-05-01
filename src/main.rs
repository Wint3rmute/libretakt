#[cfg(not(target_arch = "wasm32"))]
mod server;

// When compiling the PC/laptop sound engine server
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    server::main();
}

#[cfg(target_arch = "wasm32")]
mod frontend;

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    frontend::main();
}
