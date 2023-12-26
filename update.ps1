# Git pull
git pull

# Cargo install
cargo install --path .

# Change directory to 'runtime' and build with release configuration
Set-Location -Path "runtime"
cargo build --release
Set-Location -Path ".."

# Install Janus using Cargo
cargo install --path janus