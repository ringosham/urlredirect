# URLRedirect

A simple client to host URL redirection from Linux host to Windows virtual machines. Supports libvirt and Docker.

This runs a tiny web server on the host machine that opens URLs when the guest machine sends a request.

Takes about ~2.2MiB of memory with glibc.

## Usage (Host side)

A systemd unit is provided but its simple enough to run it on any init systems

1. Build the project using `cargo build --release`.
2. Just run it with `./target/release/urlredirect` and it will start listening on port 10080.
   * You can specify a different port with the `-p` flag.
   * It will look for your local IP of your libvirt/Docker bridge. By default, it only looks for libvirt
   * Set `DOCKER="1"` to make it look for Docker.

## Usage (Guest side)

1. Put both files in the client/ directory on the guest machine.
2. Put RemoteBrowser.vbs to C:\
3. Run RemoteBrowser.reg to register it as a "Web browser"
4. Go to "Settings > Apps > Default apps" and set "Web browser" to "RemoteBrowser"
   * For Windows 11, you will need to set the http and https protocols to RemoteBrowser instead